
//! This struct sets up code to generate the IndividualtemplateRow objects that we will
/// use to generate phenopacket code. Each IndivudalTemplateRow object is an intermediate
/// object in which we have performed sufficient quality control to know that we are able
/// to create a valid phenopacket. The IndividualTemplateFactory sets up code that leverages
/// the data in the first two rows of the template to generate an IndivudalTemplateRow from
/// each of the subsequent rows of the Excel file. We treat the constant columns with constructors (new functions)
/// that perform Q/C. The HPO columns require somewhat more functionality and use HpoTemplateFactory,
/// one for each column.
/// 
/// 
/// 
use std::collections::HashMap;
use std::fmt::{self};
use std::time::Instant;

use ontolius::ontology::csr::FullCsrOntology;

use crate::header::header_duplet::{HeaderDuplet, HeaderDupletItem, HeaderDupletItemFactory};
use crate::header::hpo_term_duplet::HpoTermDuplet;
use crate::pptcolumn::allele::Allele;
use crate::template::curie::Curie;
use crate::error::{self, Error, Result};
use crate::hpo::hpo_term_template::{HpoTemplate, HpoTemplateFactory};
use crate::pptcolumn::age::{Age, AgeTool, AgeToolTrait};
use crate::pptcolumn::deceased::DeceasedTableCell;
use crate::rphetools_traits::TableCell;
use crate::hpo::simple_hpo::{SimpleHPOMapper, HPO};
use crate::template::simple_label::SimpleLabel;
use crate::pptcolumn::transcript::Transcript;

use super::individual_template::{IndividualTemplate, SexTableCell, TitleCell};


///TODO CHANGE
const NUMBER_OF_CONSTANT_HEADER_FIELDS: usize = 17;

#[derive(Debug)]
pub struct IndividualTemplateFactory {
    hpo: SimpleHPOMapper,
    expected_n_fields: usize,
    index_to_hpo_factory_d: HashMap<usize, HpoTemplateFactory>,
    content_rows: Vec<Vec<String>>,
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
    pub fn new(hpo: &FullCsrOntology, list_of_rows: &Vec<Vec<String>>) -> Result<Self> {
        if list_of_rows.len() < 3 {
            return Err(Error::header_error(format!(
                "Templates must have at least one data line, but overall length was only {}",
                list_of_rows.len()
            )));
        }
        let first_row_headers = &list_of_rows[0];
        let second_row_headers = &list_of_rows[1];
        let n1 = first_row_headers.len();
        let n2 = second_row_headers.len();

        if n1 != n2 {
            return Err(Error::header_error(format!(
                "Malformed headers: first line has {} fields, second line has {}",
                n1, n2
            )));
        }
        let mut header_duplets: Vec<HeaderDuplet> = vec![];
        let mut in_hpo_term_section = false;
        for i in 0..(n1 - 1) {
            match HeaderDuplet::get_duplet( &first_row_headers[i]) {
                Some(duplet) => {
                    if duplet.row2() != second_row_headers[i] {
                        return Err(Error::TemplateError { 
                            msg: format!("Malformed second template row ({}) for {}: expected {}",
                                    &second_row_headers[i], duplet.row1(), duplet.row2()) 
                        });
                    } else {
                        println!("{}", &duplet);
                        if duplet.is_separator() {
                            in_hpo_term_section = true;
                        }
                        header_duplets.push(duplet);
                    }
                },
                None => {
                    if in_hpo_term_section {
                    // must be ab HPO column
                        let hdup = HpoTermDuplet::new(&first_row_headers[i], &second_row_headers[i]);
                        header_duplets.push(hdup.into_enum());
                    } else {
                        // Could not find HeaderDuplet in constant section-- the title must be erroneous
                        return Err(Error::TemplateError { msg: format!("Malformed title: '{}'", &first_row_headers[i]) });
                    }
                }
            }
        }
        
        // if we get here, then we know that the constant parts of the template have the correct
        // format. The additional columns are either valid HPO template columns or are NTR columns
        // new term request columns, for which we only output a warning
        // Because of the structure of the template, we know that the index of
        // the HPO columns begins. We require that there is at least one such column.
        let start = Instant::now();
        let hpo = SimpleHPOMapper::new(hpo);
        if hpo.is_err() {
            return Err(hpo.err().unwrap());
        }
        let simple_hpo = hpo.unwrap();
        let duration = start.elapsed(); //
        println!("Parsed HPO in: {:.3} seconds", duration.as_secs_f64());
        let mut index_to_hpo_factory: HashMap<usize, HpoTemplateFactory> = HashMap::new();
        for i in (NUMBER_OF_CONSTANT_HEADER_FIELDS + 1)..header_duplets.len() {
            let valid_label =
                simple_hpo.is_valid_term_label(&header_duplets[i].row2(), &header_duplets[i].row1());
            if valid_label.is_err() {
                return Err(Error::term_error(format!(
                    "Invalid HPO label: {}",
                    valid_label.err().unwrap()
                )));
            }
            let valid_tid = simple_hpo.is_valid_term_id(&header_duplets[i].row2());
            if valid_tid.is_err() {
                return Err(Error::term_error(format!(
                    "Invalid term id: {}",
                    valid_tid.err().unwrap()
                )));
            }
            let hpo_fac = HpoTemplateFactory::new(&header_duplets[i].row1(), &header_duplets[i].row2());
            index_to_hpo_factory.insert(i, hpo_fac);
        }
        Ok(IndividualTemplateFactory {
            hpo: simple_hpo,
            expected_n_fields: n1,
            index_to_hpo_factory_d: index_to_hpo_factory,
            content_rows: list_of_rows.iter().skip(2).cloned().collect(),
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
    pub fn individual_template_row(&self, row: Vec<String>) -> Result<IndividualTemplate> {
        let pmid = Curie::new_pmid(&row[0])?;
        let title = TitleCell::new(&row[1])?;
        let individual_id = SimpleLabel::individual_id(&row[2])?;
        let disease_id = Curie::new_disease_id(&row[4])?;
        let disease_label = SimpleLabel::disease_label(&row[5])?;
        let hgnc_id = Curie::new_hgnc_id(&row[6])?;
        let gene_sym = SimpleLabel::gene_symbol(&row[7])?;
        let tx_id = Transcript::new(&row[8])?;
        let a1 = Allele::new(&row[9])?;
        let a2 = Allele::new(&row[10])?;
        // field 11 is variant comment - skip it here!
        let age_parser = AgeTool::new();
        let onset = age_parser.parse(&row[12])?;
        let encounter = age_parser.parse(&row[13])?;
        let deceased = DeceasedTableCell::new::<&str>(&row[14])?;
        let sex = SexTableCell::new::<&str>(&row[15])?;
        // when we get here, we have parsed all of the constant columns. We can begin to parse the HPO
        // columns. The template contains a variable number of such columns
        let mut hpo_column_list: Vec<HpoTemplate> = vec![];

        // Iterate over key-value pairs
        for (idx, hpo_template_factory) in &self.index_to_hpo_factory_d {
            let cell_contents = row.get(*idx);
            match cell_contents {
                Some(val) => {
                    let hpo_tplt = hpo_template_factory.from_cell_value(val)?;
                    hpo_column_list.push(hpo_tplt);
                }
                None => {
                    println!("Probably this means there was nothing in the cell -- check this later todo");
                }
            }
        }

        // If we get here, then we know we can safely unwrap the following items
        // TODO -- FIGURE OUT WHETHER WE NEED SOME ETC.
        return Ok(IndividualTemplate::new(
            title,
            pmid,
            individual_id,
            disease_id,
            disease_label,
            hgnc_id,
            gene_sym,
            tx_id,
            a1,
            Some(a2),
            Some(onset),
            Some(encounter),
            deceased,
            sex,
            hpo_column_list,
        ));
    }

    /// Return all phenopacket templates or a list of errors if there was one or more problems
    pub fn get_templates(&self) -> Result<Vec<IndividualTemplate>> {
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
    fn hpo() -> FullCsrOntology {
        let path = "resources/hp.v2025-03-03.json.gz";
        let reader = GzDecoder::new(BufReader::new(File::open(path).unwrap()));
        let loader = OntologyLoaderBuilder::new().obographs_parser().build();

        loader.load_from_read(reader).unwrap()
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
    fn test_factory_valid_input(original_matrix: Vec<Vec<String>>, hpo: FullCsrOntology) {
        let factory = IndividualTemplateFactory::new(&hpo, &original_matrix); 
        assert!(factory.is_ok());
    }

    #[rstest]
    fn test_malformed_hpo_label(mut original_matrix: Vec<Vec<String>>, hpo: FullCsrOntology) {
        // "Hallux valgus" has extra white space
        original_matrix[0][19] = "Hallux  valgus".to_string(); 
        let factory = IndividualTemplateFactory::new(&hpo, &original_matrix); 
        assert!(&factory.is_err());
        assert!(matches!(&factory, Err(Error::TermError { .. })));
        let err = factory.unwrap_err();
        let err_msg = err.to_string();
        let expected = "Invalid HPO label: HPO Term HP:0010034 with malformed label 'Hallux  valgus' instead of Short 1st metacarpal";
        assert_eq!(expected, err_msg);
    }


    #[rstest]
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
    fn test_malformed_title_row(mut original_matrix: Vec<Vec<String>>, hpo: FullCsrOntology, #[case] idx: usize, #[case] label: &str) {
        // Test that we catch malformed labels for the first row
        original_matrix[0][idx] = label.to_string(); 
        let factory = IndividualTemplateFactory::new(&hpo, &original_matrix); 
        assert!(true);
        assert!(&factory.is_err());
        assert!(matches!(&factory, Err(Error::TemplateError { .. })));
        let err = factory.unwrap_err();
        let err_msg = err.to_string();
        let expected = format!("Malformed title: '{}'", label);
        assert_eq!(expected, err_msg);
    }

   


}
