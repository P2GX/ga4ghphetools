//! GA4GH Phenotools Repository
//! This class is used to model a file-based repository with directories and files created by this software

use std::{collections::HashMap, fs::File, io::Write, path::{Path, PathBuf}, sync::Arc};
use std::collections::BTreeSet;
use log::{error, info, trace};
use ontolius::ontology::{MetadataAware, csr::FullCsrOntology};
use serde::Serialize;
use crate::{
    cohort_qc::cohort_dir::CohortDir, 
    error::ontology_error::OntologyError, 
    ppkt::ppkt_updater::PpktUpdater, 
    repo::{self, cohort_dir_iter::CohortDirIter, cohort_wrapper::CohortWrapper, update_report::UpdateReport}};
use walkdir::WalkDir;

use crate::{
    dto::cohort_data::CohortData, 
    error::{PheToolsError, cohort_error::CohortError}, 
    hpo::self};

/// A structure representing the entire phenopacket store (GA4GHPhenoTools) repository
pub struct GptRepository {
     /// Path of the overarching root repository containing multiple gene folders with cohorts
     /// Either phenopacket store or a similarly structured repository
    pub phenopacket_store_path: PathBuf,
    /// Map of all Cohort directories (one per directory (usually: gene) in phenopacket store)
    cohort_map: HashMap<PathBuf, CohortDir> 
}

#[derive(Debug, Clone, Serialize)]
pub struct UpdateProgress {
    pub current: usize,
    pub total: usize,
    pub cohort: String,      // acronym, for display
}


impl GptRepository {
    pub fn new(root_path: &Path) -> Result<Self, PheToolsError> {
        let mut cohort_map: HashMap<PathBuf, CohortDir> = HashMap::new();
        let entries = WalkDir::new(root_path)
            .min_depth(1)
            .max_depth(1)
            .into_iter()
            .filter_map(|e| e.ok());
        for entry in entries {
            if entry.file_type().is_dir() {
                let dir_path = entry.path().to_path_buf();
                let cohort_dir = CohortDir::process_gene_directory(&dir_path)?;
                //Self::process_directory(entry.path());
                //cohort_dir.get_ppkt_map();
                cohort_map.insert(dir_path, cohort_dir);
            }
        }
    
        info!("Ingested {} gene directories.", cohort_map.len());
        Ok(Self {
            phenopacket_store_path: root_path.into(),
            cohort_map,
        })
    }

     pub fn cohort_dirs(&self) -> std::io::Result<CohortDirIter> {
        CohortDirIter::new(&self.phenopacket_store_path)
    }

    /// Reads and parses a single cohort JSON file.
    pub fn read_cohort(path: &Path) -> Result<CohortData, CohortError> {
        let file_data = std::fs::read_to_string(path)
            .map_err(|e| CohortError::io_error(path, &e.to_string()))?;
        let cohort: CohortData = serde_json::from_str(&file_data)
            .map_err(|e| CohortError::json_error(&file_data, &e.to_string()))?;
        Ok(cohort)
    }

    /// Reads every cohort file listed in `dir`, one at a time. A failure on any
    /// single file does not prevent the others from being read — the caller gets
    /// a result per path and decides how to report failures (e.g. as QcReports)
    /// rather than losing the whole directory to one bad file.
    pub fn read_all_cohorts(dir: &CohortDir) -> Vec<(PathBuf, Result<CohortData, CohortError>)> {
        dir.individuals_json
            .iter()
            .map(|p| (p.clone(), Self::read_cohort(p)))
            .collect()
    }

    pub fn get_all_cohort_wrappers(&self) -> Result<Vec<CohortWrapper>, PheToolsError> {
        let mut cohort_wrap_list = Vec::new();
        for (path, cohort_dir) in self.cohort_map.iter() {
            cohort_wrap_list.extend(cohort_dir.cohort_wrapper_list.clone());
        }
        Ok(cohort_wrap_list)
    }

    pub fn get_cohort_dir_list(&self) -> Vec<CohortDir> {
        self.cohort_map.values().cloned().collect()
    }


    /// Saves a single `CohortData` instance to its original path.
    ///
    /// This is intended to be used when `CohortData` has been edited, such as 
    /// when updating HPO IDs or term labels.
    ///
    /// # Arguments
    /// * `cohort` - The `CohortData` structure to be serialized and saved.
    /// * `path` - The file path where the JSON data should be written (full path including file name).
    ///
    /// # Errors
    /// Returns a `CohortError` if serialization fails, if the file cannot be created, 
    /// or if writing to disk fails.
     pub fn save_template_json(&self, cohort: &CohortData, path: &Path) -> Result<(), CohortError> { 
        let json = serde_json::to_string_pretty(&cohort).map_err(|e| CohortError::io_error(path, &e.to_string()))?;
        let mut file = File::create(&path).map_err(|e|CohortError::io_error(path, &e.to_string()))?;
        file.write_all(json.as_bytes()).map_err(|e|CohortError::io_error(path, &e.to_string()))?;
        Ok(())
    }

    /// Write an updated cohort back to disk, together with its regenerated phenopackets.
    ///
    /// Called by [`GptRepository::update_all_ppkt`] after the HPO term identifiers or
    /// labels of a cohort have been updated. The cohort is written back to the file it
    /// was read from, and every phenopacket of the cohort is regenerated from it.
    ///
    /// # Expected directory layout
    ///
    /// ```text
    /// <store>/
    ///   FBN1/                          <- gene directory
    ///     MFS_individuals.json         <- cohort file, `cohort_w.cohort_path()`
    ///     phenopackets/                <- sibling directory that receives the phenopackets
    /// ```
    ///
    /// The `phenopackets` directory is located relative to the **parent directory** of the
    /// cohort file, not inside the cohort file's path.
    ///
    /// # What it does
    ///
    /// 1. Regenerates the phenopackets from the updated [`CohortData`], using the most recent
    ///    ORCID in the cohort's curation history, and writes them to `phenopackets/`.
    ///    Existing files with the same name are overwritten.
    /// 2. Overwrites the cohort file with the updated cohort, as pretty-printed JSON.
    ///
    /// The phenopackets are written first on purpose. If step 1 fails, the cohort file is
    /// left unchanged, so it still has outdated HPO terms and a later update run will try
    /// the cohort again. Writing the cohort file first could leave stale phenopackets that
    /// are never regenerated.
    ///
    /// # Arguments
    ///
    /// * `cohort_w` - the cohort, its path on disk, and its existing phenopackets.
    /// * `hpo` - the HPO ontology used to build the phenopackets.
    ///
    /// # Errors
    ///
    /// Returns a [`PheToolsError`] if
    /// * the cohort has no curation history (no ORCID to attribute the phenopackets to),
    /// * the cohort path has no parent directory,
    /// * the phenopackets cannot be generated or written, or
    /// * the cohort file cannot be serialized, created or written.
    ///
    /// # Caveats
    ///
    /// The operation is not atomic. If it fails after some phenopackets were written, the
    /// `phenopackets` directory can contain a mix of old and new files.
    pub fn write_updated_cohort(
        &self,
        cohort_w: &CohortWrapper,
        hpo: Arc<FullCsrOntology>,
    ) -> Result<(), PheToolsError> {
        let cohort_data = cohort_w.cohort_data();
        let orcid = cohort_data.get_orcid()?; // most recent ORCID in the curation history
        let cohort_path = cohort_w.cohort_path();
        let gene_dir = cohort_path.parent().ok_or_else(|| {
            PheToolsError::message(format!("Cohort path has no parent: {}", cohort_path.display()))
        })?;
        let ppkt_dir = gene_dir.join("phenopackets");

        // 1. phenopackets first (see "What it does")
        let overwrite = true;
        let n = crate::write_phenopackets(cohort_data.clone(), ppkt_dir, orcid, hpo, overwrite)?;

        // 2. then the cohort file
        self.save_template_json(cohort_data, cohort_path)?;   // existing helper; error includes the path

        info!("Updated cohort {}: {} phenopackets written", cohort_data.acronym(), n);
        Ok(())
    }

    /// Check that every cohort's gene directory has a `phenopackets` subdirectory.
    ///
    /// All problems are collected and reported together, so the user can repair the store
    /// in one pass. Nothing is written or modified.
    ///
    /// # Errors
    /// Returns a [`PheToolsError`] listing each gene directory that has no `phenopackets`
    /// subdirectory (or whose cohort path has no parent directory).
    fn check_phenopackets_dirs(cohorts: &[CohortWrapper]) -> Result<(), PheToolsError> {
        let problems: BTreeSet<String> = cohorts
            .iter()
            .filter_map(|w| {
                let path = w.cohort_path();
                match path.parent() {
                    None => Some(format!("{} (cohort path has no parent directory)", path.display())),
                    Some(gene_dir) => {
                        let dir = gene_dir.join("phenopackets");
                        (!dir.is_dir()).then(|| dir.display().to_string())
                    }
                }
            })
            .collect(); // BTreeSet: sorted, and one entry per gene directory

        if problems.is_empty() {
            return Ok(());
        }
        Err(PheToolsError::message(format!(
            "Cannot update: {} gene director{} without a 'phenopackets' subdirectory. \
             No files were changed.\n{}",
            problems.len(),
            if problems.len() == 1 { "y is" } else { "ies are" },
            problems.into_iter().collect::<Vec<_>>().join("\n"),
        )))
    }


    /// Process all cohort files in the directory.
    ///
    /// This method is intended to be used to update Phenopacket Store using the `path` argument 
    /// passed to the constructor of `GptRepository`. For each subdirectory in path (these will be gene symbols,
    /// each of which contains one or multiple cohort files representing diseases associated with the gene), 
    /// it processes each of the cohort files.
    ///
    /// It checks whether the HPO identifiers and term labels are up-to-date; if not, it tries to replace them with 
    /// the current up-to-date versions and writes the file back to disk. If everything is up-to-date, the file is skipped. 
    ///
    /// # Returns
    /// An `UpdateReport` object used to present the update results to the user.
    pub fn update_all_ppkt<F>(
            &self, 
            hpo: Arc<FullCsrOntology>,
            mut on_progress: F) 
                -> Result<UpdateReport, PheToolsError> 
                where F: FnMut(UpdateProgress)
        {
        let mut report = UpdateReport::new(&self.phenopacket_store_path);
        let hpo_version = hpo.version();
        trace!("update_all_ppkt with HPO version {}", hpo_version);
        let cohort_w_list: Vec<CohortWrapper> = self.get_all_cohort_wrappers()?;
        Self::check_phenopackets_dirs(&cohort_w_list)?;
        let total = cohort_w_list.len();
        info!("Got {} cohort wrappers from {} gene directories.", cohort_w_list.len(), self.cohort_dir_count());
        for (i, cohort_w) in cohort_w_list.into_iter().enumerate() {
            let acronym = cohort_w.cohort_data().acronym(); 
            on_progress(UpdateProgress {current: i+1, total, cohort: acronym.clone()});
            let (updated_duplets, changed) = match hpo::sync_hpo_duplets(hpo.clone(), &cohort_w.cohort_data().hpo_headers) {
                Ok(x) => x,
                Err(e) => { error!("Could not check {}: {}", acronym, e); report.processed(); continue; }
            };
            if changed {
                info!("About to write updated cohort {}", acronym);
                let updated_cohort_w = cohort_w.update_headers(updated_duplets, hpo_version);
                match self.write_updated_cohort(&updated_cohort_w, hpo.clone()) {
                    Ok(()) => report.updated(),
                    Err(e) => { error!("Could not update {}: {}", acronym, e); report.processed(); }
                }
            } else {
                report.processed();
            }
        }
        info!("Processed {:?} cohorts of which {:?} were updated.", report.n_processed(), report.n_updated());
        Ok(report)
    }

    pub fn cohort_dir_count(&self) -> usize {
        self.cohort_map.len()
    }

}

#[cfg(test)]
mod tests {
    use rstest;
    use super::*;
    use crate::test_utils::fixtures::hpo;

    #[rstest::rstest]
    fn test_gpt_repository_initialization_and_qc(hpo: Arc<FullCsrOntology>) {
        let temp_dir = std::env::temp_dir().join(format!("gpt_repo_test_{}", uuid_or_random()));
        let gene_dir = temp_dir.join("BRCA1");
        std::fs::create_dir_all(&gene_dir).expect("Failed to create temporary gene directory");
        let repo = GptRepository::new(&temp_dir).unwrap();
        assert_eq!(repo.phenopacket_store_path, temp_dir);
        assert_eq!(repo.cohort_map.len(), 1);
    }

    fn uuid_or_random() -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64
    }

}