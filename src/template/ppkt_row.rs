//! PpktExporter -- One row together with Header information, all that is needed to export a GA4GH phenopacket
//!
//! Each table cell is modelled as having the ability to return a datatype and the contents as a String
//! If a PpktExporter instance has no error, then we are ready to create a phenopacket.

use std::collections::HashMap;
use std::fmt::{self};
use std::ops::Deref;
use std::sync::Arc;
use std::time::Instant;

use ontolius::ontology::csr::FullCsrOntology;
use ontolius::term::simple::SimpleTerm;
use ontolius::TermId;

use crate::header::header_duplet::{HeaderDuplet, HeaderDupletItem};
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


pub struct PpktRow {
    header_duplet_row: Arc<HeaderDupletRow>,
    content: Vec<String>,
}

impl PpktRow {
    pub fn new(
        header_duplet_row: Arc<HeaderDupletRow>,
        content: Vec<String>,

    ) -> Self {
        Self {
            header_duplet_row,
            content: content,

        }
    }

    fn get_item(&self, title: &str) -> Result<String> {
        self.header_duplet_row
        .get_idx(title)
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

   pub fn get_string_row(&self) -> Vec<String> {
        self.content.clone()
   }

   pub fn get_value_at(&self, i: usize) -> Result<String> {
        if i >= self.content.len() {
            Err(Error::TemplateError { msg: format!("Invalid index {i}") })
        } else {
            Ok(self.content[i].clone())
        }
   }
 

    pub fn update(&self, tid_map: &mut HashMap<TermId, String>, updated_hdr: Arc<HeaderDupletRow>) -> Result<Self> {
        // update the tid map with the existing  values
        let previous_hpo_id_list = self.header_duplet_row.get_hpo_id_list()?;
        let offset = self.header_duplet_row.get_hpo_offset();
        for (i, term_id) in previous_hpo_id_list.iter().enumerate() {
            let j = i + offset;
            match self.content.get(j) {
                Some(value) => {
                    tid_map.insert(term_id.clone(), value.clone());
                },
                None => {
                    return Err(Error::TemplateError { msg: format!("Could not retrieve value in update i={i}, j={j}") } );
                }
            }
        }
        let updated_hpo_id_list = updated_hdr.get_hpo_id_list()?;


        let updated_content: Result<Vec<String>> = updated_hpo_id_list
                .into_iter()
                .map(|term_id| {
                    tid_map.get(&term_id)
                        .cloned()
                        .ok_or_else(|| Error::TemplateError {
                            msg: format!("Could not retrieve updated value for '{}'", &term_id)
                        })
                })
                .collect();
        let updated_content = updated_content?;
        Ok(Self {
            header_duplet_row: updated_hdr,
            content: updated_content,
        })
    }


    pub fn get_items(&self, indices: &Vec<usize>) -> Result<Vec<String>> {
        match indices.iter().copied().max() {
            Some(max_i) => {
                if max_i > self.content.len() {
                    return Err(Error::TemplateError { msg: format!("Index {max_i} out of bounds") });
                }
                let selected: Vec<String> = indices
                    .iter()
                    .filter_map(|&idx| self.content.get(idx).cloned())
                    .collect();
                Ok(selected)
            },
            None => {
                return Err(Error::TemplateError { msg: format!("Could not extract from from indices") });
            }
        }
       
    }

    pub fn remove_whitespace(&mut self, col: usize) -> Result<()> {
        if col > self.content.len() {
            return Err(Error::TemplateError { msg: format!("row index error {col}") })
        }
        if let Some(s) = self.content.get_mut(col) {
            *s = s.chars().filter(|c| !c.is_whitespace()).collect();
        }
        Ok(())
    }

    pub fn trim(&mut self, col: usize) {
        if let Some(s) = self.content.get_mut(col) {
            *s = s.trim().to_string();
        }
    }

    
    /// Set the indicated cell to value or return an Error if the value is not valid
    pub fn set_value(&mut self, col: usize, value: &str) -> Result<()> {
        let duplet = self.header_duplet_row.get_duplet_at_index(col)?;
        duplet.qc_cell(value)?;
        if let Some(s) = self.content.get_mut(col) {
            *s = value.to_string();
        }
        Ok(())
    }
   
}


