use std::collections::HashSet;



#[derive(Debug, Clone, serde::Serialize)] // Serialize helps if passing to a web-based GUI
pub struct QcReport {
    pub cohort_name: String,
    pub message: String,
    pub is_ok: bool,
}




impl QcReport {
    

    pub fn unexpected_file(cohort_name: &str, unexpected: &str) -> Self {
        let msg = format!("Unexpected file: {}", unexpected);
        Self { cohort_name: cohort_name.to_string(), 
            message: msg, 
            is_ok: false 
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
            is_ok: false 
        }
    }

    pub fn count_mismatch(cohort_name: &str, n_nrows: usize, n_phenopackets: usize) -> Self {
        let message = format!("Rows: {} - exported phenopackets: {}", n_nrows, n_phenopackets);
        Self {
            cohort_name: cohort_name.to_string(),
            message,
            is_ok: false,
        }
    }


   
}