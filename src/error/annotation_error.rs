use thiserror::Error;
use serde::Serialize;

use crate::dto::hpo_term_dto::HpoTermData;

#[derive(Debug, Clone, Error, Serialize)]
#[serde(tag = "code", content = "message")]
pub enum AnnotationError {
    #[error("Malformed annotation: '{0}'")]
    MalformedAnnotation(String),
}


impl AnnotationError {
     pub fn misplaced_modifier(term_data: &HpoTermData) -> Self {
        let modifiers = term_data.modifiers().join(";");
        let msg = format!("{} ({}): {} - not allowed to have modifier ({}).", 
        term_data.label(), term_data.term_id(), term_data.entry(), modifiers);
        AnnotationError::MalformedAnnotation(msg)
    }

}