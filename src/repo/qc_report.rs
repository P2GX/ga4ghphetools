//! QcReport
//! Structure to record various kinds of Q/C issues to report back to the user

use std::{collections::HashSet, path::PathBuf};


#[derive(Debug, Clone, serde::Serialize)] 
#[serde(rename_all = "camelCase")]
pub enum RepoErrorType {
    UnexpectedFile,
    MoiMismatch,
    PpktExportError,
    NoHpoTermError,
    AcronymError
}


#[derive(Debug, Clone, serde::Serialize)] 
#[serde(rename_all = "camelCase")]
pub struct QcReport {
    pub cohort_name: String,
    pub message: String,
    pub error_type: RepoErrorType,
}


impl QcReport {

    pub fn unexpected_file(cohort_name: &str, unexpected: &str) -> Self {
        let msg = format!("Unexpected file: {}", unexpected);
        Self { cohort_name: cohort_name.to_string(), 
            message: msg, 
            error_type: RepoErrorType::UnexpectedFile 
        }
    }

    pub fn moi_mismatch(cohort_name: &str, ppkt_id: &str, allowable_allele_counts: &HashSet<usize>, ac: usize) -> Self {
        let mut counts: Vec<_> = allowable_allele_counts.iter().collect();
        counts.sort_unstable();
        let set_str = match counts.as_slice() {
            [single] => single.to_string(),
            multiple => {
                let joined = multiple
                    .iter()
                    .map(|n| n.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{{{}}}", joined)
            }
        };
        let message= format!("Expected counts of {} but got {} for {}.", set_str, ac, ppkt_id);
        Self { cohort_name: cohort_name.to_string(), 
            message, 
            error_type: RepoErrorType::MoiMismatch 
        }
    }

    pub fn malformed_acronym(acronym: String) -> Self {
        let message = format!("Malformed acronym: '{}' - expected GENE_DISEASE", acronym);
        Self { cohort_name: acronym, message, error_type: RepoErrorType::AcronymError }
    }

    pub fn count_mismatch(cohort_name: &str, n_nrows: usize, n_phenopackets: usize) -> Self {
        let message = format!("Rows: {} - exported phenopackets: {}", n_nrows, n_phenopackets);
        Self {
            cohort_name: cohort_name.to_string(),
            message,
            error_type: RepoErrorType::PpktExportError,
        }
    }

    pub fn no_hpo(cohort_name: &str, ppkt_id: &str) -> Self {
        let message = format!("Phenopacket {} had no observed HPO terms", ppkt_id);
        Self {
            cohort_name: cohort_name.to_string(),
            message,
            error_type: RepoErrorType::NoHpoTermError
        }
    }

}


pub struct UpdateReport {
    pub directory: String,
    pub processed: usize,
    pub updated: usize,
}

impl UpdateReport {
    pub fn new(path: &PathBuf) -> Self {
        Self {
            directory: path.to_string_lossy().to_string(),
            updated: 0,
            processed: 0
        }
    }

    pub fn updated(&mut self) {
        self.updated += 1;
        self.processed();
    }

    pub fn processed(&mut self) {
        self.processed += 1;
    }
}