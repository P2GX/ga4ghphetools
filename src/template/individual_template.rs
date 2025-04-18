//! This module contains utilities for the initial input and quality control of the table cells
//!
//! Each table cell is modelled as having the ability to return a datatype and the contents as a String
//! We garantee that if these objects are created, then we are ready to create phenopackets.

use std::collections::HashMap;
use std::fmt::{self};
use std::sync::Arc;
use std::time::Instant;

use ontolius::ontology::csr::FullCsrOntology;

use crate::pptcolumn::allele::Allele;
use crate::header_duplet::header_duplet::HeaderDupletOld;
use crate::template::curie::Curie;
use crate::error::{self, Error, Result};
use crate::hpo::hpo_term_template::{HpoTemplate, HpoTemplateFactory};
use crate::pptcolumn::age::{Age, AgeTool, AgeToolTrait};
use crate::pptcolumn::deceased::DeceasedTableCell;
use crate::rphetools_traits::TableCell;
use crate::hpo::simple_hpo::{SimpleHPOMapper, HPO};
use crate::template::simple_label::SimpleLabel;
use crate::pptcolumn::transcript::Transcript;

use super::header_duplet_row::{HeaderDupletRow, MendelianHDRow};


impl Error {
    fn unrecognized_value(val: &str, field_name: &str) -> Self {
        Error::UnrecognizedValue {
            value: val.to_string(),
            column_name: field_name.to_string(),
        }
    }
}



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

#[derive(Debug, PartialEq)]
pub enum Sex {
    Male,
    Female,
    Other,
    Unknown,
}

#[derive(Debug)]
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




pub trait PheToolsIndividualTemplate {
    fn qc(&self) -> Result<()>;
}

#[derive(Debug)]
pub struct IndividualTemplate2<T> where T: HeaderDupletRow {
    header: Arc<T>,
    values: Vec<String>
}

impl<T>  IndividualTemplate2<T> 
    where T: HeaderDupletRow 
{
    pub fn new(header: Arc<T>, values: &Vec<String>) -> Self {
        Self {
            header, values: values.clone()
        }
    }
/* 
    pub fn individual_id(&self) -> String {
        let idx = self.get_idx("individual_id");
        self.individual_id.value()
    }*/
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
