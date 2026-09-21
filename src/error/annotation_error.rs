use thiserror::Error;
use serde::Serialize;

use crate::dto::hpo_term_dto::HpoTermData;

#[derive(Debug, Clone, Error, Serialize)]
#[serde(tag = "code", content = "message")]
pub enum AnnotationError {
    #[error("Malformed annotation: '{0}'")]
    MalformedAnnotation(String),
    #[error("Unexpected disease count ({count}) for ppkt: {ppkt_id}")]
    UnexpectedDiseaseCount{count: usize, ppkt_id: String},
    #[error("Malformed disease entry: {0}")]
    MalformedDisease(String),
}


impl AnnotationError {
     pub fn misplaced_modifier(term_data: &HpoTermData) -> Self {
        let modifiers = term_data.modifiers().join(";");
        let msg = format!("{} ({}): {} - not allowed to have modifier ({}).", 
        term_data.label(), term_data.term_id(), term_data.entry(), modifiers);
        AnnotationError::MalformedAnnotation(msg)
    }

    pub fn unexpected_disease_count(disease_count: usize, ppkt_id: impl Into<String>) -> Self {
        AnnotationError::UnexpectedDiseaseCount{count: disease_count, ppkt_id: ppkt_id.into()}
    }

    pub fn malformed_disease_label(msg: impl Into<String>) -> Self {
        AnnotationError::MalformedDisease(msg.into())
    }

}