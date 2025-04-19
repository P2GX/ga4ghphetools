
//! HpoTermDuplet
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
pub struct HpoTermDuplet {
    hpo_label: String,
    hpo_id: String,
}


lazy_static! {
    pub static ref ALLOWED_HPO_ITEMS: HashSet<String> =  {
        let mut set = HashSet::new();
        set.insert("na".to_string());
        set.insert("observed".to_string());
        set.insert("excluded".to_string());
        set
    };
}

impl Error {
    fn malformed_hpo_entry(value: &str) -> Self {
        Error::HpoError { msg: format!("Malformed HPO entry: '{}'", value) } 
    }
}

impl HpoTermDuplet {
    pub fn new(label: impl Into<String>, identifier: impl Into<String>) -> Self {
        Self { hpo_label: label.into(), hpo_id: identifier.into() }
    }
}

impl HeaderDupletItem for HpoTermDuplet {
    fn row1(&self) -> String {
        self.hpo_label.clone()
    }

    fn row2(&self) -> String {
        self.hpo_id.clone()
    }

    fn qc_cell(&self, cell_contents: &str) -> Result<()> {
        if ALLOWED_HPO_ITEMS.contains(cell_contents) {
            return Ok(());
        }
        if age_util::is_valid_age_string(cell_contents) {
            return Ok(());
        }
        Err(Error::malformed_hpo_entry(cell_contents))
    }

    
}

impl HeaderDupletItemFactory for HpoTermDuplet {
    fn from_table(row1: &str, row2: &str) -> Result<Self> where Self: Sized {
        header_duplet::check_empty(row1)?;
        header_duplet::check_white_space(row1)?;
        header_duplet::check_valid_curie(row2)?;
        if ! row2.starts_with("HP:") {
            return Err(Error::malformed_hpo_entry(row2));
        }
        let duplet = Self::new(row1, row2);
        return Ok(duplet);
    }

    fn into_enum(self) -> super::header_duplet::HeaderDuplet {
        HeaderDuplet::HpoTermDuplet(self)
    }
}


#[cfg(test)]
mod test {
    use std::result;

    use super::*;
    use rstest::{fixture, rstest};


    #[rstest]
    #[case("na ", "Malformed HPO entry: 'na '")]
    fn test_invalid_hpo_field(#[case] item:&str, #[case] response:&str) {
        let duplet = HpoTermDuplet::default();
        let result = duplet.qc_cell(item);
        assert!(result.is_err());
        assert_eq!(response, result.unwrap_err().to_string());
    }

   

    #[rstest]
    #[case("na")]
    #[case("observed")]
    #[case("excluded")]
    #[case("P32Y4M")]
    fn test_valid_hpo_field(#[case] item:&str) {
        let duplet = HpoTermDuplet::default();
        let result = duplet.qc_cell(item);
        assert!(result.is_ok());
    }


    #[rstest]
    fn test_valid_ctor() {
        let duplet = HpoTermDuplet::from_table("Arachnodactyly", "HP:0001166");
        assert!(duplet.is_ok());
    }

    #[rstest]
    #[case("Arachnodactyly", "0001166", "Invalid CURIE with no colon: '0001166'")]
    fn test_invalid_ctor(#[case] r1:&str, #[case] r2:&str, #[case] err_msg:&str) {
        let duplet = HpoTermDuplet::from_table(r1, r2);
        assert!(duplet.is_err());
        // TODO -- what kind of error? -- assert!(matches!(&duplet, Err(Error::HeaderError { .. })));
        assert_eq!(err_msg, duplet.unwrap_err().to_string());
    }

}


