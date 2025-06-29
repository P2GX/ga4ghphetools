
//! HpoTermDuplet
//! The duplet and the QC routines for the PMID column
//! 

use std::collections::HashSet;
use std::str::FromStr;
use lazy_static::lazy_static;
use ontolius::TermId;

use crate::dto::template_dto::{HeaderDupletDto};
use crate::template::curie;
use crate::error::{self, Error, Result};
use crate::hpo::age_util;




#[derive(Clone, Debug, Default, PartialEq)]
pub struct HpoTermDuplet {
    pub(crate) hpo_label: String,
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
    fn malformed_hpo_entry(hpo_label: &str, hpo_id: &str, value: &str) -> Self {
        Error::HpoError { msg: format!("Malformed entry for {} ({}): '{}'", hpo_label, hpo_id, value) } 
    }

    fn malformed_hpo_term_id( value: &str) -> Self {
        Error::HpoError { msg: format!("Malformed HPO Term id: '{}'", value) } 
    }
}

impl HpoTermDuplet {
    pub fn new(label: impl Into<String>, identifier: impl Into<String>) -> Self {
        Self { hpo_label: label.into(), hpo_id: identifier.into() }
    }

    pub fn to_header_dto(&self) -> HeaderDupletDto {
        HeaderDupletDto::new(&self.hpo_label, &self.hpo_id)
    }

    pub fn from_header_dto(dto: HeaderDupletDto) -> Self {
        Self { 
            hpo_label: dto.h1, 
            hpo_id: dto.h2 
        }
    }

    pub fn row1(&self) -> String {
        self.hpo_label.clone()
    }

    pub fn row2(&self) -> String {
        self.hpo_id.clone()
    }

    pub fn hpo_id(&self) -> &str {
        &self.hpo_id
    }

    pub fn hpo_label(&self) -> &str {
        &self.hpo_label
    }

    pub fn to_term_id(&self) -> std::result::Result<TermId, String> {
        let tid = TermId::from_str(&self.hpo_id).map_err(|_| format!("Could not create TermId from {}", self.hpo_id()))?;
        Ok(tid)
    }
    
}

impl  HpoTermDuplet {
   

    /// An HPO cell can be empty, or contain observed/expected/na or an age string
    /// We plan to enforce that HPO cells cannot be empty (they will need to have na for not-available data)
    /// We also plan to allow some modifiers (e.g., "Mild") in this field, indicating "observed" with modifier
    fn qc_cell(&self, cell_contents: &str) -> Result<()> {
        if cell_contents.is_empty() {
            return Ok(());
        }
        if ALLOWED_HPO_ITEMS.contains(cell_contents) {
            return Ok(());
        }
        if age_util::is_valid_age_string(cell_contents) {
            return Ok(());
        }
        Err(Error::malformed_hpo_entry(&self.row1(), &self.row2(), cell_contents))
    }

    fn get_options(&self) -> Vec<String> {
        vec!["observed".to_string(), "excluded".to_string(), "na".to_string(), "edit".to_string()]
    }

    /// Change the value of one of the two header items for an HPO column
    /// The first row has the label and the second row has the HPO id. We allow this to be edited.
    /// We assume that the caller is provide the correct value and do not check here that it is a valid term id/label
    /// This Q/C occurs in multiple other places of the application.
    fn set_value(&mut self, idx: usize, value: &str) -> Result<()> {
       /* if idx == 0 {
            header_duplet::check_empty(value)?;
            header_duplet::check_leading_trailing_whitespace(value)?;
            self.hpo_label = value.to_string();
        } else if idx == 1 {
            header_duplet::check_empty(value)?;
            header_duplet::check_valid_curie(value)?;
            if ! value.starts_with("HP:") && value.len() == 10 {
                return Err(Error::malformed_hpo_term_id(value));
            }
            self.hpo_id = value.to_string();
        } else {
            return Err(Error::HeaderError { msg: format!("invalid index for HPO header: {idx}") });
        }*/ 
        eprint!("REFACOT ME hpo_Term_duplet");
        Ok(())
    }

}



#[cfg(test)]
mod test {
    use std::result;

    use super::*;
    use rstest::{fixture, rstest};

/*
    #[rstest]
    #[case("na ", "Malformed entry for Parasomnia (HP:0025234): 'na '")]
    fn test_invalid_hpo_field(#[case] item:&str, #[case] response:&str) {
        let duplet = HpoTermDuplet::new("Parasomnia", "HP:0025234");
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
        let duplet = HpoTermDuplet::new("Arachnodactyly", "HP:0001166");
        assert!(duplet.is_ok());
    }

    #[rstest]
    #[case("Arachnodactyly", "0001166", "Invalid CURIE with no colon: '0001166'")]
    fn test_invalid_ctor(#[case] r1:&str, #[case] r2:&str, #[case] err_msg:&str) {
        let duplet = HpoTermDuplet::new(r1, r2);
        assert!(duplet.is_err());
        // TODO -- what kind of error? -- assert!(matches!(&duplet, Err(Error::HeaderError { .. })));
        assert_eq!(err_msg, duplet.unwrap_err().to_string());
    }
 */
}


