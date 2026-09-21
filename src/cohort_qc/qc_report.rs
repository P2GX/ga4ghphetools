//! QcReport
//! Structure to record various kinds of Q/C issues to report back to the user

use std:: path::{Path, PathBuf};

use crate::{dto::cohort_dto::CohortData, error::{PheToolsError, cohort_error::CohortError, ontology_error::OntologyError}};

/// Designed for easy manipulation in front end.
/// If we ever need it, we can expand to use the various types of the various errors
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QcIssue {
    /// Coarse category — "Cohort", "Ontology", "Parse", "Annotation", "Message".
    /// Intentionally not the full structured error; see PheToolsError if/when
    /// the frontend needs to branch on specific error codes.
    pub domain: String,
    /// Human-readable message, from the error's own Display impl.
    pub message: String,
}

impl From<&PheToolsError> for QcIssue {
    fn from(err: &PheToolsError) -> Self {
        let domain = match err {
            PheToolsError::AnnotationError(_) => "Annotation",
            PheToolsError::Cohort(_) => "Cohort",
            PheToolsError::Ontology(_) => "Ontology",
            PheToolsError::Parse(_) => "Parse",
            PheToolsError::Message(_) => "Message",
        };
        QcIssue { domain: domain.to_string(), message: err.to_string() }
    }
}


#[derive(Debug, Clone, serde::Serialize)] 
#[serde(rename_all = "camelCase")]
pub struct QcReport {
    pub cohort_name: String,
     #[serde(flatten)]
    pub issues: Vec<QcIssue>,
}

impl From<&CohortError> for QcIssue {
    fn from(err: &CohortError) -> Self {
        QcIssue { domain: "Cohort".to_string(), message: err.to_string() }
    }
}

impl From<&OntologyError> for QcIssue {
    fn from(err: &OntologyError) -> Self {
        QcIssue { domain: "Ontology".to_string(), message: err.to_string() }
    }
}


impl QcReport {


    pub fn new(cohort_name: &str) -> Self {
            Self {
                cohort_name: cohort_name.to_string(), 
                issues: Vec::default() 
            }
        }


    pub fn unexpected_file(&mut self, unexpected: &str) {
        self.issues.push(QcIssue::from(&CohortError::unexpected_file(unexpected)));
    }

    pub fn no_disease_id(&mut self, ppkt_id: &str) {
        self.issues.push(QcIssue::from(&CohortError::no_disease_id(ppkt_id)));
    }

    pub fn no_disease(&mut self) {
        self.issues.push(QcIssue::from(&CohortError::empty_disease_list()));
    }

    pub fn non_mendelian(&mut self, cohort: &CohortData) {
        self.issues.push(QcIssue::from(&CohortError::non_mendelian(&cohort.acronym())));
    }

    pub fn no_ppkt_found(&mut self, disease_id: &str) {
        self.issues.push(QcIssue::from(&CohortError::no_ppkt_found(disease_id)));
    }

    /// Extend the list of issues with a (potentially empty) list of CohortErrors.
    pub fn extend_cohort_errors(&mut self, errors: impl IntoIterator<Item = CohortError>) {
        self.issues.extend(errors.into_iter().map(|e| QcIssue::from(&e)));
    }

    /// Extend the list of issues with a (potentially empty) list of OntologyErrors.
    pub fn extend_ontology_errors(&mut self, errors: impl IntoIterator<Item = OntologyError>) {
        self.issues.extend(errors.into_iter().map(|e| QcIssue::from(&e)));
    }

    /// The HPO version this cohort's term ids/labels were checked against needs updating.
    pub fn hpo_needs_version_update(&mut self) {
        self.issues.push(QcIssue {
            domain: "Ontology".to_string(),
            message: "HPO term id/label needs update".to_string(),
        });
    }

    /// Records an ontology-domain issue, preserving its own domain rather than
    /// re-wrapping it as a generic Cohort error.
    pub fn ontology_error(&mut self, error: &OntologyError) {
        self.issues.push(QcIssue::from(error));
    }

}

