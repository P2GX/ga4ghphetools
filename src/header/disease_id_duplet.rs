//! DiseaseIdDuplet
//! The duplet and the QC routines for the disease_id column
//! 


use crate::header::header_duplet::HeaderDupletItem;
use crate::error::{self, Error, Result};

use super::header_duplet::{self, HeaderDuplet, HeaderDupletItemFactory};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DiseaseIdDuplet {}


impl HeaderDupletItem for DiseaseIdDuplet {
    fn row1(&self) -> String {
        "disease_id".to_string()
    }

    fn row2(&self) -> String {
       "CURIE".to_string()
    }

    fn qc_cell(&self, cell_contents: &str) -> Result<()> {
        header_duplet::check_valid_curie(cell_contents)?;
        if !(cell_contents.starts_with("OMIM") || cell_contents.starts_with("MONDO")) {
            return Err(Error::DiseaseIdError {
                msg: format!("Disease id has invalid prefix: '{}'", cell_contents),
            });
        }
        if cell_contents.starts_with("OMIM:") {
            if cell_contents.len() != 11 {
                return Err(Error::DiseaseIdError {
                    msg: format!("OMIM identifiers must have 6 digits: '{}'", cell_contents),
                });
            }
        }

        Ok(())
    }

   
}

impl HeaderDupletItemFactory for DiseaseIdDuplet {
    fn from_table(row1: &str, row2: &str) -> Result<Self> where Self: Sized {
        let duplet = Self::default();
        if duplet.row1() != row1 {
            return Err(Error::HeaderError { msg: format!("Malformed disease_id Header: Expected '{}' but got '{}'", duplet.row1(), row1) });
        } 
        if duplet.row2() != row2 {
            return Err(Error::HeaderError { msg: format!("Malformed disease_id Header: Expected '{}' but got '{}'", duplet.row2(), row2) });
        }
        return Ok(duplet); 
    }

    fn into_enum(self) -> super::header_duplet::HeaderDuplet {
        HeaderDuplet::DiseaseIdDuplet(self)
    }
}

impl DiseaseIdDuplet {
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
    #[case("OMIM154700","Invalid CURIE with no colon: 'OMIM154700'")]
    #[case("OMIM: 154700", "Contains stray whitespace: 'OMIM: 154700'")]
    #[case("OMIM:154700 ","Contains stray whitespace: 'OMIM:154700 '")]
    #[case(" OMIM:154700", "Contains stray whitespace: ' OMIM:154700'")]
    #[case(" OMIM:154700 ",  "Contains stray whitespace: ' OMIM:154700 '")]
    #[case("OMIM:", "Invalid CURIE with no suffix: 'OMIM:'")]
    #[case(":154700","Invalid CURIE with no prefix: ':154700'")]
    #[case("OMM:154700", "Disease id has invalid prefix: 'OMM:154700'")]
    #[case("MOND:0007947", "Disease id has invalid prefix: 'MOND:0007947'")]
    #[case("OMIM:54700", "OMIM identifiers must have 6 digits: 'OMIM:54700'")]
    #[case("","Empty CURIE")]
    fn test_invalid_disease_id(#[case] item:&str, #[case] response:&str) {
        let duplet = DiseaseIdDuplet::default();
        let result = duplet.qc_cell(item);
        assert!(result.is_err());
        assert_eq!(response, result.unwrap_err().to_string());
    }

    #[rstest]
    #[case("MONDO:0007947")]
    #[case("OMIM:600947")]
    fn test_valid_disease_id(#[case] item:&str) {
        let duplet = DiseaseIdDuplet::default();
        let result = duplet.qc_cell(item);
        assert!(result.is_ok());
    }


    #[rstest]
    fn test_valid_ctor() {
        let duplet = DiseaseIdDuplet::from_table("disease_id", "CURIE");
        assert!(duplet.is_ok());
    }

    #[rstest]
    #[case("disease_id", "str", "Malformed disease_id Header: Expected 'CURIE' but got 'str'")]
    #[case("disease_id ", "CURIE", "Malformed disease_id Header: Expected 'disease_id' but got 'disease_id '")]
    #[case("disease_id", "CURIE ", "Malformed disease_id Header: Expected 'CURIE' but got 'CURIE '")]
    fn test_invalid_ctor(#[case] r1:&str, #[case] r2:&str, #[case] err_msg:&str) {
        let duplet = DiseaseIdDuplet::from_table(r1, r2);
        assert!(duplet.is_err());
        assert!(matches!(&duplet, Err(Error::HeaderError { .. })));
        assert_eq!(err_msg, duplet.unwrap_err().to_string());
    }

}

