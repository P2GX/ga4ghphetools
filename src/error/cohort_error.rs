use std::path::Path;

use ontolius::TermId;
use thiserror::Error;
use serde::Serialize;

use crate::error::cohort_error::CohortError::{MissingCellValue, NoDisease};

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
    #[error("Could not retrieve CellValue for '{0}'")]
    MissingCellValue(String),
    #[error("{0}")]
    CohortIoError(String),
}

impl CohortError {
    pub fn empty_disease_list() -> Self {
        NoDisease("Empty disease list".to_string())
    }

    pub fn missing_column(tid: &TermId) -> Self {
        CohortError::ColumnNotFound(tid.to_string())
    }

    /// Should never happen, but we need this since we retrieve the CellValue from a map
    pub fn missing_cell_value(tid: &TermId) -> Self {
        MissingCellValue(tid.to_string())
    }

    pub fn io_error(path: &Path, file_error: &str) -> Self {
        let msg =  format!("Could not extract CohortData string from {}: {}", path.to_string_lossy(), file_error);
        CohortError::CohortIoError(msg)
    }

    pub fn json_error(cohort_data: &str, json_error: &str) -> Self {
        let msg =  format!("Could not transform string {} to CohortDto: {}",
                cohort_data, json_error);
        CohortError::CohortIoError(msg)
    }


  
}