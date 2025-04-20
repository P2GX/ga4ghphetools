//! PpktExporter -- One row together with Header information, all that is needed to export a GA4GH phenopacket
//!
//! Each table cell is modelled as having the ability to return a datatype and the contents as a String
//! If a PpktExporter instance has no error, then we are ready to create a phenopacket.

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


pub struct PpktExporter {
    header_duplet_row: Arc<HeaderDupletRow>,
    content: Vec<String>,
    hpo: Arc<FullCsrOntology>,
}

impl PpktExporter {
    pub fn new(
        header_duplet_row: Arc<HeaderDupletRow>,
        content: Vec<String>,
        hpo: Arc<FullCsrOntology>,
    ) -> Self {
        Self {
            header_duplet_row,
            content: content,
            hpo
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

    /// Return the first error or OK
    pub fn qc_check(&self) -> Result<()> {
        let ncols = self.content.len();
        for i in 0..ncols {
            let cell_contents = self.content[i].as_str();
            self.header_duplet_row.qc_check(i, cell_contents)?;
        }

        Ok(())
    }

    /// Get a (potentially empty) list of Errors for this template
    pub fn get_errors(&self) -> Vec<Error> {
        self.content
            .iter()
            .enumerate()
            .filter_map(|(i, cell)| self.header_duplet_row.qc_check(i, cell).err())
            .collect()
    }
}


