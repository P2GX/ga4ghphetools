//! TitleDuplet
//! The duplet and the QC routines for the title column
//! 


use crate::header::header_duplet::HeaderDupletItem;
use crate::error::{self, Error, Result};

use super::header_duplet::{self, HeaderDuplet, HeaderDupletItemFactory};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct TitleDuplet {}




impl HeaderDupletItem for TitleDuplet {
    fn row1(&self) -> String {
        "title".to_string()
    }

    fn row2(&self) -> String {
        "str".to_string()
    }

    fn qc_cell(&self, cell_contents: &str) -> crate::error::Result<()> {
        header_duplet::check_white_space(cell_contents)?;
        header_duplet::check_empty(cell_contents)?;
        Ok(())
    }
    
    
    
}
impl HeaderDupletItemFactory for TitleDuplet {
    fn from_table(row1: &str, row2: &str) -> crate::error::Result<Self> where Self: Sized {
        let duplet = Self{};
        if duplet.row1() != row1 {
            return Err(Error::HeaderError { msg: format!("Malformed title Header: Expected '{}' but got '{}'", duplet.row1(), row1) });
        } else if duplet.row2() != row2 {
            return Err(Error::HeaderError { msg: format!("Malformed title Header: Expected '{}' but got '{}'", duplet.row2(), row2) });
        } else {
            return Ok(duplet);
        }
    }
    fn into_enum(self) -> super::header_duplet::HeaderDuplet {
        HeaderDuplet::TitleDuplet(self)
    }
}



impl TitleDuplet {
    pub fn new() -> Self {
        TitleDuplet{}
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rstest::{fixture, rstest};


    #[rstest]
    #[case("From transcriptomics to digital twins of organ function ", "Trailing whitespace in 'From transcriptomics to digital twins of organ function '")]
    #[case(" From transcriptomics to digital twins of organ function", "Leading whitespace in ' From transcriptomics to digital twins of organ function'")]
    fn test_invalid_title(#[case] item:&str, #[case] response:&str) {
        let pmid_hd = TitleDuplet{};
        let pmid = pmid_hd.qc_cell(item);
        assert!(pmid.is_err());
        assert_eq!(response, pmid.unwrap_err().to_string());
    }

    #[rstest]
    #[case("From transcriptomics to digital twins of organ function")]
    fn test_valid_title(#[case] item:&str) {
        let pmid_hd = TitleDuplet{};
        let pmid = pmid_hd.qc_cell(item);
        assert!(pmid.is_ok());
    }


    #[rstest]
    fn test_valid_ctor() {
        let duplet = TitleDuplet::from_table("title", "str");
        assert!(duplet.is_ok());
    }

    #[rstest]
    #[case("Title", "str", "Malformed title Header: Expected 'title' but got 'Title'")]
    #[case("title ", "str", "Malformed title Header: Expected 'title' but got 'title '")]
    #[case("title", "CURIE", "Malformed title Header: Expected 'str' but got 'CURIE'")]
    fn test_invalid_ctor(#[case] r1:&str, #[case] r2:&str, #[case] err_msg:&str) {
        let duplet = TitleDuplet::from_table(r1, r2);
        assert!(duplet.is_err());
        assert!(matches!(&duplet, Err(Error::HeaderError { .. })));
        assert_eq!(err_msg, duplet.unwrap_err().to_string());
    }

}