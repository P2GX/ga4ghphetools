//! SexDuplet
//! The duplet and the QC routines for the PMID column
//! 

use std::collections::HashSet;
use lazy_static::lazy_static;


use crate::template::curie;
use crate::header::header_duplet::HeaderDupletItem;
use crate::error::{self, Error, Result};
use crate::header::age_util;

use super::header_duplet::{self, HeaderDuplet, HeaderDupletItemFactory};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SexDuplet {}

lazy_static! {
    pub static ref ALLOWED_SEX_DUPLET_ITEMS: HashSet<String> =  {
        let mut set = HashSet::new();
        set.insert("M".to_string());
        set.insert("F".to_string());
        set.insert("O".to_string());
        set.insert("U".to_string());
        set
    };
}


impl HeaderDupletItem for SexDuplet {
    fn row1(&self) -> String {
        "sex".to_string()
    }

    fn row2(&self) -> String {
        "M:F:O:U".to_string()
    }

    fn qc_cell(&self, cell_contents: &str) -> Result<()> {
        header_duplet::check_empty_for_field(cell_contents, "sex")?;
        match ALLOWED_SEX_DUPLET_ITEMS.contains(cell_contents) {
            true => Ok(()),
            false => Err(Error::SexFieldError { msg: format!("Malformed entry in sex field: '{}'", cell_contents) })
        }
    }
    
    fn get_options(&self) -> Vec<String> {
        vec!["M".to_string(), "F".to_string(), "O".to_string(),"U".to_string()]
    }

    
}

impl HeaderDupletItemFactory for SexDuplet {
    fn from_table(row1: &str, row2: &str) -> Result<Self> where Self: Sized {
        let duplet = Self::default();
        if duplet.row1() != row1 {
            return Err(Error::HeaderError { msg: format!("Malformed sex Header: Expected '{}' but got '{}'", duplet.row1(), row1) });
        } else if duplet.row2() != row2 {
            return Err(Error::HeaderError { msg: format!("Malformed sex Header: Expected '{}' but got '{}'", duplet.row2(), row2) });
        } else {
            return Ok(duplet);
        }
    }
    
    fn into_enum(self) -> super::header_duplet::HeaderDuplet {
        HeaderDuplet::SexDuplet(self)
    }
}


impl SexDuplet {
    pub fn new() -> Self {
        SexDuplet{}
    }
}


#[cfg(test)]
mod test {
    use std::result;

    use super::*;
    use rstest::{fixture, rstest};


    #[rstest]
    #[case("male", "Malformed entry in sex field: 'male'")]
    #[case("f", "Malformed entry in sex field: 'f'")]
    #[case("", "sex must not be empty")]
    #[case("n/a", "Malformed entry in sex field: 'n/a'")]
    fn test_invalid_deceased_field(#[case] item:&str, #[case] response:&str) {
        let duplet = SexDuplet::default();
        let result = duplet.qc_cell(item);
        assert!(result.is_err());
        assert_eq!(response, result.unwrap_err().to_string());
    }

   

    #[rstest]
    #[case("M")]
    #[case("F")]
    #[case("O")]
    #[case("U")]
    fn test_valid_sex_field(#[case] item:&str) {
        let duplet = SexDuplet::default();
        let result = duplet.qc_cell(item);
        assert!(result.is_ok());
    }


    #[rstest]
    fn test_valid_ctor() {
        let duplet = SexDuplet::from_table("sex", "M:F:O:U");
        assert!(duplet.is_ok());
    }

    #[rstest]
    #[case("sex", "str", "Malformed sex Header: Expected 'M:F:O:U' but got 'str'")]
    #[case("sex ", "M:F:O:U", "Malformed sex Header: Expected 'sex' but got 'sex '")]
    #[case("sex", "M:F:U:O", "Malformed sex Header: Expected 'M:F:O:U' but got 'M:F:U:O'")]
    fn test_invalid_ctor(#[case] r1:&str, #[case] r2:&str, #[case] err_msg:&str) {
        let duplet = SexDuplet::from_table(r1, r2);
        assert!(duplet.is_err());
        assert!(matches!(&duplet, Err(Error::HeaderError { .. })));
        assert_eq!(err_msg, duplet.unwrap_err().to_string());
    }

}

