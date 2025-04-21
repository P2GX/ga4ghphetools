
//! HgncDuplet
//! The duplet and the QC routines for the PMID column
//! 

use crate::template::curie;
use crate::header::header_duplet::HeaderDupletItem;
use crate::error::{self, Error, Result};

use super::header_duplet::{self, HeaderDuplet, HeaderDupletItemFactory};


#[derive(Clone, Debug, Default, PartialEq)]
pub struct HgncDuplet {}

impl HeaderDupletItem for HgncDuplet {
    fn row1(&self) -> String {
        "HGNC_id".to_string()
    }

    fn row2(&self) -> String {
        "CURIE".to_string()
    }

    fn qc_cell(&self, cell_contents: &str) -> Result<()> {
        header_duplet::check_valid_curie(cell_contents)?;
        if ! cell_contents.starts_with("HGNC")  {
            return Err(Error::HgncError { msg: format!("HGNC id has invalid prefix: '{}'", cell_contents),
            });
        };
        Ok(())
    }

    fn get_options(&self) -> Vec<String> {
        vec!["edit".to_string(), "remove whitespace".to_string()]
    }
}

impl HeaderDupletItemFactory for HgncDuplet {
    fn from_table(row1: &str, row2: &str) -> Result<Self> where Self: Sized {
        let duplet = Self::default();
        if duplet.row1() != row1 {
            return Err(Error::HeaderError { msg: format!("Malformed HGNC Header: Expected '{}' but got '{}'", duplet.row1(), row1) });
        } 
        if duplet.row2() != row2 {
            return Err(Error::HeaderError { msg: format!("Malformed HGNC Header: Expected '{}' but got '{}'", duplet.row2(), row2) });
        }
        return Ok(duplet);
    }

    fn into_enum(self) -> super::header_duplet::HeaderDuplet {
        HeaderDuplet::HgncDuplet(self)
    }
}

impl HgncDuplet {
    pub fn new() -> Self {
        Self{}
    }
}


#[cfg(test)]
mod test {
    use super::*;
    use rstest::{fixture, rstest};

    #[rstest]
    #[case("HGNC34","Invalid CURIE with no colon: 'HGNC34'")]
    #[case("HGNC: 11998", "Contains stray whitespace: 'HGNC: 11998'")]
    #[case("HGNC:11998 ","Contains stray whitespace: 'HGNC:11998 '")]
    #[case(" HGNC:11998", "Contains stray whitespace: ' HGNC:11998'")]
    #[case("HNC:11998",  "HGNC id has invalid prefix: 'HNC:11998'")]
    #[case("HGNC:", "Invalid CURIE with no suffix: 'HGNC:'")]
    #[case(":11998","Invalid CURIE with no prefix: ':11998'")]
    #[case("GNC:11998", "HGNC id has invalid prefix: 'GNC:11998'")]
    #[case("MONDO:0007947", "HGNC id has invalid prefix: 'MONDO:0007947'")]
    #[case("","Empty CURIE")]
    fn test_invalid_hgnc(#[case] item:&str, #[case] response:&str) {
        let duplet = HgncDuplet::default();
        let result = duplet.qc_cell(item);
        assert!(result.is_err());
        assert_eq!(response, result.unwrap_err().to_string());
    }

    #[rstest]
    #[case("HGNC:3603")]
    #[case("HGNC:11998")]
    fn test_valid_hgnc(#[case] item:&str) {
        let duplet = HgncDuplet::default();
        let pmid = duplet.qc_cell(item);
        assert!(pmid.is_ok());
    }


    #[rstest]
    fn test_valid_ctor() {
        let duplet = HgncDuplet::from_table("HGNC_id", "CURIE");
        assert!(duplet.is_ok());
    }

    #[rstest]
    #[case("HGNC_id", "str", "Malformed HGNC Header: Expected 'CURIE' but got 'str'")]
    #[case("HGNC_id ", "CURIE", "Malformed HGNC Header: Expected 'HGNC_id' but got 'HGNC_id '")]
    #[case("HGNC_id", "CURIE ", "Malformed HGNC Header: Expected 'CURIE' but got 'CURIE '")]
    fn test_invalid_ctor(#[case] r1:&str, #[case] r2:&str, #[case] err_msg:&str) {
        let duplet = HgncDuplet::from_table(r1, r2);
        assert!(duplet.is_err());
        assert!(matches!(&duplet, Err(Error::HeaderError { .. })));
        assert_eq!(err_msg, duplet.unwrap_err().to_string());
    }

}






