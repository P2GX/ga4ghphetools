//! PhetoolsQC
//! 
//! 
use std::fmt::format;
use std::sync::Arc;

use ontolius::ontology::csr::FullCsrOntology;
use crate::error::{self, Error, Result};
use crate::pptcolumn::header_duplet::HeaderDuplet;
pub struct PheToolsQc {
    mendelian_headers: Arc<[HeaderDuplet]>,
}

impl Error {
    fn mismatched_header(idx: usize, expected: &HeaderDuplet, observed: &HeaderDuplet) -> Self {
        let msg = format!("Expected header '{}'/{}' at column {} but got '{}'/{}'",
            expected.row1(), expected.row2(), idx, observed.row1(), observed.row2());
        Error::TemplateError { msg }
    }
}

/// Create the HeaderDuplets used for Mendelian tempaltes.
fn expected_mendelian_fields() -> Vec<HeaderDuplet> {
    vec![
        HeaderDuplet::new("PMID", "CURIE"),
        HeaderDuplet::new("title", "str"),
        HeaderDuplet::new("individual_id", "str"),
        HeaderDuplet::new("comment", "optional"),
        HeaderDuplet::new("disease_id", "CURIE"),
        HeaderDuplet::new("disease_label", "str"),
        HeaderDuplet::new("HGNC_id", "CURIE"),
        HeaderDuplet::new("gene_symbol", "str"),
        HeaderDuplet::new("transcript", "str"),
        HeaderDuplet::new("allele_1", "str"),
        HeaderDuplet::new("allele_2", "str"),
        HeaderDuplet::new("variant.comment", "optional"),
        HeaderDuplet::new("age_of_onset", "age"),
        HeaderDuplet::new("age_at_last_encounter", "age"),
        HeaderDuplet::new("deceased", "yes/no/na"),
        HeaderDuplet::new("sex", "M:F:O:U"),
        HeaderDuplet::new("HPO", "na"),
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

    pub fn is_valid_mendelian_header(&self, hdup_list: &[HeaderDuplet]) -> Result<bool> {
        for i in 0..self.mendelian_headers.len() {
            if self.mendelian_headers[i] != hdup_list[i] {
                return Err(Error::mismatched_header(i, &self.mendelian_headers[i], &hdup_list[i]));
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

    use super::*;

    #[test]
    fn test_name() -> Result<()> {
        let mut headers = expected_mendelian_fields();
        let mut false_headers = expected_mendelian_fields();
        let hdup = HeaderDuplet::new("false", "false");
        false_headers.push(hdup);
        //let ptqc = PheToolsQc::new();
        //assert!(ptqc.is_valid_mendelian_header(&headers));
        //assert!(!ptqc.is_valid_mendelian_header(&false_headers));
        assert_ne!(headers, false_headers);
        assert_eq!(headers, headers);
        Ok(())
    }
}

// endregion: --- Tests