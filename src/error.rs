//! Error
//! 
//! Functionality for error handling
//! Production code uses strict error handling
//! Test code can override Error and Result as follows
//! ```ignore
//! #[cfg(test)]
//! mod tests {
//!     type Error = Box<dyn std::error::Error>;
//!     type Result<T> = core::result::Result<T,Error>;
//!     use super::*;
//! 
//!     #[test]
//!     fn test_x() -> Result<()> {
//!         // -- Setup and fixtures
//!         // -- Exec 
//!         // -- Check
//!     }
//! 
//! }
//! let x = some_function();
//! println!("{}", x);
//! ```
//! In contrast, production code has errors like this
//! ```ignore
//! #[derive(Debug, From)]
//! pub enum Error {
//!   IndexOutOfBounds { actual: usize, max: usize},
//! 
//!   #[from]
//!   SerdeJson(serde_json::Error)
//! }
//! ``` 
//! 


use core::fmt;

use derive_more::{From, Display};
use serde::Serialize;

// can be used for test modules pub type Error = Box<dyn std::err::Err>;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, From, Serialize)]
pub enum Error {
    #[from]
    Custom(String),
    WhiteSpaceStart{ element: String },
    WhiteSpaceEnd{ element: String },
    WhiteSpaceError{ message: String},
    TranscriptError{ msg: String},
    LabelTooShort{ label: String, actual: usize, min: usize},
    EmptyLabel,
    ForbiddenLabelChar{ c: char, label: String},
    MalformedLabel{ label: String },
    MalformedDiseaseLabel{ label: String},
    TermIdError{ id: String },
    HpIdNotFound{ id: String },
    ObsoleteTermId{ id: String, replacement: String },
    WrongLabel{ id:String, actual: String, expected: String},
    EmptyField{field_name: String},
    CurieError{ msg: String},
    PmidError{msg: String},
    DiseaseIdError{msg: String },
    HgncError{msg: String},
    HgvsError{msg: String},
    HeaderError{ msg: String},
    UnrecognizedValue{value: String, column_name: String },
    TemplateError{ msg: String },
    TermError{msg: String},
    AgeParseError{msg: String},
    DeceasedError{value: String}
   
    // arrange according to module
    // -- pptcolumn


    /* -- Externals
    #[from]
    #[derive(serde::Serialize)]
    Io(std::io::Error), */
}

impl Error {
    pub fn custom(val: impl std::fmt::Display) -> Self {
        Self::Custom(val.to_string())
    }

    pub fn forbidden_character<T>(c: char, label: T) -> Self
        where T: Into<String> {
        Self::ForbiddenLabelChar { c, label: label.into() }
    }

    pub fn leading_ws<T>(value: T) -> Self
        where T: Into<String> {
        Self::WhiteSpaceError { message: format!("Leading whitespace in '{}'", value.into()) }
    }

    pub fn trailing_ws<T>(value: T) -> Self
        where T: Into<String> {
            Self::WhiteSpaceError { message:  format!("Trailing whitespace in '{}'", value.into()) }
    }

    pub fn short_label<T>(value: T, actual: usize, min: usize) -> Self 
        where T: Into<String> {
            Self::LabelTooShort { label:value.into(), actual, min }
    }

    pub fn lacks_transcript_version<T>(tx: T) -> Self
        where T: Into<String> {
            let msg =  format!("Transcript '{}' is missing a version", tx.into());
            Self::TranscriptError { msg: msg }
    }

    pub fn unrecognized_transcript_prefix<T>(tx: T) -> Self
        where T: Into<String> {
            let msg = format!("Unrecognized transcript prefix '{}'", tx.into());
            Self::TranscriptError { msg }
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
            Error::LabelTooShort { label, actual, min } => {
                write!(fmt, "Label '{}' is too short ({} < required {})", label, actual, min)
            },
            Error::TermIdError { id } => {
                write!(fmt, "Malformed TermId: {id}")
            },
            Error::HpIdNotFound { id } => {
                write!(fmt, "Not able to find HPO TermId: {id}")
            },
            Error::ObsoleteTermId { id , replacement } => {
                write!(fmt, "Obsolete HPO TermId: {id}; replace with {replacement}.")
            },
            Error::MalformedLabel { label } => {
                write!(fmt, "Malformed label: '{label}'")
            },
            Error::MalformedDiseaseLabel { label } => {
                write!(fmt, "Malformed disease label: '{label}'")
            },
            Error::ForbiddenLabelChar { c, label } => {
                write!(fmt, "Forbidden character '{c}' found in label '{label}'")
            }
            Error::WhiteSpaceError { message } => {
                write!(fmt, "{message}")
            },
            Error::EmptyLabel  => {
                write!(fmt, "Empty label")
            },
            Error::HgvsError { msg } => {
                write!(fmt, "{msg}")
            },
            Error::PmidError { msg } => {
                write!(fmt, "{msg}")
            },
            Error::CurieError { msg } => {
                write!(fmt, "{msg}")
            },
            Error::DiseaseIdError { msg } => {
                write!(fmt, "{msg}")
            },
            Error::TranscriptError { msg } => {
                write!(fmt, "{msg}")
            },
            Error::AgeParseError { msg } => {
                write!(fmt, "{msg}")
            },
            Error::EmptyField { field_name } => {
                write!(fmt, "{field_name} field is empty")
            },
            _ =>  write!(fmt, "{self:?}")
        }
    }
}

impl std::error::Error for Error {}