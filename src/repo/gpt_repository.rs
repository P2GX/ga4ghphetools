//! GA4GH Phenotools Repository
//! This class is used to model a file-based repository with directories and files created by this software

use std::{fs::File, io::Write, path::{Path, PathBuf}, sync::Arc};

use ontolius::ontology::{MetadataAware, csr::FullCsrOntology};
use phenopackets::schema::v1::interpretation::PhenopacketOrFamily::Phenopacket;
use crate::ppkt::ppkt_updater::PpktUpdater;
use walkdir::WalkDir;

use crate::{dto::cohort_dto::CohortData, error::{PheToolsError, cohort_error::CohortError}, hpo::{self, update_hpo_duplets}, repo::{cohort_dir::CohortDir, cohort_qc::CohortQc, qc_report::UpdateReport, repo_qc::RepoQc}};

/// A structure representing the entire phenopacket store (GA4GHPhenoTools) repository
pub struct GptRepository {
     /// Path of the overarching root repository containing multiple gene folders with cohorts
    pub path: PathBuf,
    /// List of all Cohort directories (one per gene in phenopacket store)
    cohort_list: Vec<CohortDir>
}




impl GptRepository {
    pub fn new(root_path: &Path) -> Self {
        let mut all_cohorts: Vec<CohortDir> = Vec::new();
        let entries = WalkDir::new(root_path)
            .min_depth(1)
            .max_depth(1)
            .into_iter()
            .filter_map(|e| e.ok());

        for entry in entries {
            if entry.file_type().is_dir() {
                let gene_data = CohortDir::process_gene_directory(entry.path());
                all_cohorts.push(gene_data);
            }
        }
    
        println!("Processed {} gene directories.", all_cohorts.len());
        Self {
            path: root_path.into(),
            cohort_list: all_cohorts
        }
    }

    pub fn repo_qc(&self) -> Result<RepoQc, String> {
         let result: Result<Vec<CohortQc>, String> = self.cohort_list.iter()
            .map(|cl| cl.get_cohort_qc())
            .collect();
        match result {
            Ok(cohort_qc_list) => Ok(RepoQc::new(&self.path, cohort_qc_list)),
            Err(e) => Err(e),
        }
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
        let mut report = UpdateReport::new(&self.path);
        let hpo_version = hpo.version();
        for cdir in &self.cohort_list {
            for cohort_path in cdir.get_individuals_json_files() {
                let mut cohort = CohortDir::read_cohort(cohort_path)?;
                if hpo::duplets_need_update(hpo.clone(), &cohort.hpo_headers).map_err(|e| {
                        PheToolsError::from(format!("Failed checking HPO duplets for {:?}: {}", cohort_path, e))
                    })? {
                    cohort.hpo_headers = update_hpo_duplets(hpo.clone(), &cohort.hpo_headers)?;
                    self.save_template_json(&cohort, cohort_path)?;
                    println!("[INFO] Updated cohort at {:?}.", &cohort_path);
                    let ppkt_map = cdir.get_ppkt_map()?;
                    for (pbuf, ppkt) in ppkt_map.iter() {
                        let ppkt_updater = PpktUpdater::from_existing(hpo.clone(), &cohort, &ppkt)?;
                        let updated_ppkt = ppkt_updater.get_ppkt();
                        crate::ppkt::write_ppkt(&updated_ppkt, &pbuf)?;
                        println!("[INFO]\twrote updated phenopacket to {}", &pbuf.to_string_lossy().into_owned());
                    }
                    report.updated();
                } else {
                    report.processed();
                }
            }
        }
        Ok(report)
    }

}

#[cfg(test)]
mod tests {
    use ontolius::{TermId, ontology::OntologyTerms};
use rstest::{fixture, rstest};
    use crate::test_utils::fixtures::hpo;
    use super::*;

    #[fixture]
    fn repo_path() -> String {
        // 1. Start from the project root (where Cargo.toml lives)
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        
        // 2. Join the relative path components
        let relative_path = std::path::Path::new(manifest_dir)
            .join("..")
            .join("phenopacket-store")
            .join("notebooks");
        // 3. Convert to absolute path and resolve ".."
        // Note: canonicalize returns an error if the path doesn't exist
        let absolute_path = std::fs::canonicalize(relative_path)
            .expect("The path to phenopacket-store/notebooks does not exist");

        // 4. Return as String
        absolute_path.to_string_lossy().to_string()
    }

    #[rstest]
    fn test_all(repo_path: String) {
        let root_path: PathBuf = repo_path.into();
        let gptr = GptRepository::new(&root_path);
        let rqc = gptr.repo_qc().unwrap();
        for e in &rqc.errors {
            println!("{:?}", e);
        }
    } 

     #[rstest]
     fn test_update(hpo: Arc<FullCsrOntology>) {
        let root = "/Users/peterrobinson/TMP/notebooks";
        let pbuf: PathBuf = root.into();
        let gptr = GptRepository::new(&pbuf);
        let update_report = gptr.update_all_ppkt(hpo).unwrap();
        println!("Updated: {}", update_report.directory);
         println!("Processed {} cohorts, of which {} were updated", update_report.processed, update_report.updated);
        assert!(true);
     }

     #[rstest]
     fn test_failing_cohort(hpo: Arc<FullCsrOntology>) {
        let cohort_file = "/Users/peterrobinson/TMP/notebooks/APOA4/APOA4_ADTKD6_individuals.json";
        let pbuf: PathBuf = cohort_file.into();
        let mut cohort = CohortDir::read_cohort(&pbuf).unwrap();
        if crate::hpo::duplets_need_update(hpo.clone(), &cohort.hpo_headers).unwrap() {
            cohort.hpo_headers = update_hpo_duplets(hpo.clone(), &cohort.hpo_headers).unwrap();
            println!("Updated headers")
        } else {
            println!("No need to update headers")
        }
     }

       #[rstest]
     fn test_term_get(hpo: Arc<FullCsrOntology>) {
        let tid: TermId = "HP:0020109".parse().unwrap();
        let term = hpo.term_by_id(&tid);
        assert!(term.is_some());
        
     }



}