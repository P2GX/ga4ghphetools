
//! TranscriptDuplet
//! The duplet and the QC routines for the PMID column
//! 

use std::cell;

use crate::template::curie;
use crate::header::header_duplet::HeaderDupletItem;
use crate::error::{self, Error, Result};

use super::header_duplet::{self, HeaderDuplet, HeaderDupletItemFactory};


#[derive(Clone, Debug, Default, PartialEq)]
pub struct TranscriptDuplet {}

impl HeaderDupletItem for TranscriptDuplet {
    fn row1(&self) -> String {
        "transcript".to_string()
    }

    fn row2(&self) -> String {
        "str".to_string()
    }

    fn qc_cell(&self, cell_contents: &str) -> Result<()> {
        header_duplet::check_empty(cell_contents)?;
        if ! cell_contents.starts_with("ENST") && ! cell_contents.starts_with("NM_") {
            return Err(Error::unrecognized_transcript_prefix(cell_contents));
        }  
        if ! cell_contents.contains(".") {
            return Err(Error::lacks_transcript_version(cell_contents));
        } 
        if let Some((before_last, last)) = cell_contents.rsplit_once('.') {
            println!("{} {}", before_last, last);
            if before_last.is_empty() {
                return Err(Error::TranscriptError { msg: format!("Maformed transcript: '{}'", cell_contents) });
            }
            if ! last.chars().all(|c| c.is_ascii_digit()) {
                return Err(Error::TranscriptError { msg: format!("Maformed transcript version: '{}'", cell_contents) });
            }
        }
        Ok(())
    }

    fn get_options(&self) -> Vec<String> {
        vec!["edit".to_string(), "remove whitespace".to_string()]
    }
    
}

impl HeaderDupletItemFactory for TranscriptDuplet {
    fn from_table(row1: &str, row2: &str) -> Result<Self> where Self: Sized {
        let duplet = Self::default();
        if duplet.row1() != row1 {
            return Err(Error::HeaderError { msg: format!("Malformed transcript Header: Expected '{}' but got '{}'", duplet.row1(), row1) });
        } 
        if duplet.row2() != row2 {
            return Err(Error::HeaderError { msg: format!("Malformed transcript Header: Expected '{}' but got '{}'", duplet.row2(), row2) });
        }
        return Ok(duplet);
    }

    fn into_enum(self) -> super::header_duplet::HeaderDuplet {
        HeaderDuplet::TranscriptDuplet(self)
    }
}

impl TranscriptDuplet {
    pub fn new() -> Self {
        TranscriptDuplet{}
    }
}


#[cfg(test)]
mod test {
    use super::*;
    use rstest::{fixture, rstest};

    #[rstest]
    #[case("NM_006139","Transcript 'NM_006139' is missing a version")]
    #[case("NM006139.4", "Unrecognized transcript prefix 'NM006139.4'")]
    fn test_invalid_gene_symbol(#[case] item:&str, #[case] response:&str) {
        let duplet = TranscriptDuplet::default();
        let result = duplet.qc_cell(item);
        assert!(result.is_err());
        assert_eq!(response, result.unwrap_err().to_string());
    }

    #[rstest]
    #[case("NM_006139.4")]
    #[case("ENST00000316623.10")]
    fn test_valid_gene_symbol(#[case] item:&str) {
        let duplet = TranscriptDuplet::default();
        let result = duplet.qc_cell(item);
        assert!(result.is_ok());
    }


    #[rstest]
    fn test_valid_ctor() {
        let duplet = TranscriptDuplet::from_table("transcript", "str");
        assert!(duplet.is_ok());
    }

    #[rstest]
    #[case("gene", "str", "Malformed transcript Header: Expected 'transcript' but got 'gene'")]
    #[case("transcript ", "str", "Malformed transcript Header: Expected 'transcript' but got 'transcript '")]
    #[case("transcript", "CURIE", "Malformed transcript Header: Expected 'str' but got 'CURIE'")]
    fn test_invalid_ctor(#[case] r1:&str, #[case] r2:&str, #[case] err_msg:&str) {
        let duplet = TranscriptDuplet::from_table(r1, r2);
        assert!(duplet.is_err());
        assert!(matches!(&duplet, Err(Error::HeaderError { .. })));
        assert_eq!(err_msg, duplet.unwrap_err().to_string());
    }

}





