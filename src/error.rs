//! Error
//!
//! Functionality for error handling. The Error enum offers specific Error types and convenience
//! functions for creating Error instances. API-facing funtions transform these errors into Strings.
//!

use core::fmt;
use derive_more::From;
use ontolius::TermId;
use serde::Serialize;

use crate::dto::validation_errors::ValidationErrors;



pub type Result<T> = core::result::Result<T, Error>;


#[derive(Debug, From, Serialize)]
pub enum Error {
    #[from]
    Custom(String),
    ForbiddenLabelChar {
        c: char,
        label: String,
    },
    MalformedLabel {
        label: String,
    },
    MalformedDiseaseLabel {
        label: String,
    },
    TimeElementError {
        msg: String
    },
    VariantError {
        msg: String,
    },
    EditError {
        msg: String,
    },
   
}

impl Error {
    pub fn custom(val: impl std::fmt::Display) -> Self {
        Self::Custom(val.to_string())
    }


}

impl From<&str> for Error {
    fn from(val: &str) -> Self {
        Self::Custom(val.to_string())
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> fmt::Result {
        match self {
            Error::MalformedLabel { label } => {
                write!(fmt, "Malformed label: '{label}'")
            },
            Error::MalformedDiseaseLabel { label } => {
                write!(fmt, "Malformed disease label: '{label}'")
            },
            Error::ForbiddenLabelChar { c, label } => {
                write!(fmt, "Forbidden character '{c}' found in label '{label}'")
            },
            Error::EditError { msg }
    
            | Error::TimeElementError { msg }
            | Error::VariantError { msg } => {
                write!(fmt, "{msg}")
            },
            | _ => write!(fmt, "{self:?}"),
        }
    }
}

impl std::error::Error for Error {}
