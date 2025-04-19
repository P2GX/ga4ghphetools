//! PhetoolsQC
//!
//!
use std::fmt::format;
use std::sync::Arc;

use crate::{error::{self, Error, Result}, header::header_duplet::{HeaderDuplet, HeaderDupletItem}};
use ontolius::ontology::csr::FullCsrOntology;
pub struct PheToolsQc {
    mendelian_headers: Arc<[HeaderDuplet]>,
}

impl Error {
    fn mismatched_header(idx: usize, expected: &HeaderDuplet, observed: &HeaderDuplet) -> Self {
        let msg = format!(
            "Expected header '{}'/{}' at column {} but got '{}'/{}'",
            expected.row1(),
            expected.row2(),
            idx,
            observed.row1(),
            observed.row2()
        );
        Error::TemplateError { msg }
    }
}

/// Create the HeaderDuplets used for Mendelian tempaltes.
fn expected_mendelian_fields() -> Vec<HeaderDuplet> {
    vec![
    ]
}

impl PheToolsQc {
    pub fn new() -> Self {
        Self {
            mendelian_headers: Arc::from(expected_mendelian_fields()),
        }
    }

    pub fn headers(&self) -> &[HeaderDuplet] {
        &self.mendelian_headers
    }

    /// TODO document
    pub fn is_valid_mendelian_header(&self, hdup_list: &[HeaderDuplet]) -> Result<bool> {
        for i in 0..self.mendelian_headers.len() {
            if self.mendelian_headers[i] != hdup_list[i] {
                return Err(Error::mismatched_header(
                    i,
                    &self.mendelian_headers[i],
                    &hdup_list[i],
                ));
            }
        }
        Ok(true)
    }
}

// region:    --- Tests

#[cfg(test)]
mod tests {
    type Error = Box<dyn std::error::Error>;
    type Result<T> = core::result::Result<T, Error>; // For tests.

    use crate::header::{header_duplet::{HeaderDuplet, HeaderDupletItemFactory}, pmid_duplet::PmidDuplet};

    use super::*;

    #[test]
    fn test_name() -> Result<()> {
        let mut headers = expected_mendelian_fields();
        let mut false_headers = expected_mendelian_fields();
        let hdup = PmidDuplet::new();
        false_headers.push(hdup.into_enum());
        //let ptqc = PheToolsQc::new();
        //assert!(ptqc.is_valid_mendelian_header(&headers));
        //assert!(!ptqc.is_valid_mendelian_header(&false_headers));
        assert_ne!(headers, false_headers);
        assert_eq!(headers, headers);
        Ok(())
    }
}

// endregion: --- Tests
