//! This module contains utilities for the initial input and quality control of the table cells
//!
//! Each table cell is modelled as having the ability to return a datatype and the contents as a String
//! We garantee that if these objects are created, then we are ready to create phenopackets.

use std::collections::HashMap;
use std::fmt::{self, format};

use std::{collections::HashSet};

use crate::curie::Curie;
use crate::hpo::{SimpleHPO, HPO};
use crate::main;
use crate::simple_label::SimpleLabel;
use crate::hpo_term_template::{HpoTemplate, HpoTemplateFactory, HpoTermStatus};
use crate::onset::Onset;




/// There are two header lines. For the static fields, the information is only needed from the
/// first header. For the HPO columns, the label is shown in the first header and the HPO id is
/// shown in the second field. The purpose of this struct is simply to record the strings in
/// both rows so that we can do some Q/C prior to starting to create the DataFrame object.
struct HeaderDuplet {
    h1: String,
    h2: String,
}

impl HeaderDuplet {

    pub fn new(header1: &str ,  header2: &str) -> Self {
        HeaderDuplet {
            h1: header1.to_string(),
            h2: header2.to_string(),
        }
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

#[derive(Debug)]
pub enum TableCellDataType {
    Title(TitleCell),
    PMID(Curie),
}



pub trait TableCell {
    fn value(&self) -> String;
}


pub struct HeaderItem {

}

/// This struct represents the contents of a cell of the Excel table that represents the title of a publication
#[derive(Debug)]
pub struct TitleCell {
    title: String,
}

impl TitleCell {
    pub fn new(title: &str) -> Result<Self, String> {
        if title.is_empty() {
            return Err("Title field is empty".to_string())
        } else if title.chars().last().map_or(false, |c| c.is_whitespace()) {
            return Err(format!("Title '{}' ends with whitespace", title));
        } else if title.chars().next().map_or(false, |c| c.is_whitespace()) {
            return Err(format!("Title '{}' begins with whitepsace", title));
        } 
        Ok(TitleCell { title: title.to_string(), })
    }
}

impl TableCell for TitleCell {
    fn value(&self) -> String {
        self.title.clone()
    }
}



pub struct IndividualTemplate {
    title: TitleCell,
    pmid: Curie,
    individual_id: SimpleLabel,
    disease_id: Curie,
    disease_label: SimpleLabel,
    hgnc_id: Curie,
    gene_symbol: SimpleLabel
}


impl IndividualTemplate {
   
   pub fn new(title: TitleCell, 
                pmid: Curie,
                individualId: SimpleLabel,
                diseaseId: Curie,
                diseaseLabel: SimpleLabel,
                hgnc: Curie,
                gene_sym: SimpleLabel) -> Self {
                    IndividualTemplate {
                        title: title,
                        pmid: pmid,
                        individual_id: individualId,
                        disease_id: diseaseId,
                        disease_label: diseaseLabel,
                        hgnc_id: hgnc,
                        gene_symbol: gene_sym,
                    }
                }

}


/// This struct sets up code to generate the IndividualtemplateRow objects that we will
/// use to generate phenopacket code. Each IndivudalTemplateRow object is an intermediate
/// object in which we have performed sufficient quality control to know that we are able
/// to create a valid phenopacket. The IndividualTemplateFactory sets up code that leverages
/// the data in the first two rows of the template to generate an IndivudalTemplateRow from
/// each of the subsequent rows of the Excel file. We treat the constant columns with constructors (new functions)
/// that perform Q/C. The HPO columns require somewhat more functionality and use HpoTemplateFactory,
/// one for each column.
pub struct IndividualTemplateFactory {
    hpo: SimpleHPO,
    expected_n_fields: usize,
    index_to_hpo_factory_d: HashMap<usize, HpoTemplateFactory>,
}

impl IndividualTemplateFactory {
    pub fn new (
            hpo_json_path: &str, 
            list_of_rows: &Vec<Vec<String>>,
        ) -> Result<Self, String>  {
        if list_of_rows.len() < 3 {
            return Err(format!("Templates must have at least one data line, but overall length was only {}",
                list_of_rows.len()))
        }
        let first_row_headers = &list_of_rows[0];
        let second_row_headers= &list_of_rows[1];
        let n1 = first_row_headers.len();
        let n2 = second_row_headers.len();

        if n1 != n2 {
            return Err(format!("Malformed headers: first line has {} fields, second line has {}", n1, n2));
        }
        let mut header_duplets: Vec<HeaderDuplet> = vec![];
        for i in 0..(n1-1) {
            header_duplets.push(HeaderDuplet::new(&first_row_headers[i], &second_row_headers[i]));
            println!("{} ", header_duplets[i]); // Print each column name (header)
        }
        if let Err(res) = qc_list_of_header_items(&header_duplets) {
            return Err(res);
        }
        // if we get here, then we know that the constant parts of the template have the correct
        // format. The additional columns are either valid HPO template columns or are NTR columns
        // new term request columns, for which we only output a warning
        // Because of the structure of the template, we know that the index of
        // the HPO columns begins. We require that there is at least one such column.
        let hpo = SimpleHPO::new(hpo_json_path);
        if hpo.is_err() {
            return Err(hpo.err().unwrap());
        }
        let simple_hpo = hpo.unwrap();
        let mut index_to_hpo_factory: HashMap<usize, HpoTemplateFactory> = HashMap::new();
        for i in (NUMBER_OF_CONSTANT_HEADER_FIELDS + 1)..header_duplets.len() {
            let valid_tid =  simple_hpo.is_valid_term_id(&header_duplets[i].h1);
            if valid_tid.is_err() {
                return Err(format!("Invalid term id: {}", valid_tid.err().unwrap()));
            }
            let valid_label = simple_hpo.is_valid_term_label(&header_duplets[i].h1, &header_duplets[i].h2);
            if valid_label.is_err() {
                return Err(format!("Invalid HPO label: {}", valid_label.err().unwrap()));
            }  
            let hpo_fac = HpoTemplateFactory::new(&header_duplets[i].h1, &header_duplets[i].h2);
            index_to_hpo_factory.insert(i,hpo_fac);          
        }
        Ok(IndividualTemplateFactory {
            hpo: simple_hpo,
            expected_n_fields: n1,
            index_to_hpo_factory_d: index_to_hpo_factory
        })
    }

    /// This function transforms one line of the input Excel file into an IndividualTemplate object
    /// This object is a quality-controlled intermediate representation of the data that can
    /// easily be transformed into a phenopacket
    /// # Arguments
    ///
    /// * `row` - A vector of the fields of the Excel file row, represented as Strings
    ///
    /// # Returns
    /// 
    /// A result containing the corresponding IndividualTemplate object or an Err with Vector of strings representing the problems
    pub fn individual_template_row(row: Vec<String>) -> Result<IndividualTemplate, Vec<String>> {
        let mut list_of_errors: Vec<String> = vec![];
        let title = match TitleCell::new(&row[0]) {
            Ok(title) => Some(title),
            Err(err) => {
                list_of_errors.push(err);
                None
            }
        };
        let pmid = match Curie::new_pmid(&row[1]) {
            Ok(pmid) => Some(pmid), 
            Err(err) => {
                list_of_errors.push(err.to_string()); 
                None 
            }
        };
        let individual_id = match SimpleLabel::individual_id(&row[2]) {
            Ok(id ) => Some(id),
            Err(err) => {
                list_of_errors.push(err.to_string());
                None
            }
        };
        let disease_id = match Curie::new_disease_id(&row[4]) {
            Ok(id) => Some(id),
            Err(err) => {
                list_of_errors.push(err.to_string());
                None
            }
        };
        let disease_label = match SimpleLabel::disease_label(&row[5]) {
            Ok(id) => Some(id),
            Err(err) => {
                list_of_errors.push(err);
                None
            }
        };
        let hgnc_id = match Curie::new_disease_id(&row[6]) {
            Ok(id) => Some(id),
            Err(err) => {
                list_of_errors.push(err);
                None
            }
        };
        let gene_sym = match SimpleLabel::gene_symbol(&row[7]) {
            Ok(sym) => Some(sym),
            Err(err) => {
                list_of_errors.push(err);
                None
            }  
        };

        if ! list_of_errors.is_empty() {
            return Err(list_of_errors);
        } else {
            // If we get here, then we know we can safely unwrap the following items
            return Ok(IndividualTemplate::new(title.unwrap(), 
                                            pmid.unwrap(), 
                                            individual_id.unwrap(),
                                            disease_id.unwrap(),
                                            disease_label.unwrap(),
                                            hgnc_id.unwrap(),
                                            gene_sym.unwrap()));
        }
    }

}



fn qc_list_of_header_items(header_duplets: &Vec<HeaderDuplet>) -> Result<(), String> {
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
        let s = format!("Could not parse headers: {}", errors.join(", "));
        return Err(s);
    }
    Ok(())
}



#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_title_ctor() {
        let tests = vec![
            ("We are missing something", "We are missing something"),
            ("We are missing something ", "Title 'We are missing something ' ends with whitespace"),
            (" We are missing something", "Title ' We are missing something' begins with whitepsace"),
            ("", "Title field is empty")
        ];
        for test in tests {
            let title = TitleCell::new(test.0);
            match title {
                Ok(title) => assert_eq!(test.1, title.value()),
                Err(err) => assert_eq!(test.1, err),
            }
        }
    }

   


    #[test]
    fn test_header_duplet_ctor() {
        let hd = HeaderDuplet::new("Arachnodactly", "HP:0001166");
        let expected_header1 = String::from("Arachnodactly");
        let expected_header2 = String::from("HP:0001166");
        assert_eq!(expected_header1, hd.h1);
        assert_eq!(expected_header2, hd.h2);
    }




}