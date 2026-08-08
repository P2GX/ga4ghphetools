use ontolius::TermId;
use thiserror::Error;
use serde::Serialize;

use crate::error::cohort_error::CohortError::NoDisease;

#[derive(Debug, Clone, Error, Serialize)]
#[serde(tag = "code", content = "message")]
pub enum CohortError {
    #[error("Cannot save null cohort")]
    NotInitialized,
    #[error("Need to specify acronym before saving")]
    MissingAcronym,
    #[error("No mode of inheritance specified for {0}")]
    MissingMoi(String),
    #[error("Could not find column that corresponds to {0}")]
    ColumnNotFound(String),
    #[error("{0}")]
    NoDisease(String),
}

impl CohortError {
    pub fn empty_disease_list() -> Self {
        NoDisease("Empty disease list".to_string())
    }

    pub fn missing_column(tid: &TermId) -> Self {
        CohortError::ColumnNotFound(tid.to_string())
    }
}