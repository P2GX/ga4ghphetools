
//! Allele1Duplet
//! The duplet and the QC routines for the PMID column
//! 

use std::cell;

use crate::template::curie;
use crate::header_duplet::header_duplet::HeaderDupletItem;
use crate::error::{self, Error, Result};
use crate::header_duplet::allele_util;

#[derive(Debug, Default)]
pub struct Allele1Duplet {}

impl HeaderDupletItem for Allele1Duplet {
    fn row1(&self) -> String {
        "allele_1".to_string()
    }

    fn row2(&self) -> String {
        "str".to_string()
    }

    fn qc_cell(&self, cell_contents: &str) -> Result<()> {
        Self::check_empty(cell_contents)?;
        Self::check_white_space(cell_contents)?;
        if cell_contents.starts_with("c.") {
            if allele_util::check_valid_hgvs(cell_contents) {
                return Ok(());
            }
        } else {
            if allele_util::check_valid_structural(cell_contents){
                return Ok(());
            }
        }
        Err(Error::AlleleError { msg: format!("Malformed allele '{}'", cell_contents) })
    }

    fn from_table(row1: &str, row2: &str) -> Result<Self> where Self: Sized {
        let duplet = Self::default();
        if duplet.row1() != row1 {
            return Err(Error::HeaderError { msg: format!("Malformed allele_1 Header: Expected '{}' but got '{}'", duplet.row1(), row1) });
        } 
        if duplet.row2() != row2 {
            return Err(Error::HeaderError { msg: format!("Malformed allele_1 Header: Expected '{}' but got '{}'", duplet.row2(), row2) });
        }
        return Ok(duplet);
    }
}



#[cfg(test)]
mod test {
    use super::*;
    use rstest::{fixture, rstest};

    #[rstest]
    #[case("c123A>G","Malformed allele 'c123A>G'")]
    #[case("c.123A>G ", "Trailing whitespace in 'c.123A>G '")]
    #[case(" c.123A>G","Leading whitespace in ' c.123A>G'")]
    #[case("c.123A > G", "Malformed allele 'c.123A > G'")]
    fn test_invalid_allele_1(#[case] item:&str, #[case] response:&str) {
        let duplet = Allele1Duplet::default();
        let result = duplet.qc_cell(item);
        assert!(result.is_err());
        assert_eq!(response, result.unwrap_err().to_string());
    }


    #[rstest]
    #[case("c.123A>G")]      // Substitution
    #[case("c.34del")]       // Deletion
    #[case("c.100G>A")]      // Another substitution
    #[case("c.200_201del")]  // Deletion with range]
    #[case("c.123_124insT")] // Insertion
    #[case("c.123_124delinsT")]
    fn test_valid_allele_1(#[case] item:&str) {
        let duplet = Allele1Duplet::default();
        let pmid = duplet.qc_cell(item);
        assert!(pmid.is_ok());
    }


    #[rstest]
    fn test_valid_ctor() {
        let duplet = Allele1Duplet::from_table("allele_1", "str");
        assert!(duplet.is_ok());
    }

    #[rstest]
    #[case("allele_1", "allele_1", "Malformed allele_1 Header: Expected 'str' but got 'allele_1'")]
    #[case("allele_2", "str", "Malformed allele_1 Header: Expected 'allele_1' but got 'allele_2'")]
    #[case("allele_1 ", "str", "Malformed allele_1 Header: Expected 'allele_1' but got 'allele_1 '")]
    fn test_invalid_ctor(#[case] r1:&str, #[case] r2:&str, #[case] err_msg:&str) {
        let duplet = Allele1Duplet::from_table(r1, r2);
        assert!(duplet.is_err());
        assert!(matches!(&duplet, Err(Error::HeaderError { .. })));
        assert_eq!(err_msg, duplet.unwrap_err().to_string());
    }

}





