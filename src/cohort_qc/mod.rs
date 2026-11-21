use std::sync::Arc;

use ontolius::ontology::csr::FullCsrOntology;

use crate::{cohort_qc::cohort_qc::CohortDataQc, dto::cohort_data::CohortData, error::cohort_error::CohortError};


pub(crate) mod cohort_dir;
pub mod qc_report;
pub(crate) mod cohort_qc;
pub(crate) mod disease_qc;
pub(crate) mod field_checks;




/// Perform basic Q/C operations
/// This function is used while we are creating and saving a Cohort
/// The Phenoboard user clicks on Check & Fix and sees a list of errors with the chance to correct them.
pub fn validate_cohort_template(
    hpo: Arc<FullCsrOntology>,
    cohort_dto: &CohortData)
-> Result<(), CohortError> {
    let cohort_qc = CohortDataQc::new(hpo);
    let report = cohort_qc.qc_check(cohort_dto)?;
    if !report.issues.is_empty() {
        let msg = report.issues.iter()
            .map(|i| i.message.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        return Err(CohortError::ontology_error(msg));
    }
    cohort_qc.check_metadata(cohort_dto)?;
    cohort_qc.qc_conflicting_pairs(cohort_dto)
}





