use std::collections::HashSet;

use serde::de::Unexpected;



#[derive(Debug, Clone, serde::Serialize)] 
#[serde(rename_all = "camelCase")]
enum RepoErrorType {
    UnexpectedFile,
    MoiMismatch,
    PpktExportError,
    NoHpoTermError
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
        let set = format!(
            "{{{}}}",
            allowable_allele_counts
                .iter()
                .map(|n| n.to_string())
                .collect::<Vec<_>>()
                .join(",")
        );
        let message= format!("Expected counts of {} but got {} for {}.", set,ac, ppkt_id);
        Self { cohort_name: cohort_name.to_string(), 
            message, 
            error_type: RepoErrorType::MoiMismatch 
        }
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