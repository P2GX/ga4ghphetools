use std::path::{Path, PathBuf};

use crate::cohort_qc::qc_report::QcReport;





#[derive(Debug, Clone, serde::Serialize)] 
#[serde(rename_all = "camelCase")]
pub struct RepoQc {
    pub repo_path: PathBuf,
    pub cohort_count: usize,
    pub errors: Vec<QcReport>
}


impl RepoQc {
    pub fn new(repository_path: &Path, cohort_qc_list: Vec<QcReport>) -> Self {
        Self {
            repo_path: repository_path.to_path_buf(),
            cohort_count: cohort_qc_list.len(),
            errors: cohort_qc_list
        }
    }

}