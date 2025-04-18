//! PmidDuplet
//! The duplet and the QC routines for the PMID column
//! 

use crate::template::curie;
use crate::header_duplet::header_duplet::HeaderDupletItem;
use crate::error::{self, Error, Result};


#[derive(Debug, Default)]
pub struct PmidDuplet {}


impl HeaderDupletItem for PmidDuplet {
    fn row1(&self) -> String {
        "PMID".to_string()
    }

    fn row2(&self) -> String {
        "CURIE".to_string()
    }

    fn qc_cell(&self, cell_contents: &str) -> crate::error::Result<()> {
        let _ = curie::check_valid_curie(cell_contents)?;
        if !cell_contents.starts_with("PMID") {
            return Err(Error::CurieError {
                msg: format!("Invalid PubMed prefix: '{}'", cell_contents),
            });
        }

        Ok(())
    }
    
    fn from_table(row1: &str, row2: &str) -> Result<Self> where Self: Sized {
        let pmid_d = Self::default();
        if pmid_d.row1() != row1 {
            return Err(Error::HeaderError { msg: format!("Malformed PMID Header: Expected 'PMID' but got '{row1}'") });
        } 
        if pmid_d.row2() != row2 {
            return Err(Error::HeaderError { msg: format!("Malformed PMID Header: Expected 'CURIE' but got '{row2}'") });
        }
        return Ok(pmid_d);
    }
}



#[cfg(test)]
mod test {
    use super::*;
    use rstest::{fixture, rstest};


    #[rstest]
    #[case("PMID: 12345", "Contains stray whitespace: 'PMID: 12345'")]
    #[case("PMID:12345 ", "Contains stray whitespace: 'PMID:12345 '")]
    #[case (" PMID:12345", "Contains stray whitespace: ' PMID:12345'")]
    #[case("PMD:12345", "Invalid PubMed prefix: 'PMD:12345'")]
    #[case("PMID12345", "Invalid CURIE with no colon: 'PMID12345'")]
    #[case("PMID:12a45", "Invalid CURIE with non-digit characters in suffix: 'PMID:12a45'")]
    #[case("", "Empty CURIE")]
    fn test_invalid_pmid(#[case] item:&str, #[case] response:&str,) {
        let pmid_hd = PmidDuplet{};
        let pmid = pmid_hd.qc_cell(item);
        assert!(pmid.is_err());
        assert_eq!(response, pmid.unwrap_err().to_string());
    }

    #[rstest]
    #[case("PMID:12345")]
    #[case("PMID:98765")]
    fn test_valid_pmid(#[case] item:&str) {
        let pmid_hd = PmidDuplet{};
        let pmid = pmid_hd.qc_cell(item);
        assert!(pmid.is_ok());
    }


    #[rstest]
    fn test_valid_ctor() {
        let pmid_d = PmidDuplet::from_table("PMID", "CURIE");
        assert!(pmid_d.is_ok());
    }

    #[rstest]
    #[case("PMID", "str", "Malformed PMID Header: Expected 'CURIE' but got 'str'")]
    #[case("pmid", "CURIE", "Malformed PMID Header: Expected 'PMID' but got 'pmid'")]
    #[case("PMIR", "CURIE", "Malformed PMID Header: Expected 'PMID' but got 'PMIR'")]
    fn test_invalid_ctor(#[case] r1:&str, #[case] r2:&str, #[case] err_msg:&str) {
        let pmid_d = PmidDuplet::from_table(r1, r2);
        assert!(pmid_d.is_err());
        assert!(matches!(&pmid_d, Err(Error::HeaderError { .. })));
        assert_eq!(err_msg, pmid_d.unwrap_err().to_string());
    }

}