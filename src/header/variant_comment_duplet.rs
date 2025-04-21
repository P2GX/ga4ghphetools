
//! VariantCommentDuplet
//! The duplet and the QC routines for the PMID column
//! 

use crate::template::curie;
use crate::header::header_duplet::HeaderDupletItem;
use crate::error::{self, Error, Result};

use super::header_duplet::{self, HeaderDuplet, HeaderDupletItemFactory};


#[derive(Clone, Debug, Default, PartialEq)]
pub struct VariantCommentDuplet {}

impl HeaderDupletItem for VariantCommentDuplet {
    fn row1(&self) -> String {
       "variant.comment".to_string()
    }

    fn row2(&self) -> String {
        "optional".to_string()
    }

    fn qc_cell(&self, cell_contents: &str) -> Result<()> {
        header_duplet::check_tab(cell_contents)?;
        Ok(())
    }

    fn get_options(&self) -> Vec<String> {
        vec!["edit".to_string(), "trim".to_string()]
    }
    
}

impl HeaderDupletItemFactory for VariantCommentDuplet {
    fn from_table(row1: &str, row2: &str) -> Result<Self> where Self: Sized {
        let duplet = Self::default();
        if duplet.row1() != row1 {
            return Err(Error::HeaderError { msg: format!("Malformed variant.comment Header: Expected '{}' but got '{}'", duplet.row1(), row1) });
        } else if duplet.row2() != row2 {
            return Err(Error::HeaderError { msg: format!("Malformed variant.comment Header: Expected '{}' but got '{}'", duplet.row2(), row2) });
        } else {
            return Ok(duplet);
        }
    }
    
    fn into_enum(self) -> super::header_duplet::HeaderDuplet {
        HeaderDuplet::VariantCommentDuplet(self)
    }
}

impl VariantCommentDuplet {
    pub fn new() -> Self {
        VariantCommentDuplet{}
    }
}

#[cfg(test)]
mod test {
    use std::result;

    use super::*;
    use rstest::{fixture, rstest};


    #[rstest]
    #[case("Some comment about the\tpaper", "Value must not contain a tab character")]
    fn test_invalid_comment(#[case] item:&str, #[case] response:&str) {
        let duplet = VariantCommentDuplet::default();
        let result = duplet.qc_cell(item);
        assert!(result.is_err());
        assert_eq!(response, result.unwrap_err().to_string());
    }

    #[rstest]
    #[case("From transcriptomics to digital twins of organ function")]
    fn test_valid_comment(#[case] item:&str) {
        let duplet = VariantCommentDuplet::default();
        let result = duplet.qc_cell(item);
        assert!(result.is_ok());
    }


    #[rstest]
    fn test_valid_ctor() {
        let duplet = VariantCommentDuplet::from_table("variant.comment", "optional");
        assert!(duplet.is_ok());
    }

    #[rstest]
    #[case("Variant.Comment", "str", "Malformed variant.comment Header: Expected 'variant.comment' but got 'Variant.Comment'")]
    #[case("variant.comment ", "optional", "Malformed variant.comment Header: Expected 'variant.comment' but got 'variant.comment '")]
    #[case("optional", "comment", "Malformed variant.comment Header: Expected 'variant.comment' but got 'optional'")]
    fn test_invalid_ctor(#[case] r1:&str, #[case] r2:&str, #[case] err_msg:&str) {
        let duplet = VariantCommentDuplet::from_table(r1, r2);
        assert!(duplet.is_err());
        assert!(matches!(&duplet, Err(Error::HeaderError { .. })));
        assert_eq!(err_msg, duplet.unwrap_err().to_string());
    }

}

