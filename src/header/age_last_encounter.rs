
//! AgeLastEncounterDuplet
//! The duplet and the QC routines for the PMID column
//! 

use crate::template::curie;
use crate::header::header_duplet::HeaderDupletItem;
use crate::error::{self, Error, Result};
use crate::header::age_util;

use super::header_duplet::{self, HeaderDuplet, HeaderDupletItemFactory};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct AgeLastEncounterDuplet {}

impl HeaderDupletItem for AgeLastEncounterDuplet {
    fn row1(&self) -> String {
        "age_at_last_encounter".to_string()
    }

    fn row2(&self) -> String {
        "age".to_string()
    }

    fn qc_cell(&self, cell_contents: &str) -> Result<()> {
        header_duplet::check_white_space_for_field(cell_contents, "age_at_last_encounter")?;
        header_duplet::check_empty_for_field(cell_contents, "age_at_last_encounter")?;
        if age_util::is_valid_age_string(cell_contents) {
            Ok(())
        } else {
            Err(Error::AgeParseError { msg: format!("Malformed age_at_last_encounter '{}'", cell_contents) })
        }
    }

    

}

impl HeaderDupletItemFactory for AgeLastEncounterDuplet {
    fn from_table(row1: &str, row2: &str) -> Result<Self> where Self: Sized {
        let duplet = Self::default();
        if duplet.row1() != row1 {
            return Err(Error::HeaderError { msg: format!("Malformed age_at_last_encounter Header: Expected '{}' but got '{}'", duplet.row1(), row1) });
        } else if duplet.row2() != row2 {
            return Err(Error::HeaderError { msg: format!("Malformed age_at_last_encounter Header: Expected '{}' but got '{}'", duplet.row2(), row2) });
        } else {
            return Ok(duplet);
        }
    }

    fn into_enum(self) -> super::header_duplet::HeaderDuplet {
        HeaderDuplet::AgeLastEncounterDuplet(self)
    }
}

impl AgeLastEncounterDuplet {
    pub fn new() -> Self {
        Self{}
    }
}
/// Note: Except for CTOR, items are identical as for age_of_onset, so we only test the CTOR
#[cfg(test)]
mod test {
    use std::result;

    use super::*;
    use rstest::{fixture, rstest};

    #[rstest]
    fn test_valid_ctor() {
        let duplet = AgeLastEncounterDuplet::from_table("age_at_last_encounter", "age");
        assert!(duplet.is_ok());
    }

    #[rstest]
    #[case("age of onset", "str", "Malformed age_at_last_encounter Header: Expected 'age_at_last_encounter' but got 'age of onset'")]
    #[case("age_at_last_encounter ", "age", "Malformed age_at_last_encounter Header: Expected 'age_at_last_encounter' but got 'age_at_last_encounter '")]
    #[case("age_at_last_encounter", "str", "Malformed age_at_last_encounter Header: Expected 'age' but got 'str'")]
    fn test_invalid_ctor(#[case] r1:&str, #[case] r2:&str, #[case] err_msg:&str) {
        let duplet = AgeLastEncounterDuplet::from_table(r1, r2);
        assert!(duplet.is_err());
        assert!(matches!(&duplet, Err(Error::HeaderError { .. })));
        assert_eq!(err_msg, duplet.unwrap_err().to_string());
    }
}