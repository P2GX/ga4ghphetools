//! Error
//!
//! Functionality for error handling. The Error enum offers specific Error types and convenience
//! functions for creating Error instances. API-facing funtions transform these errors into Strings.
//!


use derive_more::From;

use serde::Serialize;


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



