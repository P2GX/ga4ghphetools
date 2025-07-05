//! ValidationErrors
//! a structure to return all validation errors encountered as a Vector of strings
//! 
//! # Example
//! 
//! ```ignore
//! let mut v = ValidationErrors::new();
//! v.push_result(check_pmid(&ind.pmid));
//! v.push_result(check_title(&ind.title));
//! v.into_result()
//! ```
//! The final line will either return Ok(()) or the Error
//!


use std::fmt;

use serde::Serialize;


#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationErrors {
    errors: Vec<String>,
}


impl ValidationErrors {
    
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_string(error: impl Into<String>) -> Self {
        Self { errors: vec![error.into()] }
    }

    pub fn push_result(&mut self, res: Result<(), String>) {
        if let Err(e) = res {
            self.errors.push(e);
        }
    }

    pub fn push_verr_result(&mut self, res: Result<(), ValidationErrors>) {
        if let Err(verr) = res {
            if verr.has_error() {
                self.errors.extend(verr.errors);
            }
        }
    }

    pub fn extend_errs(&mut self, err_strings: Vec<String>) {
        self.errors.extend(err_strings);
    }

    pub fn push_str(&mut self, message: impl Into<String>) {
        self.errors.push(message.into());
    }

    pub fn into_result(self) -> Result<(), Vec<String>> {
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors)
        }
    }


    pub fn from_one_err(error: impl Into<String>) -> Self {
        Self { errors: vec![error.into()] }
    }

    pub fn add_errors(&mut self, additional_errors: &Vec<String>) {
        self.errors.extend(additional_errors.clone());
    }

    pub fn has_error(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn errors(&self) -> &Vec<String> {
        &self.errors
    }

    pub fn ok(self) -> Result<(), Self> {
        if self.has_error() {
            Err(self)
        } else {
            Ok(())
        }
    }
}

impl std::fmt::Display for ValidationErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Validation errors: {:?}", self.errors)
    }
}

impl std::error::Error for ValidationErrors {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }

    fn description(&self) -> &str {
        "description() is deprecated; use Display"
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        self.source()
    }
}

