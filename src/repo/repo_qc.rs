use std::path::PathBuf;

use crate::repo::{cohort_qc::CohortQc, qc_report::QcReport};



#[derive(Debug, Clone, serde::Serialize)] 
#[serde(rename_all = "camelCase")]
pub struct RepoQc {
    pub repo_path: String,
    pub cohort_count: usize,
    pub phenopacket_count: usize,
    pub errors: Vec<QcReport>
}


impl RepoQc {
    pub fn new(repository_path: &PathBuf, cohort_qc_list: Vec<CohortQc>) -> Self {
        let phenopacket_count = Self::phenopacket_count(&cohort_qc_list);
        let cohort_count = cohort_qc_list.len();
        let repo_path: String = repository_path.to_string_lossy().to_string();
        let errors = Self::get_errors(cohort_qc_list);
        Self {
            repo_path,
            cohort_count,
            phenopacket_count,
            errors
        }
    }

    pub fn phenopacket_count(cohort_qc_list: &Vec<CohortQc>) -> usize {
        let mut c = 0 as usize;
        for cohort in cohort_qc_list {
            c += cohort.ppkt_count();
        }
        return c;
    }

    fn get_errors(cohort_qc_list: Vec<CohortQc>) -> Vec<QcReport> {
        let mut errs: Vec<QcReport> = Vec::new();
        for cohort in cohort_qc_list {
            let cohort_errs = cohort.get_errors();
            errs.extend(cohort_errs);
        }

        errs
    }

    

}