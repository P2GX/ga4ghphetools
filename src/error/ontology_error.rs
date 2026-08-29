use thiserror::Error;
use serde::Serialize;

use crate::{dto::hpo_term_dto::HpoTermDuplet, error::ontology_error::OntologyError::OntologyMatch};

#[derive(Debug, Clone, Error, Serialize)]
#[serde(tag = "code", content = "message")]
pub enum OntologyError {
    #[error("Could not create TermId from '{0}'")]
    TermIdCreation(String),
     #[error("Term not found for TermId '{0}'")]
    TermNotFound(String),
    #[error("{0}")]
    OntologyManipulationError(String),
    #[error("Could not find {0} in {1}")]
    MissingTid(String,String),
    #[error("Term {0} already present in Cohort")]
    RedundantTid(String),
    #[error("Hpo terms must match but we got {0} and {0}")]
    OntologyMatch(String, String),
    #[error("{0}")]
    TermDupletError(String)

    
}


impl OntologyError {
     pub fn term_id_creation(term_id: impl Into<String>) -> Self {
        OntologyError::TermIdCreation(term_id.into())
    }

    pub fn header_length_mismatch_err(previous: usize, current: usize) -> Self {
        let msg = format!("Length mismatch for update HPO row with new header: previous row HPOs: {} but header: {}",
            previous, current);
            OntologyError::OntologyManipulationError(msg)
    }

    pub fn manipulation_err(msg: impl Into<String>) -> Self {
        OntologyError::OntologyManipulationError(msg.into())
    }

    pub fn missing_tid(tid: impl Into<String>, location: impl  Into<String>) -> Self {
        OntologyError::MissingTid(tid.into(), location.into())
    }

    pub fn redundant_tid(tid: impl Into<String>) -> Self {
        OntologyError::RedundantTid(tid.into())
    }

    pub fn term_not_found(tid: impl Into<String>) -> Self {
        OntologyError::TermNotFound(tid.into())
    }

    pub fn ontology_match(duplet1: &HpoTermDuplet, duplet2: &HpoTermDuplet) -> Self {
        OntologyMatch(duplet1.hpo_label().to_string(), duplet2.hpo_label().to_string())
    }

    pub fn tid_mismatch(tid1: impl  Into<String>, tid2: impl  Into<String>) -> Self {
        OntologyMatch(tid1.into(), tid2.into())
    }

    pub fn label_mismatch(label1: impl  Into<String>, label2: impl  Into<String>) -> Self {
        OntologyMatch(label1.into(), label2.into())
    }

    pub fn term_duplet_conversion_error(duplet: &HpoTermDuplet) -> Self {
        let msg = format!("Failed to parse TermId from row2: {} (converting duplet: {:?})", duplet.hpo_id(), duplet); 
        OntologyError::TermDupletError(msg.into())
    }



}