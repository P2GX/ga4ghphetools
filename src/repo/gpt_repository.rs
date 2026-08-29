//! GA$GH Phenotools Repository
//! This class is used to model a file-based repository with directories and files created by this software

use std::{fs::File, io::Write, path::PathBuf, sync::Arc};

use ontolius::ontology::csr::FullCsrOntology;
use walkdir::WalkDir;

use crate::{dto::cohort_dto::CohortData, error::{PheToolsError, cohort_error::CohortError, ontology_error::OntologyError}, hpo::{self, update_hpo_duplets}, repo::{self, cohort_dir::{self, CohortDir}, cohort_qc::CohortQc, qc_report::UpdateReport, repo_qc::RepoQc}};


pub struct GptRepository {
     /// Path of the directory corresponding to cohort_name (e.g., a gene smbol such as ZRSR2)
    pub path: PathBuf,
    cohort_list: Vec<CohortDir>
}




impl GptRepository {
    pub fn new(root_path: &PathBuf) -> Self {
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


    /// Save a single CohortData at its original path
    /// Intended to be used if we have edited the CohortData, e.g., updating the HPO ids/labels
     pub fn save_template_json(&self, cohort: CohortData, path: &PathBuf) -> Result<(), CohortError> { 
        let template_name = cohort.acronym();
        let fname = format!("{}_individuals.json", template_name);
        let save_path = path.join(template_name);
        let json = serde_json::to_string_pretty(&cohort).map_err(|e| CohortError::io_error(path, &e.to_string()))?;
        let mut file = File::create(&save_path).map_err(|e|CohortError::io_error(path, &e.to_string()))?;
        file.write_all(json.as_bytes()).map_err(|e|CohortError::io_error(path, &e.to_string()))?;
        Ok(())
    }

     
    pub fn update_all_ppkt(&self, hpo: Arc<FullCsrOntology>) -> Result<UpdateReport, PheToolsError> {
        let mut report = UpdateReport::new(&self.path);
        for cdir in &self.cohort_list {
            let cohort_path = &cdir.path;
            let cohort_list = cdir.get_cohort_data()?;
            for mut cohort in cohort_list {
                let needs_update = hpo::duplets_need_update(hpo.clone(), &cohort.hpo_headers)?;
                if needs_update {
                     let updated = update_hpo_duplets(hpo.clone(), &cohort.hpo_headers)?;
                     cohort.hpo_headers = updated;
                     self.save_template_json(cohort, cohort_path)?;
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
    use rstest::{fixture, rstest};

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
         //println!("{}", relative_path);
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



}