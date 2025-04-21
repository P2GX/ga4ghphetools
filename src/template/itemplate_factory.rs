
//! IndividualTemplateFactory - create templates for GA4GH Phenopacket generation.
//! 
//! This struct sets up code to generate the IndividualtemplateRow objects that we will
//! use to generate phenopackets. Each IndivudalTemplateRow object is an intermediate
//! object in which we have performed sufficient quality control to know that we are able
//! to create a valid phenopacket. The IndividualTemplateFactory sets up code that leverages
//! the data in the first two rows of the template to generate an IndivudalTemplateRow from
//! each of the subsequent rows of template matrix (e.g., such as we receive from a front-end GUI)
 
use std::collections::HashMap;
use std::fmt::{self};
use std::sync::Arc;
use std::time::Instant;

use ontolius::ontology::csr::FullCsrOntology;
use ontolius::ontology::OntologyTerms;
use ontolius::term::MinimalTerm;
use ontolius::TermId;

use crate::header::header_duplet::{HeaderDuplet, HeaderDupletItem, HeaderDupletItemFactory};
use crate::header::hpo_term_duplet::HpoTermDuplet;
use crate::template::curie::Curie;
use crate::error::{self, Error, Result};
use crate::rphetools_traits::TableCell;
use crate::hpo::simple_hpo::{SimpleHPOMapper, HPO};
use crate::template::simple_label::SimpleLabel;
use super::header_duplet_row::HeaderDupletRow;
use super::ppkt_exporter::PpktExporter;





pub struct IndividualTemplateFactory {
    header_duplet_row: Arc<HeaderDupletRow>,
    content_rows: Vec<Vec<String>>,
    hpo: Arc<FullCsrOntology>,
}

impl Error {
    
    fn template_error(val: &str) -> Self {
        Error::TemplateError {
            msg: val.to_string(),
        }
    }

    fn header_error(val: String) -> Self {
        Error::HeaderError { msg: val }
    }

    fn term_error(val: String) -> Self {
        Error::TermError { msg: val }
    }
}

impl IndividualTemplateFactory {
    /// Create a GA4GH Phenopacket factory from a matrix of strings representing a Phetools template
    /// 
    /// The list of rows contains the entire template. The first two rows represent the Header Duplet, and
    /// the remaining rows each represent all of the data needed to create a phenopacket
    /// The new function performs Q/C on the first two header rows and only checks that the remaining rows
    /// have the same length as the header. In principle other functions will have Q/C all data entries by
    /// the time we get to this function, which is designed to prepare the export of JSON files with 
    /// GA4GH Phenopackets
    pub fn new(hpo: Arc<FullCsrOntology>, list_of_rows: &Vec<Vec<String>>) -> Result<Self> {
        if list_of_rows.len() < 3 {
            return Err(Error::header_error(format!(
                "Templates must have at least one data line, but overall length was only {}",
                list_of_rows.len()
            )));
        }
        let first_row_headers = &list_of_rows[0];
        let second_row_headers = &list_of_rows[1];
        let n_columns = first_row_headers.len();
        let n_cols_row2 = second_row_headers.len();

        if n_columns != n_cols_row2 {
            return Err(Error::header_error(format!(
                "Malformed headers: first line has {} fields, second line has {}",
                n_columns, n_cols_row2
            )));
        }
        let separator_index = Self::get_separator_index(first_row_headers, second_row_headers)?;
        let mut header_duplets: Vec<HeaderDuplet> = vec![];

        for i in 0..=separator_index {
            match HeaderDuplet::get_duplet( &first_row_headers[i]) {
                Some(duplet) => {
                    if duplet.row2() != second_row_headers[i] {
                        return Err(Error::TemplateError { 
                            msg: format!("Malformed second template row ({}) for {}: expected {}",
                                    &second_row_headers[i], duplet.row1(), duplet.row2()) 
                        });
                    } else {
                        header_duplets.push(duplet);
                    }
                },
                None => {
                        // Could not find HeaderDuplet in constant section-- the title must be erroneous
                        return Err(Error::TemplateError { msg: format!("Malformed title: '{}'", &first_row_headers[i]) });
                }
            }
        }
        /// everything after this must be an HPO term
        for i in separator_index+1..n_columns {
            let hpo_label = &first_row_headers[i];
            let hpo_id = &second_row_headers[i];
            let term_id: TermId = hpo_id.parse().unwrap();
            match hpo.term_by_id(&term_id) {
                Some(term) => {
                    if term.name() == hpo_label {
                        let hpo_duplet = HpoTermDuplet::new(hpo_label, hpo_id);
                        header_duplets.push(hpo_duplet.into_enum());
                    } else {
                        /// TODO better error messages
                        /// cargo test --package rphetools --lib -- template::itemplate_factory::test::test_malformed_hpo_label --exact --show-output 
                        return Err(Error::wrong_hpo_label_error(hpo_id, &hpo_label, term.name()));
                    }
                },
                None => {
                    return Err(Error::TemplateError { msg: format!("Could not find term for HPO id {} (expected: {})", hpo_id, hpo_label) });
                }
            }
        }
        /// If we get here, then we have successfully ingested all Header duplets.
        let header_duplet_row = HeaderDupletRow::from_duplets(&header_duplets)?;
        Ok(IndividualTemplateFactory {
            header_duplet_row: Arc::new(header_duplet_row),
            content_rows: list_of_rows.iter().skip(2).cloned().collect(),
            hpo
        })
    }

    /// function intended to be used by new to find the location of the HPO/na separator column
    /// We have already checked that the two first rows have equal length
    fn get_separator_index(
        first_row_headers: &Vec<String>, 
        second_row_headers: &Vec<String>
    ) -> Result<usize> {
        let n_cols = first_row_headers.len();
        for i in 0..n_cols {
            if first_row_headers[i] == "HPO" && second_row_headers[i] == "na" {
                return Ok(i);
            }
        }
        return Err(Error::TemplateError { msg: format!("Could not find HPO separator column in matrix with {} columns", n_cols) });
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
    pub fn individual_template_row(&self, row: Vec<String>) -> Result<PpktExporter> {
        let hdrow_arc = self.header_duplet_row.clone(); // reference counted clone
        let hpo_arc = Arc::clone(&self.hpo);
        let exporter = PpktExporter::new(hdrow_arc, row.clone(), hpo_arc);
        Ok(exporter)
    }


   

    /// Return all phenopacket templates or a list of errors if there was one or more problems
    pub fn get_templates(&self) -> Result<Vec<PpktExporter>> {
        let mut templates = Vec::new();
        for row in &self.content_rows {
            let itemplate = self.individual_template_row(row.to_vec());
            match itemplate {
                Ok(template) => {
                    templates.push(template);
                }
                Err(errs) => {
                    return Err(errs);
                }
            }
        }
        Ok(templates)
    }
}



#[cfg(test)]
mod test {
    use crate::{error::Error, header::{header_duplet::HeaderDupletItem, hpo_term_duplet::HpoTermDuplet}};
    use lazy_static::lazy_static;
    use ontolius::{io::OntologyLoaderBuilder, ontology::csr::MinimalCsrOntology};
    use polars::io::SerReader;
    use super::*;
    use std::{fs::File, io::BufReader};
    use rstest::{fixture, rstest};
    use flate2::bufread::GzDecoder;

    #[fixture]
    fn hpo() -> Arc<FullCsrOntology> {
        let path = "resources/hp.v2025-03-03.json.gz";
        let reader = GzDecoder::new(BufReader::new(File::open(path).unwrap()));
        let loader = OntologyLoaderBuilder::new().obographs_parser().build();
        let hpo = loader.load_from_read(reader).unwrap();
        Arc::new(hpo)
    }



    #[fixture]
    fn row1() -> Vec<String> 
    {
        let row: Vec<&str> = vec![
            "PMID", "title", "individual_id", "comment", "disease_id", "disease_label", "HGNC_id",	"gene_symbol", 
            "transcript", "allele_1", "allele_2", "variant.comment", "age_of_onset", "age_at_last_encounter", 
            "deceased", "sex", "HPO",	"Clinodactyly of the 5th finger", "Hallux valgus",	"Short 1st metacarpal", 
            "Ectopic ossification in muscle tissue", "Long hallux", "Pain", "Short thumb"
        ];
        row.into_iter().map(|s| s.to_owned()).collect()
    }

    #[fixture]
    fn row2() -> Vec<String> 
    {
        let row: Vec<&str> = vec![
            "CURIE", "str", "str", "optional", "CURIE", "str", "CURIE", "str", "str", "str", "str", "optional", "age", "age", "yes/no/na", "M:F:O:U", "na",
            "HP:0004209", "HP:0001822", "HP:0010034", "HP:0011987", "HP:0001847", "HP:0012531", "HP:0009778"];
        row.into_iter().map(|s| s.to_owned()).collect()
    }

    #[fixture]
    fn row3() -> Vec<String> {
        let row: Vec<&str> =  vec![
            "PMID:29482508", "Difficult diagnosis and genetic analysis of fibrodysplasia ossificans progressiva: a case report", "current case", "", 
            "OMIM:135100", "Fibrodysplasia ossificans progressiva", "HGNC:171", "ACVR1", 
            "NM_001111067.4", "c.617G>A", "na", "NP_001104537.1:p.(Arg206His)", 
            "P9Y", "P16Y", "no", "M", "na", "na", "P16Y", "na", "P16Y", "P16Y", "P16Y", "na"];
        row.into_iter().map(|s| s.to_owned()).collect()
    }


    #[fixture]
    fn original_matrix(row1: Vec<String>, row2: Vec<String>, row3: Vec<String>)  -> Vec<Vec<String>> {
        let mut rows = Vec::with_capacity(3);
        rows.push(row1);
        rows.push(row2);
        rows.push(row3);
        rows
    }

    /// Make sure that our test matrix is valid before we start changing fields to check if we pick up errors
    #[rstest]
    fn test_factory_valid_input(original_matrix: Vec<Vec<String>>, hpo: Arc<FullCsrOntology>) {
        let factory = IndividualTemplateFactory::new(hpo, &original_matrix); 
        assert!(factory.is_ok());
    }

    /// There are seventeen fields in the constant section, 
    /// therefore the index of the separator is 16 (the last field)
    #[rstest]
    fn test_index_of_separator_column(original_matrix: Vec<Vec<String>>) {
        let first_row_headers = &original_matrix[0];
        let second_row_headers = &original_matrix[1];
        let result = IndividualTemplateFactory::get_separator_index(first_row_headers, second_row_headers);
        assert!(result.is_ok());
        let i = result.unwrap();
        assert_eq!(16, i);
    }

    #[rstest]
    fn test_malformed_hpo_label(mut original_matrix: Vec<Vec<String>>, hpo: Arc<FullCsrOntology>) {
        // "Hallux valgus" has extra white space
        original_matrix[0][19] = "Hallux  valgus".to_string(); 
        let factory = IndividualTemplateFactory::new(hpo, &original_matrix); 
        assert!(&factory.is_err());
        assert!(matches!(&factory, Err(Error::TermError { .. })));
        let err = factory.err().unwrap();
        let err_msg = err.to_string();
        let expected = "HPO Term HP:0010034 with malformed label 'Hallux  valgus' instead of 'Short 1st metacarpal'";
        assert_eq!(expected, err_msg);
    }


    #[rstest]
    #[case(0, "PMI")]
    #[case(1, "title ")]
    #[case(1, " title ")]
    #[case(1, "titl")]
    #[case(2, "individual")]
    #[case(3, "disease_i")]
    #[case(4, "diseaselabel")]
    #[case(5, "hgnc")]
    #[case(6, "symbol")]
    #[case(7, "tx")]
    #[case(8, "allel1")]
    #[case(9, "allele2")]
    #[case(10, "vcomment")]
    #[case(11, "age")]
    #[case(12, "age_last_counter")]
    #[case(13, "deceasd")]
    #[case(14, "sexcolumn")]
    #[case(15, "")]
    fn test_malformed_title_row(mut original_matrix: Vec<Vec<String>>, hpo: Arc<FullCsrOntology>, #[case] idx: usize, #[case] label: &str) {
        // Test that we catch malformed labels for the first row
        original_matrix[0][idx] = label.to_string(); 
        let factory = IndividualTemplateFactory::new(hpo, &original_matrix); 
        assert!(&factory.is_err());
        assert!(matches!(&factory, Err(Error::TemplateError { .. })));
        let err = factory.err().unwrap();
        let err_msg = err.to_string();
        let expected = format!("Malformed title: '{}'", label);
        assert_eq!(expected, err_msg);
    }

    // test malformed entries
    // we change entries in the third row (which is the first and only data row)
    // and introduce typical potential errors
    #[rstest]
    #[case(0, "PMID29482508", "Invalid CURIE with no colon: 'PMID29482508'")]
    #[case(0, "PMID: 29482508", "Contains stray whitespace: 'PMID: 29482508'")]
    #[case(1, "", "Value must not be empty")]
    #[case(1, "Difficult diagnosis and genetic analysis of fibrodysplasia ossificans progressiva: a case report ", 
        "Trailing whitespace in 'Difficult diagnosis and genetic analysis of fibrodysplasia ossificans progressiva: a case report '")]
    #[case(2, "individual(1)", "Forbidden character '(' found in label 'individual(1)'")]
    #[case(2, " individual A", "Leading whitespace in ' individual A'")]
    #[case(4, "MIM:135100", "Disease id has invalid prefix: 'MIM:135100'")]
    #[case(4, "OMIM: 135100", "Contains stray whitespace: 'OMIM: 135100'")]
    #[case(4, "OMIM:13510", "OMIM identifiers must have 6 digits: 'OMIM:13510'")]
    #[case(5, "Fibrodysplasia ossificans progressiva ", "Trailing whitespace in 'Fibrodysplasia ossificans progressiva '")]
    #[case(6, "HGNC:171 ", "Contains stray whitespace: 'HGNC:171 '")]
    #[case(6, "HGNC171", "Invalid CURIE with no colon: 'HGNC171'")]
    #[case(6, "HGNG:171", "HGNC id has invalid prefix: 'HGNG:171'")]
    #[case(7, "ACVR1 ", "Trailing whitespace in 'ACVR1 '")]
    #[case(8, "NM_001111067", "Transcript 'NM_001111067' is missing a version")]
    #[case(9, "617G>A", "Malformed allele '617G>A'")]
    #[case(10, "", "Value must not be empty")]
    #[case(12, "P2", "Malformed age_of_onset 'P2'")]
    #[case(13, "Adultonset", "Malformed age_at_last_encounter 'Adultonset'")]
    #[case(14, "?", "Malformed deceased entry: '?'")]
    #[case(14, "alive", "Malformed deceased entry: 'alive'")]
    #[case(15, "male", "Malformed entry in sex field: 'male'")]
    #[case(15, "f", "Malformed entry in sex field: 'f'")]
    #[case(18, "Observed", "Malformed entry for Ectopic ossification in muscle tissue (HP:0011987): 'Observed'")]
    #[case(18, "yes", "Malformed entry for Ectopic ossification in muscle tissue (HP:0011987): 'yes'")]
    #[case(18, "exc.", "Malformed entry for Ectopic ossification in muscle tissue (HP:0011987): 'exc.'")]
      fn test_malformed_entry(
        mut original_matrix: Vec<Vec<String>>, 
        hpo: Arc<FullCsrOntology>, 
        #[case] idx: usize, 
        #[case] entry: &str,
        #[case] error_msg: &str) {
           original_matrix[2][idx] = entry.to_string();
        let factory = IndividualTemplateFactory::new(hpo, &original_matrix); 
        assert!(factory.is_ok());
        let factory = factory.unwrap();
        let templates = factory.get_templates().unwrap();
        assert_eq!(1, templates.len());
        let itemplate = &templates[0];
        let result = itemplate.qc_check();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(error_msg, err.to_string());
    }


}
