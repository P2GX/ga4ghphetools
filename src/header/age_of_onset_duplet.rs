
//! AgeOfOnsetDuplet
//! The duplet and the QC routines for the PMID column
//! 

use crate::template::curie;
use crate::header::header_duplet::HeaderDupletItem;
use crate::error::{self, Error, Result};
use crate::header::age_util;

use super::header_duplet::{self, HeaderDuplet, HeaderDupletItemFactory};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct AgeOfOnsetDuplet {}

impl HeaderDupletItem for AgeOfOnsetDuplet {
    fn row1(&self) -> String {
        "age_of_onset".to_string()
    }

    fn row2(&self) -> String {
         "age".to_string()
    }

    fn qc_cell(&self, cell_contents: &str) -> Result<()> {
        header_duplet::check_white_space_for_field(cell_contents, "age_of_onset")?;
        header_duplet::check_empty_for_field(cell_contents, "age_of_onset")?;
        if age_util::is_valid_age_string(cell_contents) {
            Ok(())
        } else {
            Err(Error::AgeParseError { msg: format!("Malformed age_of_onset '{}'", cell_contents) })
        }
    }

   
}

impl HeaderDupletItemFactory for AgeOfOnsetDuplet {
    fn from_table(row1: &str, row2: &str) -> Result<Self> where Self: Sized {
        let duplet = Self::default();
        if duplet.row1() != row1 {
            return Err(Error::HeaderError { msg: format!("Malformed age_of_onset Header: Expected '{}' but got '{}'", duplet.row1(), row1) });
        } else if duplet.row2() != row2 {
            return Err(Error::HeaderError { msg: format!("Malformed age_of_onset Header: Expected '{}' but got '{}'", duplet.row2(), row2) });
        } else {
            return Ok(duplet);
        }
    }

    fn into_enum(self) -> super::header_duplet::HeaderDuplet {
        HeaderDuplet::AgeOfOnsetDuplet(self)
    }
}

impl AgeOfOnsetDuplet {
    pub fn new() -> Self {
        Self{}
    }
}

#[cfg(test)]
mod test {
    use std::result;

    use super::*;
    use rstest::{fixture, rstest};


    #[rstest]
    #[case("2 years", "Malformed age_of_onset '2 years'")]
    #[case("2Y", "Malformed age_of_onset '2Y'")]
    #[case("Lateonset", "Malformed age_of_onset 'Lateonset'")]
    fn test_invalid_age(#[case] item:&str, #[case] response:&str) {
        let duplet = AgeOfOnsetDuplet::default();
        let result = duplet.qc_cell(item);
        assert!(result.is_err());
        assert_eq!(response, result.unwrap_err().to_string());
    }

   

    #[rstest]
    #[case("P2Y")]
    #[case("P22Y3M")]
    #[case("P22Y3M21D")]
    #[case("Late onset")]
    #[case("Middle age onset")]
    #[case("Young adult onset")]
    #[case( "Late young adult onset")]
    #[case("Intermediate young adult onset")]
    #[case("Early young adult onset")]
    #[case("Adult onset")]
    #[case("Juvenile onset")]
    #[case("Childhood onset")]
    #[case("Infantile onset")]
    #[case("Neonatal onset")]
    #[case("Congenital onset")]
    #[case("Antenatal onset")]
    #[case("Embryonal onset")]
    #[case("Fetal onset")]
    #[case("Late first trimester onset")]
    #[case("Second trimester onset")]
    #[case("Third trimester onset")]
    #[case("G23w4d")]
    fn test_valid_age(#[case] item:&str) {
        let duplet = AgeOfOnsetDuplet::default();
        let result = duplet.qc_cell(item);
        assert!(result.is_ok());
    }


    #[rstest]
    fn test_valid_ctor() {
        let duplet = AgeOfOnsetDuplet::from_table("age_of_onset", "age");
        assert!(duplet.is_ok());
    }

    #[rstest]
    #[case("age of onset", "str", "Malformed age_of_onset Header: Expected 'age_of_onset' but got 'age of onset'")]
    #[case("age_of_onset ", "age", "Malformed age_of_onset Header: Expected 'age_of_onset' but got 'age_of_onset '")]
    #[case("age_of_onset", "str", "Malformed age_of_onset Header: Expected 'age' but got 'str'")]
    fn test_invalid_ctor(#[case] r1:&str, #[case] r2:&str, #[case] err_msg:&str) {
        let duplet = AgeOfOnsetDuplet::from_table(r1, r2);
        assert!(duplet.is_err());
        assert!(matches!(&duplet, Err(Error::HeaderError { .. })));
        assert_eq!(err_msg, duplet.unwrap_err().to_string());
    }

}

