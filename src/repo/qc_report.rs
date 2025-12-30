

#[derive(Debug, Clone, serde::Serialize)] // Serialize helps if passing to a web-based GUI
pub struct QcReport {
    pub cohort_name: String,
    pub message: String,
    pub is_ok: bool,
}




impl QcReport {
    pub fn no_unepected_files(cohort_name: &str) -> Self {
        Self { 
            cohort_name: cohort_name.to_string(), 
            message: "No unexpected files".to_string(), 
            is_ok: true }
    }

    pub fn unexpected_files(cohort_name: &str, unexpected: &Vec<String>) -> Self {
        let msg = format!("Unexpected files: {}", unexpected.join("; "));
        Self { cohort_name: cohort_name.to_string(), 
            message: msg, 
            is_ok: false 
        }
    }


   
}