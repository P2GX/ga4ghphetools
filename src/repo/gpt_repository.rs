//! GA$GH Phenotools Repository
//! This class is used to model a file-based repository with directories and files created by this software

use std::path::PathBuf;

use walkdir::WalkDir;

use crate::repo::{cohort_dir::CohortDir, cohort_qc::CohortQc, repo_qc::RepoQc};


pub struct GptRepository {
     /// Path of the directory corresponding to cohort_name (e.g., a gene smbol such as ZRSR2)
    pub path: PathBuf,
    cohort_list: Vec<CohortDir>
}


impl GptRepository {
    fn new(root_path: &str) -> Self {
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
            Ok(cohort_qc_list) => Ok(RepoQc::new(cohort_qc_list)),
            Err(e) => Err(e),
        }
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
    fn test_repo(repo_path: String) {
        let repo = GptRepository::new(&repo_path);
        let repoqc = repo.repo_qc().unwrap();
        let count = repoqc.phenopacket_count();
        println!("Total phenopackets: {}", count);
        repoqc.check_moi();
    }



}