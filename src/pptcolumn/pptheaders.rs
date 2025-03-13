///! The pyphetools template has two header lines. For the static fields, the information is only needed from the
/// first header. For the HPO columns, the label is shown in the first header and the HPO id is
/// shown in the second field. The purpose of this struct is simply to record the strings in
/// both rows so that we can do some Q/C prior to starting to create the DataFrame object.
/// 
use std::{collections::HashMap, fmt};

use ontolius::Identified;
use ontolius::term::MinimalTerm;
use ontolius::{ontology::csr::MinimalCsrOntology, term::simple::SimpleMinimalTerm, TermId};
use regex::Regex;

use crate::{disease_gene_bundle::DiseaseGeneBundle, hpo::hpo_term_arranger::HpoTermArranger};


/// The fields of row one and two of the pyphetools template file (i.e., the header)
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
/// The constant header fields for the first row of the pyphetools template file
static EXPECTED_H1_FIELDS: [&str; NUMBER_OF_CONSTANT_HEADER_FIELDS]= [
    "PMID", "title", "individual_id", "comment", "disease_id", "disease_label", 
    "HGNC_id", "gene_symbol", "transcript", "allele_1", "allele_2", "variant.comment", 
    "age_of_onset", "age_at_last_encounter", "deceased", "sex", "HPO"];
/// The constant header fields for the second row of the pyphetools template file
const EXPECTED_H2_FIELDS: [&str; NUMBER_OF_CONSTANT_HEADER_FIELDS]= [
    "CURIE", "str", "str", "optional", "CURIE", "str", 
    "CURIE",  "str", "str", "str", "str", "optional", 
    "age", "age", "yes/no/na", "M:F:O:U", "na"];

 /// PptHeader: Pyphetools Header - manage the generation of the first two rows of our template.
pub struct PptHeader;


impl PptHeader {
    pub fn get_header_duplets<'a>(&self, hpo_terms: &Vec<SimpleMinimalTerm>, 
                                        hpo:&'a MinimalCsrOntology) -> Result<Vec<HeaderDuplet>, Vec<String>> {
        let mut header_duplets: Vec<HeaderDuplet> = vec![];
        let mut errors: Vec<String> = Vec::new();
        for i in 0..NUMBER_OF_CONSTANT_HEADER_FIELDS {
            header_duplets.push(HeaderDuplet::new(EXPECTED_H1_FIELDS[i], EXPECTED_H2_FIELDS[i]));
        }
        // Arrange the HPO terms in a sensible order.
        let mut hpo_arranger = HpoTermArranger::new(hpo);
        let term_id_to_label_d: HashMap<TermId, String> = hpo_terms
            .iter()
            .map(|term| (term.identifier().clone(), term.name().to_string()))
            .collect();
        let term_ids: Vec<TermId> = term_id_to_label_d.keys().cloned().collect();
        let arranged_term_ids = hpo_arranger.arrange_terms(&term_ids);

        for tid in arranged_term_ids {
            let result = term_id_to_label_d.get(&tid);
            match result {
                Some(name) => header_duplets.push(HeaderDuplet::new(name, tid.to_string())),
                None => errors.push(format!("Could not get HPO term label for {}", tid)),
            }
        }
        if ! errors.is_empty() {
            return Err(errors);
        }
        if let Err(res) = self.qc_list_of_header_items(&header_duplets) {
            return Err(errors);
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

    /// When we first create the pyphetools template, we create the first two (header) lines
    /// and then we create 5 additional lines that are empty except for the constant parts
    /// i.e., information about the disease and disease gene that are constant in all lines
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

    pub fn get_initialized_matrix<'a>(&self, dg_bundle: DiseaseGeneBundle, 
                                        hpo_terms: &Vec<SimpleMinimalTerm>,
                                        hpo:&'a MinimalCsrOntology) -> 
                    Result<Vec<Vec<String>>, Vec<String>>   {
        let header_duplets = self.get_header_duplets(hpo_terms, hpo)?;
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
        for _ in 0..5 {
            matrix.push(self.get_empty_row(&dg_bundle, n_columns));
        }


        Ok(matrix)
    }


}


