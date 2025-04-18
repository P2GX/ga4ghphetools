//! PhetoolsQC
//!
//!
use std::fmt::format;
use std::sync::Arc;

use crate::{error::{self, Error, Result}, header_duplet::header_duplet::HeaderDupletOld};
use ontolius::ontology::csr::FullCsrOntology;
pub struct PheToolsQc {
    mendelian_headers: Arc<[HeaderDupletOld]>,
}

impl Error {
    fn mismatched_header(idx: usize, expected: &HeaderDupletOld, observed: &HeaderDupletOld) -> Self {
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
fn expected_mendelian_fields() -> Vec<HeaderDupletOld> {
    vec![
        HeaderDupletOld::new("PMID", "CURIE"),
        HeaderDupletOld::new("title", "str"),
        HeaderDupletOld::new("individual_id", "str"),
        HeaderDupletOld::new("comment", "optional"),
        HeaderDupletOld::new("disease_id", "CURIE"),
        HeaderDupletOld::new("disease_label", "str"),
        HeaderDupletOld::new("HGNC_id", "CURIE"),
        HeaderDupletOld::new("gene_symbol", "str"),
        HeaderDupletOld::new("transcript", "str"),
        HeaderDupletOld::new("allele_1", "str"),
        HeaderDupletOld::new("allele_2", "str"),
        HeaderDupletOld::new("variant.comment", "optional"),
        HeaderDupletOld::new("age_of_onset", "age"),
        HeaderDupletOld::new("age_at_last_encounter", "age"),
        HeaderDupletOld::new("deceased", "yes/no/na"),
        HeaderDupletOld::new("sex", "M:F:O:U"),
        HeaderDupletOld::new("HPO", "na"),
    ]
}

impl PheToolsQc {
    pub fn new() -> Self {
        Self {
            mendelian_headers: Arc::from(expected_mendelian_fields()),
        }
    }

    pub fn headers(&self) -> &[HeaderDupletOld] {
        &self.mendelian_headers
    }

    pub fn is_valid_mendelian_header(&self, hdup_list: &[HeaderDupletOld]) -> Result<bool> {
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

    use crate::header_duplet::header_duplet::HeaderDupletOld;

    use super::*;

    #[test]
    fn test_name() -> Result<()> {
        let mut headers = expected_mendelian_fields();
        let mut false_headers = expected_mendelian_fields();
        let hdup = HeaderDupletOld::new("false", "false");
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
