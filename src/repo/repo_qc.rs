use std::path::PathBuf;

use crate::cohort_qc::qc_report::QcReport;





#[derive(Debug, Clone, serde::Serialize)] 
#[serde(rename_all = "camelCase")]
pub struct RepoQc {
    pub repo_path: String,
    pub cohort_count: usize,
    pub errors: Vec<QcReport>
}


impl RepoQc {
    pub fn new(repository_path: &PathBuf, cohort_qc_list: Vec<QcReport>) -> Self {
        let cohort_count = cohort_qc_list.len();
        let repo_path: String = repository_path.to_string_lossy().to_string();
        Self {
            repo_path,
            cohort_count,
            errors: cohort_qc_list
        }
    }

}