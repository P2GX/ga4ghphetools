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
    TranscriptWithoutVersion{ transcript: String },
    LabelTooShort{ label: String, actual: usize, min: usize},
    TermIdError{ id: String },
    HpIdNotFound{ id: String },

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
}

impl From<&str> for Error {
    fn from(val: &str) -> Self {
        Self::Custom(val.to_string())
    }
}


impl core::fmt::Display for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> fmt::Result {
        match self {
            Error::WhiteSpaceStart { element } => write!(fmt, "Whitespace at start of: '{}'", element),
            Error::WhiteSpaceEnd { element } => write!(fmt, "Whitespace at end of: '{}'", element),
            Error::TranscriptWithoutVersion { transcript } => {
                write!(fmt, "Transcript '{}' is missing a version", transcript)
            }
            Error::LabelTooShort { label, actual, min } => {
                write!(fmt, "Label '{}' is too short ({} < required {})", label, actual, min)
            }
            _ =>  write!(fmt, "{self:?}")
        }
    }
}

impl std::error::Error for Error {}