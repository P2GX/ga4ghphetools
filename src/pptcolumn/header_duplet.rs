//! HeaderDuplet
//! 
//! The pyphetools template has two header rows
//! We refer to the two rows of one column as a header duplet.

use std::fmt;

use regex::Regex;


/// The HeaderDuplet represents the first two rows of the pyphetools template.
/// 
/// There are two header lines. For the static fields, the information is only needed from the
/// first header. For the HPO columns, the label is shown in the first header and the HPO id is
/// shown in the second field. The purpose of this struct is simply to record the strings in
/// both rows so that we can do some Q/C prior to starting to create the DataFrame object.
pub struct HeaderDuplet {
    h1: String,
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
}

impl fmt::Display for HeaderDuplet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "HeaderDuplet(h1: {}, h2: {})", self.h1, self.h2)
    }
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
pub fn qc_list_of_header_items(header_duplets: &Vec<HeaderDuplet>) -> Result<(), Vec<String>> {
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
