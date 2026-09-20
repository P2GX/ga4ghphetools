use std::sync::Arc;

use ontolius::ontology::csr::FullCsrOntology;

use crate::{cohort_qc::cohort_qc::CohortDataQc, dto::cohort_dto::CohortData, error::cohort_error::CohortError};


pub(crate) mod cohort_dir;
pub mod qc_report;
pub(crate) mod cohort_qc;
mod disease_qc;





/// Perform basic Q/C operations
/// This function is used while we are creating and saving a Cohort
/// TODO rename to check_cohort_errors
pub fn validate_cohort_template(
    hpo: Arc<FullCsrOntology>,
    cohort_dto: &CohortData)
-> Result<(), CohortError> {
    let cohort_qc = CohortDataQc::new(hpo);
    cohort_qc.qc_check(cohort_dto)?;
    cohort_qc.check_metadata(cohort_dto)?;
    cohort_qc.qc_conflicting_pairs(cohort_dto)
}





