//! DiseaseLabelDuplet
//! The duplet and the QC routines for the disease_label column
//! 


use crate::header_duplet::header_duplet::HeaderDupletItem;
use crate::error::{self, Error, Result};

#[derive(Debug, Default)]
pub struct DiseaseLabelDuplet {}

impl HeaderDupletItem for DiseaseLabelDuplet  {
    fn row1(&self) -> String {
        "disease_label".to_string()
    }

    fn row2(&self) -> String {
       "str".to_string()
    }

    fn qc_cell(&self, cell_contents: &str) -> Result<()> {
        Self::check_white_space(cell_contents)?;
        Self::check_empty(cell_contents)?;
        Ok(())
    }

    fn from_table(row1: &str, row2: &str) -> Result<Self> where Self: Sized {
        let duplet = Self::default();
        if duplet.row1() != row1 {
            return Err(Error::HeaderError { msg: format!("Malformed disease_label Header: Expected '{}' but got '{}'", duplet.row1(), row1) });
        } 
        if duplet.row2() != row2 {
            return Err(Error::HeaderError { msg: format!("Malformed disease_label Header: Expected '{}' but got '{}'", duplet.row2(), row2) });
        }
        return Ok(duplet);
    }
}


#[cfg(test)]
mod test {
    use std::result;

    use super::*;
    use rstest::{fixture, rstest};

    #[rstest]
    #[case("Marfan syndrome ","Trailing whitespace in 'Marfan syndrome '")]
    #[case(" Marfan syndrome", "Leading whitespace in ' Marfan syndrome'")]
    #[case("Marfan  syndrome","Consecutive whitespace in 'Marfan  syndrome'")]
    #[case("","Value must not be empty")]
    fn test_invalid_disease_label(#[case] item:&str, #[case] response:&str) {
        let duplet = DiseaseLabelDuplet::default();
        let result = duplet.qc_cell(item);
        assert!(result.is_err());
        assert_eq!(response, result.unwrap_err().to_string());
    }

    #[rstest]
    #[case("Marfan syndrome")]
    #[case("Neurofibromattosis type 1")]
    fn test_valid_disease_label(#[case] item:&str) {
        let duplet = DiseaseLabelDuplet::default();
        let result = duplet.qc_cell(item);
        assert!(result.is_ok());
    }


    #[rstest]
    fn test_valid_ctor() {
        let duplet = DiseaseLabelDuplet::from_table("disease_label", "str");
        assert!(duplet.is_ok());
    }

    #[rstest]
    #[case("disease_label", "CURIE", "Malformed disease_label Header: Expected 'str' but got 'CURIE'")]
    #[case("disease_label ", "str", "Malformed disease_label Header: Expected 'disease_label' but got 'disease_label '")]
    #[case("disease_id", "str ", "Malformed disease_label Header: Expected 'disease_label' but got 'disease_id'")]
    fn test_invalid_ctor(#[case] r1:&str, #[case] r2:&str, #[case] err_msg:&str) {
        let duplet = DiseaseLabelDuplet::from_table(r1, r2);
        assert!(duplet.is_err());
        assert!(matches!(&duplet, Err(Error::HeaderError { .. })));
        assert_eq!(err_msg, duplet.unwrap_err().to_string());
    }

}





