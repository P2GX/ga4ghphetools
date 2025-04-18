//! IndividualIdDuplet
//! The duplet and the QC routines for the individual_id column
//! 

use std::collections::HashSet;

use crate::template::curie;
use crate::header_duplet::header_duplet::HeaderDupletItem;
use crate::error::{self, Error, Result};

#[derive(Debug)]
pub struct IndividualIdDuplet {}


/// These characters are not allowed in the individual id field
fn check_forbidden_chars(value: &str) -> Result<()> {
    let forbidden_chars: HashSet<char> = ['/', '\\', '(', ')', '.'].iter().copied().collect();
    match value.chars().find(|&c| forbidden_chars.contains(&c)) {
        Some(fc) => Err(Error::forbidden_character(fc, value)),
        None => Ok(()),
    }
}


impl HeaderDupletItem for IndividualIdDuplet {
    fn row1(&self) -> String {
        "individual_id".to_string()
    }

    fn row2(&self) -> String {
        "str".to_string()
    }

    fn qc_cell(&self, cell_contents: &str) -> Result<()> {
        Self::check_white_space(cell_contents)?;
        check_forbidden_chars(cell_contents)?;
        Self::check_empty(cell_contents)?;
        Ok(())
    }
    
    fn from_table(row1: &str, row2: &str) -> Result<Self> where Self: Sized {
        let duplet = Self{};
        if duplet.row1() != row1 {
            return Err(Error::HeaderError { msg: format!("Malformed individual_id Header: Expected '{}' but got '{}'", duplet.row1(), row1) });
        } else if duplet.row2() != row2 {
            return Err(Error::HeaderError { msg: format!("Malformed individual_id Header: Expected '{}' but got '{}'", duplet.row2(), row2) });
        } else {
            return Ok(duplet);
        }
    }
}


#[cfg(test)]
mod test {
    use super::*;
    use rstest::{fixture, rstest};


    #[rstest]
    #[case("person A ", "Trailing whitespace in 'person A '")]
    #[case(" person A", "Leading whitespace in ' person A'")]
    #[case ("individual/A", "Forbidden character '/' found in label 'individual/A'")]
    #[case("Family B\\individual 1", "Forbidden character '\\' found in label 'Family B\\individual 1'")]
    #[case("Person(A)", "Forbidden character '(' found in label 'Person(A)'")]
    #[case("individual)", "Forbidden character ')' found in label 'individual)'")]
    #[case("", "Value must not be empty")]
    fn test_invalid_individual_id(#[case] item:&str, #[case] response:&str,) {
        let duplet = IndividualIdDuplet{};
        let result = duplet.qc_cell(item);
        assert!(result.is_err());
        assert_eq!(response, result.unwrap_err().to_string());
    }

    #[rstest]
    #[case("individual A")]
    #[case("individual:a")]
    #[case("individual-a")]
    #[case("Family 1, individual A")]
    fn test_valid_individual_id(#[case] item:&str) {
        let duplet = IndividualIdDuplet{};
        let result = duplet.qc_cell(item);
        assert!(result.is_ok());
    }


    #[rstest]
    fn test_valid_ctor() {
        let duplet = IndividualIdDuplet::from_table("individual_id", "str");
        assert!(duplet.is_ok());
    }

    #[rstest]
    #[case("individual", "str", "Malformed individual_id Header: Expected 'individual_id' but got 'individual'")]
    #[case("individual_id", "CURIE", "Malformed individual_id Header: Expected 'str' but got 'CURIE'")]
    #[case("individual_id ", "str", "Malformed individual_id Header: Expected 'individual_id' but got 'individual_id '")]
    fn test_invalid_ctor(#[case] r1:&str, #[case] r2:&str, #[case] err_msg:&str) {
        let duplet = IndividualIdDuplet::from_table(r1, r2);
        assert!(duplet.is_err());
        assert!(matches!(&duplet, Err(Error::HeaderError { .. })));
        assert_eq!(err_msg, duplet.unwrap_err().to_string());
    }

}