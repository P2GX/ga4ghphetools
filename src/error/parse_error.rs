use thiserror::Error;
use serde::Serialize;

#[derive(Debug, Clone, Error, Serialize)]
#[serde(tag = "code", content = "message")]
pub enum ParseError {
    #[error("Malformed HPO cell contents: '{0}'")]
    MalformedCellValue(String),
}

impl ParseError {
    pub fn malformed_cell_value(raw: impl Into<String>) -> Self {
        ParseError::MalformedCellValue(raw.into())
    }
}