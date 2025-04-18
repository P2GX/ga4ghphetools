//! HpoSeparatorDuplet
//! The duplet and the QC routines for the PMID column
//! 

use std::collections::HashSet;


use crate::template::curie;
use crate::header_duplet::header_duplet::HeaderDupletItem;
use crate::error::{self, Error, Result};
use crate::header_duplet::age_util;

#[derive(Debug, Default)]
pub struct HpoSeparatorDuplet {}



impl HeaderDupletItem for HpoSeparatorDuplet {
    fn row1(&self) -> String {
        "HPO".to_string() 
    }

    fn row2(&self) -> String {
        "na".to_string()
    }

    fn qc_cell(&self, cell_contents: &str) -> Result<()> {
        Self::check_empty(cell_contents)?;
        if cell_contents == "na" {
            Ok(())
        } else {
            Err(Error::HeaderError { msg: format!("Malformed HPO (separator) entry: '{}'", cell_contents) })
        }
    }

    fn from_table(row1: &str, row2: &str) -> Result<Self> where Self: Sized {
        let duplet = Self::default();
        if duplet.row1() != row1 {
            return Err(Error::HeaderError { msg: format!("Malformed HPO (separator) Header: Expected '{}' but got '{}'", duplet.row1(), row1) });
        } else if duplet.row2() != row2 {
            return Err(Error::HeaderError { msg: format!("Malformed HPO (separator) Header: Expected '{}' but got '{}'", duplet.row2(), row2) });
        } else {
            return Ok(duplet);
        }
    }
}




#[cfg(test)]
mod test {
    use std::result;

    use super::*;
    use rstest::{fixture, rstest};


    #[rstest]
    #[case("", "Value must not be empty")]
    #[case("na ", "Malformed HPO (separator) entry: 'na '")]
    #[case("n/a", "Malformed HPO (separator) entry: 'n/a'")]
    #[case("observed", "Malformed HPO (separator) entry: 'observed'")]
    #[case("excluded", "Malformed HPO (separator) entry: 'excluded'")]
    fn test_invalid_hpo_separator(#[case] item:&str, #[case] response:&str) {
        let duplet = HpoSeparatorDuplet::default();
        let result = duplet.qc_cell(item);
        assert!(result.is_err());
        assert_eq!(response, result.unwrap_err().to_string());
    }

   

    #[rstest]
    #[case("na")]
    fn test_valid_hpo_separator_field(#[case] item:&str) {
        let duplet = HpoSeparatorDuplet::default();
        let result = duplet.qc_cell(item);
        assert!(result.is_ok());
    }


    #[rstest]
    fn test_valid_ctor() {
        let duplet = HpoSeparatorDuplet::from_table("HPO", "na");
        assert!(duplet.is_ok());
    }

    #[rstest]
    #[case("HPO", "str", "Malformed HPO (separator) Header: Expected 'na' but got 'str'")]
    #[case("HPO ", "na", "Malformed HPO (separator) Header: Expected 'HPO' but got 'HPO '")]
    #[case("separator", "na", "Malformed HPO (separator) Header: Expected 'HPO' but got 'separator'")]
    fn test_invalid_ctor(#[case] r1:&str, #[case] r2:&str, #[case] err_msg:&str) {
        let duplet = HpoSeparatorDuplet::from_table(r1, r2);
        assert!(duplet.is_err());
        assert!(matches!(&duplet, Err(Error::HeaderError { .. })));
        assert_eq!(err_msg, duplet.unwrap_err().to_string());
    }

}
