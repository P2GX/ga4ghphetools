//! This module contains utilities for the initial input and quality control of the table cells
//!
//! Each table cell is modelled as having the ability to return a datatype and the contents as a String
//! We garantee that if these objects are created, then we are ready to create phenopackets.

use std::collections::HashMap;
use std::fmt::{self};
use std::sync::Arc;
use std::time::Instant;

use ontolius::ontology::csr::FullCsrOntology;

use crate::template::curie::Curie;
use crate::error::{self, Error, Result};
use crate::rphetools_traits::TableCell;
use crate::hpo::simple_hpo::{SimpleHPOMapper, HPO};
use crate::template::simple_label::SimpleLabel;
use super::header_duplet_row::{self, HeaderDupletRow};


impl Error {
    fn unrecognized_value(val: &str, field_name: &str) -> Self {
        Error::UnrecognizedValue {
            value: val.to_string(),
            column_name: field_name.to_string(),
        }
    }

    fn malformed_title(title: &str) -> Self {
        Error::TemplateError { msg: format!("Malformed template header '{}'", title) }
    }

    fn no_content(i: usize) -> Self {
        Error::TemplateError { msg: format!("No content and index '{i}'") }
    }
}


pub struct IndividualTemplate<'a> {
    header_duplet_row: &'a HeaderDupletRow,
    content: &'a Vec<String>,
}

impl<'a> IndividualTemplate<'a> {
    pub fn new(
        header_duplet_row: &'a HeaderDupletRow,
        content: &'a Vec<String>,
    ) -> Self {
        Self {
            header_duplet_row,
            content: content,
        }
    }

    fn get_item(&self, title: &str) -> Result<String> {
        self.header_duplet_row
        .get_idx(title)
        .ok_or_else(|| Error::malformed_title(title))
        .and_then(|i| {
            self.content
                .get(i)
                .cloned()
                .ok_or_else(|| Error::no_content(i))
        })
    }

    pub fn individual_id(&self) -> Result<String> {
        self.get_item("individual_id")
    }

    pub fn pmid(&self) -> Result<String> {
        self.get_item("PMID")
    }

    pub fn title(&self) -> Result<String> {
        self.get_item("title")
    }

    pub fn disease_id(&self) -> Result<String> {
        self.get_item("disease_id")
    }

    pub fn disease_label(&self) -> Result<String> {
        self.get_item("disease_label")
    }

    pub fn hgnc_id(&self) -> Result<String> {
        self.get_item("hgnc_id")
    }

    pub fn gene_symbol(&self) -> Result<String> {
        self.get_item("gene_symbol")
    }

    pub fn transcript_id(&self) -> Result<String> {
        self.get_item("transcript_id")
    }

    pub fn allele_1(&self) -> Result<String> {
        self.get_item("allele_1")
    }

    pub fn allele_2(&self) -> Result<String> {
        self.get_item("allele_2")
    }

    pub fn age_of_onset(&self) -> Result<String> {
        self.get_item("age_at_onset")
    }

    pub fn age_at_last_encounter(&self) -> Result<String> {
        self.get_item("age_at_last_encounter")
    }

    pub fn deceased(&self) -> Result<String> {
        self.get_item("deceased")
    }

    pub fn sex(&self) -> Result<String> {
        self.get_item("sex")
    }
}


