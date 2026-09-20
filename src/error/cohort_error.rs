use std::{collections::HashSet, path::Path};

use ontolius::TermId;
use thiserror::Error;
use serde::Serialize;



#[derive(Debug, Clone, Error, Serialize)]
#[serde(tag = "code", content = "message")]
pub enum CohortError {
    #[error("Cannot save null cohort")]
    NotInitialized,
    #[error("Need to specify acronym before saving")]
    MissingAcronym,
    #[error("No mode of inheritance specified for {0}")]
    LackingMoi(String),
    #[error("Unrecognize mode of inheritance: '{0}'")]
    InvalidMoi(String),
    #[error("Expected counts of {expected} but got {actual} for {ppkt_id}.")]
    MoiMismatch { ppkt_id: String, expected: String, actual: usize },
    #[error("Lacking disease id for {0}")]
    LackingDiseaseId(String),
    #[error("Could not find column that corresponds to {0}")]
    ColumnNotFound(String),
    #[error("Could not find disease in cohort")]
    NoDisease,
    #[error("No phenopackets found for {0}")]
    NoPpkt(String),
    #[error("Phenopacket {0} had no observed HPO terms")]
    NoHpos(String),
    #[error("Could not retrieve CellValue for '{0}'")]
    MissingCellValue(String),
    #[error("{0}")]
    CohortIoError(String),
    #[error("Redundant annotations found: {count}")]
    RedundantAnnotations { count: usize },
    #[error("Format error: {message}")]
    FormatErr { message: String },
    #[error("Unexpected file: {file}")]
    UnexpectedFile { file: String },
    #[error("Rows: {n_rows} - exported phenopackets: {n_phenopackets}")]
    PpktExportError { n_rows: usize, n_phenopackets: usize },
    #[error("Phenopacket {ppkt_id} had no observed HPO terms")]
    NoHpoTermError { ppkt_id: String },
    #[error("Malformed acronym: '{acronym}' - expected GENE_DISEASE")]
    AcronymError { acronym: String },
    #[error("Non-Mendelian Q/C analysis not implemented {0}")]
    NonMendelianWarning(String),
    #[error("Rows: {n_rows} - exported phenopackets: {n_phenopackets}")]
    CountMismatch { n_rows: usize, n_phenopackets: usize },
    #[error("{0}")]
    OntologyError(String),
    #[error("{0}")]
    DuplicateEntry(String),
    #[error("{0}")]
    MissingField(String),
    #[error("{0}")]
    Curation(String)
}

impl CohortError {
    pub fn empty_disease_list() -> Self {
        CohortError::NoDisease
    }

    pub fn missing_column(tid: &TermId) -> Self {
        CohortError::ColumnNotFound(tid.to_string())
    }

    /// Should never happen, but we need this since we retrieve the CellValue from a map
    pub fn missing_cell_value(tid: &TermId) -> Self {
        CohortError::MissingCellValue(tid.to_string())
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

    pub fn unexpected_file(unexpected_file_name: &str) -> Self {
        CohortError::UnexpectedFile { file: unexpected_file_name.to_string() }
    }

    pub fn no_disease_id(ppkt_id: &str) -> Self {
        CohortError::LackingDiseaseId(ppkt_id.to_string())
    }

    pub fn no_ppkt_found(disease_id: &str) -> Self {
        CohortError::NoPpkt(disease_id.to_string())
    }

    pub fn invalid_moi(moi: &str) -> Self {
        CohortError::InvalidMoi(moi.to_string())
    }

     pub fn moi_mismatch(ppkt_id: &str, allowable_allele_counts: &HashSet<usize>, ac: usize) -> Self {
        let mut counts: Vec<_> = allowable_allele_counts.iter().collect();
        counts.sort_unstable();
        let set_str = match counts.as_slice() {
            [single] => single.to_string(),
            multiple => {
                let joined = multiple
                    .iter()
                    .map(|n| n.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{{{}}}", joined)
            }
        };
        CohortError::MoiMismatch{expected: set_str, actual: ac, ppkt_id: ppkt_id.to_string()}
    }

    pub fn non_mendelian(cohort_acronym: &str) -> Self {
        CohortError::NonMendelianWarning(cohort_acronym.to_string())
    }

    pub fn malformed_acronym(cohort_acronym: &str) -> Self {
        CohortError::AcronymError { acronym: cohort_acronym.to_string() }
    }

    pub fn count_mismatch(n_nrows: usize, n_phenopackets: usize) -> Self {
        CohortError::CountMismatch { n_rows: n_nrows, n_phenopackets }
    }

    pub fn no_observed_hpo_annots(ppkt_id: impl Into<String>) -> Self {
        CohortError::NoHpos(ppkt_id.into())
    }

    pub fn ontology_error(message: impl Into<String>) -> Self {
        CohortError::OntologyError(message.into())
    }

    pub fn duplicate_entry(message: impl Into<String>) -> Self {
        CohortError::DuplicateEntry(message.into())
    }

    pub fn missing_field(message: impl Into<String>) -> Self {
        CohortError::MissingField(message.into())
    }

    pub fn curation_error(message: impl Into<String>) -> Self {
        CohortError::Curation(message.into())
    }
}