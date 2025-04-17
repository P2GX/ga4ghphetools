//! This module contains utilities for the initial input and quality control of the table cells
//!
//! Each table cell is modelled as having the ability to return a datatype and the contents as a String
//! We garantee that if these objects are created, then we are ready to create phenopackets.

use std::collections::HashMap;
use std::fmt::{self};
use std::time::Instant;

use ontolius::ontology::csr::FullCsrOntology;

use crate::pptcolumn::allele::Allele;
use crate::pptcolumn::header_duplet::HeaderDuplet;
use crate::template::curie::Curie;
use crate::error::{self, Error, Result};
use crate::hpo::hpo_term_template::{HpoTemplate, HpoTemplateFactory};
use crate::pptcolumn::age::{Age, AgeTool, AgeToolTrait};
use crate::pptcolumn::deceased::DeceasedTableCell;
use crate::rphetools_traits::TableCell;
use crate::hpo::simple_hpo::{SimpleHPOMapper, HPO};
use crate::template::simple_label::SimpleLabel;
use crate::pptcolumn::transcript::Transcript;

impl Error {
    fn unrecognized_value(val: &str, field_name: &str) -> Self {
        Error::UnrecognizedValue {
            value: val.to_string(),
            column_name: field_name.to_string(),
        }
    }

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



/// These fields are always required by our template
const NUMBER_OF_CONSTANT_HEADER_FIELDS: usize = 17;
static EXPECTED_H1_FIELDS: [&str; NUMBER_OF_CONSTANT_HEADER_FIELDS] = [
    "PMID",
    "title",
    "individual_id",
    "comment",
    "disease_id",
    "disease_label",
    "HGNC_id",
    "gene_symbol",
    "transcript",
    "allele_1",
    "allele_2",
    "variant.comment",
    "age_of_onset",
    "age_at_last_encounter",
    "deceased",
    "sex",
    "HPO",
];
const EXPECTED_H2_FIELDS: [&str; NUMBER_OF_CONSTANT_HEADER_FIELDS] = [
    "CURIE",
    "str",
    "str",
    "optional",
    "CURIE",
    "str",
    "CURIE",
    "str",
    "str",
    "str",
    "str",
    "optional",
    "age",
    "age",
    "yes/no/na",
    "M:F:O:U",
    "na",
];

#[derive(Debug)]
pub enum TableCellDataType {
    Title(TitleCell),
    PMID(Curie),
}

pub struct HeaderItem {}

/// This struct represents the contents of a cell of the Excel table that represents the title of a publication
#[derive(Debug)]
pub struct TitleCell {
    title: String,
}

impl TitleCell {
    pub fn new(title: &str) -> Result<Self> {
        if title.is_empty() {
            return Err(Error::EmptyField {
                field_name: "Title".to_string(),
            });
        } else if title.chars().last().map_or(false, |c| c.is_whitespace()) {
            return Err(Error::trailing_ws(title.to_string()));
        } else if title.chars().next().map_or(false, |c| c.is_whitespace()) {
            return Err(Error::leading_ws(title.to_string()));
        }
        Ok(TitleCell {
            title: title.to_string(),
        })
    }
}

impl TableCell for TitleCell {
    fn value(&self) -> String {
        self.title.clone()
    }
}

fn qc_cell_entry(value: &str) -> Result<()> {
    if value.is_empty() {
        return Err(Error::EmptyField {
            field_name: "".to_ascii_lowercase(),
        });
    } else if value.chars().last().map_or(false, |c| c.is_whitespace()) {
        return Err(Error::trailing_ws(value));
    } else if value.chars().next().map_or(false, |c| c.is_whitespace()) {
        return Err(Error::leading_ws(value));
    } else {
        Ok(())
    }
}

#[derive(PartialEq)]
pub enum Sex {
    Male,
    Female,
    Other,
    Unknown,
}

pub struct SexTableCell {
    sex: Sex,
}

impl SexTableCell {
    pub fn new<S: Into<String>>(value: &str) -> Result<Self> {
        match value {
            "M" => Ok(SexTableCell { sex: Sex::Male }),
            "F" => Ok(SexTableCell { sex: Sex::Female }),
            "O" => Ok(SexTableCell { sex: Sex::Other }),
            "U" => Ok(SexTableCell { sex: Sex::Unknown }),
            _ => Err(Error::unrecognized_value(value, "Sex")),
        }
    }

    pub fn male(&self) -> bool {
        return self.sex == Sex::Male;
    }

    pub fn female(&self) -> bool {
        return self.sex == Sex::Female;
    }

    pub fn other_sex(&self) -> bool {
        return self.sex == Sex::Other;
    }
}

impl TableCell for SexTableCell {
    fn value(&self) -> String {
        match self.sex {
            Sex::Male => "M".to_string(),
            Sex::Female => "F".to_string(),
            Sex::Other => "O".to_string(),
            Sex::Unknown => "U".to_string(),
        }
    }
}

pub struct IndividualTemplate {
    title: TitleCell,
    pmid: Curie,
    individual_id: SimpleLabel,
    disease_id: Curie,
    disease_label: SimpleLabel,
    hgnc_id: Curie,
    gene_symbol: SimpleLabel,
    transcript_id: Transcript,
    allele_1: Allele,
    allele_2: Option<Allele>,
    age_at_onset: Option<Age>,
    age_at_last_encounter: Option<Age>,
    deceased: DeceasedTableCell,
    sex: SexTableCell,
    hpo_column_list: Vec<HpoTemplate>,
}

impl IndividualTemplate {
    pub fn new(
        title: TitleCell,
        pmid: Curie,
        individual_id: SimpleLabel,
        disease_id: Curie,
        disease_label: SimpleLabel,
        hgnc: Curie,
        gene_sym: SimpleLabel,
        tx_id: Transcript,
        allele1: Allele,
        allele2: Option<Allele>,
        age_onset: Option<Age>,
        age_last_encounter: Option<Age>,
        deceased: DeceasedTableCell,
        sex: SexTableCell,
        hpo_columns: Vec<HpoTemplate>,
    ) -> Self {
        IndividualTemplate {
            title: title,
            pmid: pmid,
            individual_id,
            disease_id,
            disease_label,
            hgnc_id: hgnc,
            gene_symbol: gene_sym,
            transcript_id: tx_id,
            allele_1: allele1,
            allele_2: allele2,
            age_at_onset: age_onset,
            age_at_last_encounter: age_last_encounter,
            deceased: deceased,
            sex: sex,
            hpo_column_list: hpo_columns,
        }
    }
    pub fn individual_id(&self) -> String {
        self.individual_id.value()
    }

    pub fn pmid(&self) -> String {
        self.pmid.value()
    }

    pub fn title(&self) -> String {
        self.title.value()
    }

    pub fn disease_id(&self) -> String {
        self.disease_id.value()
    }

    pub fn disease_label(&self) -> String {
        self.disease_label.value()
    }

    pub fn hgnc_id(&self) -> String {
        self.hgnc_id.value()
    }

    pub fn gene_symbol(&self) -> String {
        self.gene_symbol.value()
    }

    pub fn transcript_id(&self) -> String {
        self.transcript_id.value()
    }

    pub fn allele_1(&self) -> String {
        self.allele_1.value()
    }

    pub fn allele_2(&self) -> Option<String> {
        match &self.allele_2 {
            Some(a) => Some(a.value()),
            None => None,
        }
    }

    pub fn age_of_onset(&self) -> Option<Age> {
        self.age_at_onset.clone()
    }

    pub fn age_at_last_encounter(&self) -> Option<Age> {
        self.age_at_last_encounter.clone()
    }

    pub fn deceased(&self) -> &DeceasedTableCell {
        &self.deceased
    }

    pub fn sex(&self) -> &SexTableCell {
        &self.sex
    }

    pub fn hpo_terms(&self) -> &Vec<HpoTemplate> {
        &self.hpo_column_list
    }
}

/// This object collects all errors found in a template when parsing the content rows
///
/// If we find one or more individual errors, we will return this error
#[derive(Debug)]
pub struct TemplateError {
    pub messages: Vec<Error>,
}

impl TemplateError {
    pub fn new(messages: Vec<Error>) -> Self {
        TemplateError { messages }
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
    hpo: SimpleHPOMapper,
    expected_n_fields: usize,
    index_to_hpo_factory_d: HashMap<usize, HpoTemplateFactory>,
    content_rows: Vec<Vec<String>>,
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
        for i in 0..(n1 - 1) {
            header_duplets.push(HeaderDuplet::new(
                &first_row_headers[i],
                &second_row_headers[i],
            ));
            //println!("{} ", header_duplets[i]); // Print each column name (header)
        }
        if let Err(res) = qc_list_of_header_items(&header_duplets) {
            return Err(res);
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

fn qc_list_of_header_items(header_duplets: &Vec<HeaderDuplet>) -> Result<()> {
    // check each of the items in turn

    let mut errors: Vec<String> = vec![];
    for (i, duplet) in header_duplets.into_iter().enumerate() {
        if i < NUMBER_OF_CONSTANT_HEADER_FIELDS && duplet.row1() != EXPECTED_H1_FIELDS[i] {
            errors.push(format!(
                "Malformed header: expected {}, got {}",
                EXPECTED_H1_FIELDS[i], duplet.row1()
            ))
        }
        if i < NUMBER_OF_CONSTANT_HEADER_FIELDS && duplet.row2() != EXPECTED_H2_FIELDS[i] {
            errors.push(format!(
                "Malformed header (row 2): expected {}, got {}",
                EXPECTED_H2_FIELDS[i], duplet.row1()
            ))
        }
        if i > NUMBER_OF_CONSTANT_HEADER_FIELDS {
            break;
        }
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use crate::pptcolumn::header_duplet::HeaderDuplet;

    use super::*;

    #[test]
    fn test_title_ctor() {
        let tests = vec![
            ("We are missing something", "We are missing something"),
            (
                "We are missing something ",
                "Trailing whitespace in 'We are missing something '",
            ),
            (
                " We are missing something",
                "Leading whitespace in ' We are missing something'",
            ),
            ("", "Title field is empty"),
        ];
        for test in tests {
            let title = TitleCell::new(test.0);
            match title {
                Ok(title) => assert_eq!(test.1, title.value()),
                Err(err) => assert_eq!(test.1, err.to_string()),
            }
        }
    }

    #[test]
    fn test_header_duplet_ctor() {
        let hd = HeaderDuplet::new("Arachnodactly", "HP:0001166");
        let expected_header1 = String::from("Arachnodactly");
        let expected_header2 = String::from("HP:0001166");
        assert_eq!(expected_header1, hd.row1());
        assert_eq!(expected_header2, hd.row2());
    }
}
