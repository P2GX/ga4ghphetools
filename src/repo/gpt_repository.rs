//! GA4GH Phenotools Repository
//! This class is used to model a file-based repository with directories and files created by this software

use std::{collections::HashMap, fs::File, io::Write, path::{Path, PathBuf}, sync::Arc};
use log::{error, info, trace};
use ontolius::ontology::{MetadataAware, csr::FullCsrOntology};
use crate::{cohort_qc::cohort_dir::CohortDir, error::ontology_error::OntologyError, ppkt::ppkt_updater::PpktUpdater, repo::{self, cohort_dir_iter::CohortDirIter, cohort_wrapper::CohortWrapper, update_report::UpdateReport}};
use walkdir::WalkDir;

use crate::{
    dto::cohort_data::CohortData, 
    error::{PheToolsError, cohort_error::CohortError}, 
    hpo::self};

/// A structure representing the entire phenopacket store (GA4GHPhenoTools) repository
pub struct GptRepository {
     /// Path of the overarching root repository containing multiple gene folders with cohorts
     /// Either phenopackete store or a similarly structured repository
    pub phenopacket_store_path: PathBuf,
    /// Map of all Cohort directories (one per directory (usually: gene) in phenopacket store)
    cohort_map: HashMap<PathBuf, CohortDir> 
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


    pub fn write_updated_cohort(&self, cohort_w: &CohortWrapper, hpo: Arc<FullCsrOntology>) -> Result<(), PheToolsError> {
        /// First write the updated CohortData
        let cohort_data = cohort_w.cohort_data();
        let orcid = cohort_data.get_orcid()?; // use existing ORCID id, latest used for this cohort
        let path = cohort_w.cohort_path();
        let json = serde_json::to_string_pretty(&cohort_data).map_err(|_|"Could not serialize to JSON".to_string())?;
        let mut file = File::create(&path).map_err(|_|"Could not create file".to_string())?;
        file.write_all(json.as_bytes()).map_err(|_|"Could not write file".to_string())?;
        /// Now write the updated phenopackets
        /// We use the updated CohortData to create updated phenopackets and write them.
        let ppkt_path: PathBuf = path.join("phenopackets");
        let overwrite = true;
        crate::write_phenopackets(cohort_data.clone(), ppkt_path, orcid, hpo.clone(), overwrite)?;
        info!("Wrote updated phenopackets for cohort {}; n={} phenopackets written", cohort_data.acronym(), cohort_data.rows.len());
        Ok(())
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
    pub fn update_all_ppkt(&self, hpo: Arc<FullCsrOntology>) -> Result<UpdateReport, PheToolsError> {
        let mut report = UpdateReport::new(&self.phenopacket_store_path);
        let hpo_version = hpo.version();
        trace!("update_all_ppkt with HPO version {}", hpo_version);
        let cohort_w_list: Vec<CohortWrapper> = self.get_all_cohort_wrappers()?;
        info!("Got {} cohort wrappers from {} gene directories.", cohort_w_list.len(), self.cohort_dir_count());
        for cohort_w in cohort_w_list {
            let cohort_data = cohort_w.cohort_data();
            let (updated_duplets, changed) = hpo::sync_hpo_duplets(hpo.clone(), &cohort_data.hpo_headers)?;
            if changed {
                 info!("About to write updated cohort {}", cohort_data.acronym());
                let updated_cohort_w = cohort_w.update_headers(updated_duplets);
                self.write_updated_cohort(&updated_cohort_w, hpo.clone());
                report.updated();
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