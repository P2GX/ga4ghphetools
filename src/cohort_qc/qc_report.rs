//! QcReport
//! Structure to record various kinds of Q/C issues to report back to the user

use std:: path::{Path, PathBuf};

use crate::{dto::cohort_dto::CohortData, error::{PheToolsError, cohort_error::CohortError, ontology_error::OntologyError}};



#[derive(Debug, Clone, serde::Serialize)] 
#[serde(rename_all = "camelCase")]
pub struct QcReport {
    pub cohort_name: String,
     #[serde(flatten)]
    pub issues: Vec<PheToolsError>,
}


impl QcReport {


    pub fn new(cohort_name: &str) -> Self {
            Self {
                cohort_name: cohort_name.to_string(), 
                issues: Vec::default() 
            }
        }


    pub fn unexpected_file(&mut self, unexpected: &str)  {
        let c_error = CohortError::unexpected_file(unexpected);
        self.issues.push(c_error.into());
    }

    pub fn no_disease_id(&mut self, ppkt_id: &str) {
        let c_error = CohortError::no_disease_id(&ppkt_id.to_string());
        self.issues.push(c_error.into());
    }

    pub fn no_disease(&mut self) {
        let c_error = CohortError::empty_disease_list();
        self.issues.push(c_error.into());
    }

    pub fn non_mendelian(&mut self, cohort: &CohortData) {
        let c_error = CohortError::non_mendelian(&cohort.acronym());
        self.issues.push(c_error.into());
    }


    pub fn no_ppkt_found(&mut self, disease_id: &str) {
        let c_error = CohortError::no_ppkt_found(disease_id);
        self.issues.push(c_error.into());
    }

    pub fn hpo_needs_version_update(&mut self) {
        let c_error = CohortError::ontology_error("HPO term id/label needs update");
        self.issues.push(c_error.into());
    }

    /// Extend the list of issues with a (potentially empty) list or an Option of CohortError(s)
    pub fn extend_cohort_errors(&mut self, errors: impl IntoIterator<Item = CohortError>) {
        self.issues.extend(errors.into_iter().map(PheToolsError::from));
    }

    pub fn extend_ontology_errors(&mut self, errors: impl IntoIterator<Item = OntologyError>) {
        self.issues.extend(errors.into_iter().map(PheToolsError::from));
    }

    pub fn ontology_error(&mut self, error: &OntologyError) {
        let msg = error.to_string();
        let c_error = CohortError::ontology_error(&msg);
        self.issues.push(c_error.into());
    }

}

