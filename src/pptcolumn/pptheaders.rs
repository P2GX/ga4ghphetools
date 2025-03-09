///! The pyphetools template has two header lines. For the static fields, the information is only needed from the
/// first header. For the HPO columns, the label is shown in the first header and the HPO id is
/// shown in the second field. The purpose of this struct is simply to record the strings in
/// both rows so that we can do some Q/C prior to starting to create the DataFrame object.
/// 
use std::fmt;

use ontolius::{base::{term::simple::SimpleMinimalTerm, Identified}, prelude::MinimalTerm};

use crate::{disease_gene_bundle::DiseaseGeneBundle, hpo};
pub struct HeaderDuplet {
    h1: String,
    h2: String,
}

impl HeaderDuplet {

    pub fn new<S: Into<String>, T: Into<String>>(header1: S ,  header2: T) -> Self {
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
static EXPECTED_H1_FIELDS: [&str; NUMBER_OF_CONSTANT_HEADER_FIELDS]= ["PMID", "title", "individual_id", "comment", "disease_id", 
"disease_label", "HGNC_id", "gene_symbol", "transcript", "allele_1", "allele_2", 
"variant.comment", "age_of_onset", "age_at_last_encounter", "deceased", "sex", "HPO"];
const EXPECTED_H2_FIELDS: [&str; NUMBER_OF_CONSTANT_HEADER_FIELDS]= ["CURIE", "str", "str", "optional", "CURIE", "str", "CURIE", 
 "str", "str", "str", "str", "optional", "age", "age", "yes/no/na", "M:F:O:U", "na"];

pub struct PptHeader;





impl PptHeader {
    pub fn getHeaderDuplets(&self, hpo_terms: &Vec<SimpleMinimalTerm>) -> Result<Vec<HeaderDuplet>, Vec<String>> {
        let mut header_duplets: Vec<HeaderDuplet> = vec![];
        for i in 0..NUMBER_OF_CONSTANT_HEADER_FIELDS {
            header_duplets.push(HeaderDuplet::new(EXPECTED_H1_FIELDS[i], EXPECTED_H2_FIELDS[i]));
        }
        for term in hpo_terms {
            let hpo_id = term.identifier().to_string();
            let hpo_label = term.name().to_string();
            header_duplets.push(HeaderDuplet::new(term.name(), term.identifier().to_string()));
        }
        if let Err(res) = self.qc_list_of_header_items(&header_duplets) {
            return Err(res);
        } else {
            return Ok(header_duplets);
        }
    }

    
    /// perform quality control of the two header rows of a pyphetools template file
    pub fn qc_list_of_header_items(&self, header_duplets: &Vec<HeaderDuplet>) -> Result<(), Vec<String>> {
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
            if i > NUMBER_OF_CONSTANT_HEADER_FIELDS {
                break;
            }
        }
        if errors.len() > 0 {
            return Err(errors);
        }
        Ok(())
    }

    /// 	disease_id	disease_label	HGNC_id	gene_symbol	transcript	
    /// allele_1	allele_2	variant.comment	age_of_onset	age_at_last_encounter	deceased	sex	HPO
    fn get_empty_row(&self, dg_bundle: &DiseaseGeneBundle, row_len: usize) -> Vec<String> {
        let mut row = vec![];
        for _ in 0..4 {
            row.push(String::new()); // empty PMID, title, individual_id, comment
        }
        row.push(dg_bundle.disease_id_as_string());
        row.push(dg_bundle.disease_name());
        row.push(dg_bundle.hgnc_id_as_string());
        row.push(dg_bundle.gene_symbol());
        row.push(dg_bundle.transcript());
        for _ in 0..7 {
             // allele_1,  allele_2, variant.comment, age_of_onset, age_at_last_encounter, deceased,sex	
            row.push(String::new());  
        }
        row.push("na".to_string()); // the HPO column has "na" as a demarker
        // calculate numer of remaining columns (where the HPO terms go)
        let remaining_columns = row_len - NUMBER_OF_CONSTANT_HEADER_FIELDS;
        for _ in 0..remaining_columns {
            row.push(String::new());  
        }


        row
    }

    pub fn get_initialized_matrix(&self, dg_bundle: DiseaseGeneBundle, hpo_terms: &Vec<SimpleMinimalTerm>) -> 
                    Result<Vec<Vec<String>>, Vec<String>>   {
        let header_duplets = self.getHeaderDuplets(hpo_terms)?;
        self.qc_list_of_header_items(&header_duplets)?;
        let mut errors = vec![];
        let mut matrix: Vec<Vec<String>> = vec![];
        // initialize the first two rows
        let n_columns = header_duplets.len();
        let mut row1 = vec![];
        let mut row2 = vec![];
        for i in 0..n_columns {
            match header_duplets.get(i) {
                Some(hd) => {
                    row1.push(hd.row1());
                    row2.push(hd.row2());
                },
                None => {
                    errors.push(format!("Could not retrieve header for index {}", i));
                }
            }
        }
        matrix.push(row1);
        matrix.push(row2);
        // We add a default value of 5 additional rows
        for i in 0..5 {
            matrix.push(self.get_empty_row(&dg_bundle, n_columns));
        }


        Ok(matrix)
    }


}