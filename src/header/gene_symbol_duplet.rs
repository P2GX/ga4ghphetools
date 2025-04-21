
//! GeneSymbolDuplet
//! The duplet and the QC routines for the PMID column
//! 

use std::cell;

use crate::template::curie;
use crate::header::header_duplet::HeaderDupletItem;
use crate::error::{self, Error, Result};

use super::header_duplet::{self, HeaderDuplet, HeaderDupletItemFactory};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct GeneSymbolDuplet {}

impl HeaderDupletItem for GeneSymbolDuplet {

    fn row1(&self) -> String {
        "gene_symbol".to_string()
    }

    fn row2(&self) -> String {
       "str".to_string()
    }

    fn qc_cell(&self, cell_contents: &str) -> Result<()> {
        header_duplet::check_empty(cell_contents)?;
        header_duplet::check_white_space(cell_contents)?;
        if cell_contents.contains(" ") {
            return Err(Error::HeaderError { msg: format!("Gene symbol must not contain whitespace: '{cell_contents}'") });
        }
        Ok(())
    }

    fn get_options(&self) -> Vec<String> {
        vec!["edit".to_string(), "trim".to_string()]
    }
}

impl HeaderDupletItemFactory for  GeneSymbolDuplet {
    fn from_table(row1: &str, row2: &str) -> Result<Self> where Self: Sized {
        let duplet = Self::default();
        if duplet.row1() != row1 {
            return Err(Error::HeaderError { msg: format!("Malformed gene_symbol Header: Expected '{}' but got '{}'", duplet.row1(), row1) });
        } 
        if duplet.row2() != row2 {
            return Err(Error::HeaderError { msg: format!("Malformed gene_symbol Header: Expected '{}' but got '{}'", duplet.row2(), row2) });
        }
        return Ok(duplet);
    }

    fn into_enum(self) -> super::header_duplet::HeaderDuplet {
        HeaderDuplet::GeneSymbolDuplet(self)
    }
}

impl GeneSymbolDuplet {
    pub fn new() -> Self {
        Self{}
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rstest::{fixture, rstest};

    #[rstest]
    #[case("FBN1 ","Trailing whitespace in 'FBN1 '")]
    #[case(" FBN1", "Leading whitespace in ' FBN1'")]
    #[case("FBN 1","Gene symbol must not contain whitespace: 'FBN 1'")]
    #[case("","Value must not be empty")]
    fn test_invalid_gene_symbol(#[case] item:&str, #[case] response:&str) {
        let duplet = GeneSymbolDuplet::default();
        let result = duplet.qc_cell(item);
        assert!(result.is_err());
        assert_eq!(response, result.unwrap_err().to_string());
    }

    #[rstest]
    #[case("FBN1")]
    #[case("NF1")]
    fn test_valid_gene_symbol(#[case] item:&str) {
        let duplet = GeneSymbolDuplet::default();
        let result = duplet.qc_cell(item);
        assert!(result.is_ok());
    }


    #[rstest]
    fn test_valid_ctor() {
        let duplet = GeneSymbolDuplet::from_table("gene_symbol", "str");
        assert!(duplet.is_ok());
    }

    #[rstest]
    #[case("gene", "str", "Malformed gene_symbol Header: Expected 'gene_symbol' but got 'gene'")]
    #[case("gene_symbol ", "CURIE", "Malformed gene_symbol Header: Expected 'gene_symbol' but got 'gene_symbol '")]
    #[case("gene_symbol", "CURIE", "Malformed gene_symbol Header: Expected 'str' but got 'CURIE'")]
    fn test_invalid_ctor(#[case] r1:&str, #[case] r2:&str, #[case] err_msg:&str) {
        let duplet = GeneSymbolDuplet::from_table(r1, r2);
        assert!(duplet.is_err());
        assert!(matches!(&duplet, Err(Error::HeaderError { .. })));
        assert_eq!(err_msg, duplet.unwrap_err().to_string());
    }

}





