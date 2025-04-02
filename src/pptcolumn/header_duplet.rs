//! HeaderDuplet
//! 
//! The pyphetools template has two header rows
//! We refer to the two rows of one column as a header duplet.

use std::fmt;
use regex::Regex;
use lazy_static::lazy_static;
use crate::error::{self, Error, Result};


/// The HeaderDuplet represents the first two rows of the pyphetools template.
/// 
/// There are two header lines. For the static fields, the information is only needed from the
/// first header. For the HPO columns, the label is shown in the first header and the HPO id is
/// shown in the second field. The purpose of this struct is simply to record the strings in
/// both rows so that we can do some Q/C prior to starting to create the DataFrame object.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HeaderDuplet {
    /// field in the first row
    h1: String,
    /// field in the second row
    h2: String,
}



impl HeaderDuplet {

    pub fn new<S, T>(header1: S , header2: T) -> Self 
        where S: Into<String>, T: Into<String>
       {
        HeaderDuplet {
            h1: header1.into(),
            h2: header2.into(),
        }
    }

    pub fn row1(&self) -> String {
        self.h1.clone()
    }

    pub fn row2(&self) -> String {
        self.h2.clone()
    }


    pub fn extract_from_string_matrix(matrix: &Vec<Vec<String>>) -> Result<Vec<HeaderDuplet>> {
        if matrix.len() < 2 {
            return Err(Error::TemplateError { msg: format!("Insuffient rows ({}) to construct header duplets", matrix.len()) });
        }
        let row_len = matrix[0].len();
        let mut header_duplet_list: Vec<HeaderDuplet> = Vec::new();
        for i in 0..row_len {
            let hdup = HeaderDuplet::new(matrix[0][i].clone(), matrix[1][i].clone());
            header_duplet_list.push(hdup);
        }
        Ok(header_duplet_list)
    }
}

impl fmt::Display for HeaderDuplet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "HeaderDuplet(h1: {}, h2: {})", self.h1, self.h2)
    }
}


const NUMBER_OF_CONSTANT_HEADER_FIELDS_MENDELIAN: usize = 17; 

lazy_static! {
    static ref PMID_HEADER: HeaderDuplet = HeaderDuplet::new("PMID", "CURIE");
    static ref TITLE_HEADER: HeaderDuplet = HeaderDuplet::new("title", "str");
    static ref INDIVIDUAL_ID_HEADER: HeaderDuplet = HeaderDuplet::new("individual_id", "str");
    static ref COMMENT_HEADER: HeaderDuplet = HeaderDuplet::new("comment", "optional");
    static ref DISEASE_ID_HEADER: HeaderDuplet = HeaderDuplet::new("disease_id", "CURIE");
    static ref DISEASE_LABEL_HEADER: HeaderDuplet = HeaderDuplet::new("disease_label", "str");
    static ref HGNC_ID_HEADER: HeaderDuplet = HeaderDuplet::new("HGNC_id", "CURIE");
    static ref GENE_SYMBOL_HEADER: HeaderDuplet = HeaderDuplet::new("gene_symbol", "str");
    static ref TRANSCRIPT_HEADER: HeaderDuplet = HeaderDuplet::new("transcript", "str");
    static ref ALLELE_1_HEADER: HeaderDuplet = HeaderDuplet::new("allele_1", "str");
    static ref ALLELE_2_HEADER: HeaderDuplet = HeaderDuplet::new("allele_2", "str");
    static ref VARIANT_COMMENT_HEADER: HeaderDuplet = HeaderDuplet::new("variant.comment", "optional");
    static ref AGE_OF_ONSET_HEADER: HeaderDuplet = HeaderDuplet::new("age_of_onset", "age");
    static ref AGE_AT_LAST_ECOUNTER_HEADER: HeaderDuplet = HeaderDuplet::new("age_at_last_encounter", "age");
    static ref DECEASED_HEADER: HeaderDuplet = HeaderDuplet::new("deceased", "yes/no/na");
    static ref SEX_HEADER: HeaderDuplet = HeaderDuplet::new("sex", "M:F:O:U");
    static ref HPO_SEPARATOR_HEADER: HeaderDuplet = HeaderDuplet::new("HPO", "na");

   
}


fn expected_mendelian_fields() -> Vec<&'static HeaderDuplet> {
    vec![
        &PMID_HEADER, &TITLE_HEADER, &INDIVIDUAL_ID_HEADER, &COMMENT_HEADER,  
        &DISEASE_ID_HEADER, &DISEASE_LABEL_HEADER, &HGNC_ID_HEADER, &GENE_SYMBOL_HEADER, 
        &TRANSCRIPT_HEADER, &ALLELE_1_HEADER, &ALLELE_2_HEADER, &VARIANT_COMMENT_HEADER, 
        &AGE_OF_ONSET_HEADER, &AGE_AT_LAST_ECOUNTER_HEADER, &DECEASED_HEADER, 
        &SEX_HEADER, &HPO_SEPARATOR_HEADER
    ]
}





/// These fields are always required by our template
const NUMBER_OF_CONSTANT_HEADER_FIELDS: usize = 17; 
/// The constant header fields for the first row of the pyphetools template file
static EXPECTED_H1_FIELDS: [&str; NUMBER_OF_CONSTANT_HEADER_FIELDS]= [
    "PMID", "title", "individual_id", "comment", "disease_id", "disease_label", 
    "HGNC_id", "gene_symbol", "transcript", "allele_1", "allele_2", "variant.comment", 
    "age_of_onset", "age_at_last_encounter", "deceased", "sex", "HPO"
];
/// The constant header fields for the second row of the pyphetools template file
const EXPECTED_H2_FIELDS: [&str; NUMBER_OF_CONSTANT_HEADER_FIELDS]= [
    "CURIE", "str", "str", "optional", "CURIE", "str", 
    "CURIE",  "str", "str", "str", "str", "optional", 
    "age", "age", "yes/no/na", "M:F:O:U", "na"
];


 /// perform quality control of the two header rows of a pyphetools template file
pub fn qc_list_of_header_items(header_duplets: &Vec<HeaderDuplet>) -> core::result::Result<(), Vec<String>> {
    // check each of the items in turn

    let mut errors: Vec<String> = vec![];
    for (i, duplet) in header_duplets.into_iter().enumerate() {
        if i < NUMBER_OF_CONSTANT_HEADER_FIELDS && duplet.h1 != EXPECTED_H1_FIELDS[i] {
            errors.push(format!("Malformed header: expected {}, got {}", 
                            EXPECTED_H1_FIELDS[i], 
                            duplet.h1))
        } 
        if i < NUMBER_OF_CONSTANT_HEADER_FIELDS && duplet.h2 != EXPECTED_H2_FIELDS[i] {
            errors.push(format!("Malformed header (row 2): expected {}, got {}", 
                            EXPECTED_H2_FIELDS[i], 
                            duplet.h1))
        } 
        // for the following fields, we are in the HPO columns
        // these columns are different for each template. The first row contains the term label
        // and the second row contains the HPO term id. We just do some basic format checks
        if i > NUMBER_OF_CONSTANT_HEADER_FIELDS {
            if duplet.h1.starts_with(|c: char| c.is_whitespace()) {
                errors.push(format!("Column {}: Term label '{}' starts with whitespace", i, duplet.h1));
            }
            if duplet.h1.ends_with(|c: char| c.is_whitespace()) {
                errors.push(format!("Column {}: Term label '{}' ends with whitespace", i, duplet.h1));
            }
            let re = Regex::new(r"^HP:\d{7}$").unwrap();
            if ! re.is_match(&duplet.h2) {
                errors.push(format!("Column {}: Invalid HPO id '{}'", i, duplet.h2));
            }
        }
    }
    if errors.len() > 0 {
        return Err(errors);
    }
    Ok(())
}


// region:    --- Tests

#[cfg(test)]
mod tests {
    type Error = Box<dyn std::error::Error>;
    type Result<T> = core::result::Result<T, Error>; // For tests.

    use super::*;

    #[test]
    fn test_ctor() -> Result<()> {
        let hdup_a = HeaderDuplet::new("Title", "str");
        let hdup_b =  HeaderDuplet::new("Title", "str");
        let hdup_c =  HeaderDuplet::new("HGNC", "CURIE");
        assert_eq!(hdup_a, hdup_b);
        assert_ne!(hdup_a, hdup_c);
    
        Ok(())
    }
}

// endregion: --- Tests