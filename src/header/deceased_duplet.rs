
//! DeceasedDuplet
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
pub struct DeceasedDuplet {}


lazy_static! {
    pub static ref ALLOWED_DECEASED_ITEMS: HashSet<String> =  {
        let mut set = HashSet::new();
        set.insert("yes".to_string());
        set.insert("no".to_string());
        set.insert("na".to_string());
        set
    };
}

impl HeaderDupletItem for DeceasedDuplet {
    fn row1(&self) -> String {
        "deceased".to_string() 
    }

    fn row2(&self) -> String {
        "yes/no/na".to_string()
    }

    fn qc_cell(&self, cell_contents: &str) -> Result<()> {
        header_duplet::check_empty_for_field(cell_contents, "deceased")?;
        match ALLOWED_DECEASED_ITEMS.contains(cell_contents) {
            true => Ok(()),
            false => Err(Error::DeceasedError{msg: format!("Malformed deceased entry: '{}'", cell_contents)})
        }
    }

   
}

impl HeaderDupletItemFactory for DeceasedDuplet {
    fn from_table(row1: &str, row2: &str) -> Result<Self> where Self: Sized {
        let duplet = Self::default();
        if duplet.row1() != row1 {
            return Err(Error::HeaderError { msg: format!("Malformed deceased Header: Expected '{}' but got '{}'", duplet.row1(), row1) });
        } else if duplet.row2() != row2 {
            return Err(Error::HeaderError { msg: format!("Malformed deceased Header: Expected '{}' but got '{}'", duplet.row2(), row2) });
        } else {
            return Ok(duplet);
        }
    }

    fn into_enum(self) -> super::header_duplet::HeaderDuplet {
        HeaderDuplet::DeceasedDuplet(self)
    }
}


impl DeceasedDuplet {
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
    #[case("?", "Malformed deceased entry: '?'")]
    #[case("", "deceased must not be empty")]
    #[case("yes ", "Malformed deceased entry: 'yes '")]
    #[case("n/a", "Malformed deceased entry: 'n/a'")]
    fn test_invalid_deceased_field(#[case] item:&str, #[case] response:&str) {
        let duplet = DeceasedDuplet::default();
        let result = duplet.qc_cell(item);
        assert!(result.is_err());
        assert_eq!(response, result.unwrap_err().to_string());
    }

   

    #[rstest]
    #[case("yes")]
    #[case("no")]
    #[case("na")]
    fn test_valid_deceased_field(#[case] item:&str) {
        let duplet = DeceasedDuplet::default();
        let result = duplet.qc_cell(item);
        assert!(result.is_ok());
    }


    #[rstest]
    fn test_valid_ctor() {
        let duplet = DeceasedDuplet::from_table("deceased", "yes/no/na");
        assert!(duplet.is_ok());
    }

    #[rstest]
    #[case("age of onset", "str", "Malformed deceased Header: Expected 'deceased' but got 'age of onset'")]
    #[case("deceased ", "age", "Malformed deceased Header: Expected 'deceased' but got 'deceased '")]
    #[case("deceased", "str", "Malformed deceased Header: Expected 'yes/no/na' but got 'str'")]
    fn test_invalid_ctor(#[case] r1:&str, #[case] r2:&str, #[case] err_msg:&str) {
        let duplet = DeceasedDuplet::from_table(r1, r2);
        assert!(duplet.is_err());
        assert!(matches!(&duplet, Err(Error::HeaderError { .. })));
        assert_eq!(err_msg, duplet.unwrap_err().to_string());
    }

}

