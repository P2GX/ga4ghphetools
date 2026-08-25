use serde::Serialize;
use thiserror::Error;

use crate::error::{annotation_error::AnnotationError, cohort_error::CohortError, ontology_error::OntologyError, parse_error::ParseError};

pub mod annotation_error;
pub mod cohort_error;
pub mod ontology_error;
pub mod parse_error;



#[derive(Debug, Clone, Error, Serialize)]
#[serde(tag = "domain", content = "error")]
pub enum PheToolsError {
    #[error(transparent)]
    AnnotationError(#[from] AnnotationError),
    #[error(transparent)]
    Cohort(#[from] CohortError),
    #[error(transparent)]
    Ontology(#[from] OntologyError),
    #[error(transparent)]
    Parse(#[from] ParseError), 
}