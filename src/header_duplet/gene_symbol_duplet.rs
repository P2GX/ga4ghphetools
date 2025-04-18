
//! GeneSymbolDuplet
//! The duplet and the QC routines for the PMID column
//! 

use std::cell;

use crate::template::curie;
use crate::header_duplet::header_duplet::HeaderDupletItem;
use crate::error::{self, Error, Result};


#[derive(Debug, Default)]
pub struct GeneSymbolDuplet {}

impl HeaderDupletItem for GeneSymbolDuplet {

    fn row1(&self) -> String {
        "gene_symbol".to_string()
    }

    fn row2(&self) -> String {
       "str".to_string()
    }

    fn qc_cell(&self, cell_contents: &str) -> Result<()> {
        Self::check_empty(cell_contents)?;
        Self::check_white_space(cell_contents)?;
        if cell_contents.contains(" ") {
            return Err(Error::HeaderError { msg: format!("Gene symbol must not contain whitespace: '{cell_contents}'") });
        }
        Ok(())
    }

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





