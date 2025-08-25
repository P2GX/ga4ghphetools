
//! HpoTermDuplet
//! The duplet and the QC routines for the PMID column
//! 

use std::str::FromStr;
use ontolius::TermId;
use serde::{Deserialize, Serialize};






/// A structure to represent an HPO term (id and label) in a simple way
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct HpoTermDuplet {
    pub hpo_label: String,
    pub hpo_id: String,
}


impl HpoTermDuplet {
    pub fn new(label: impl Into<String>, identifier: impl Into<String>) -> Self {
        Self { hpo_label: label.into(), hpo_id: identifier.into() }
    }

    pub fn hpo_id(&self) -> &str {
        &self.hpo_id
    }

    pub fn hpo_label(&self) -> &str {
        &self.hpo_label
    }

    pub fn to_term_id(&self) -> std::result::Result<TermId, String> {
        let tid = TermId::from_str(&self.hpo_id).map_err(|_| format!("Could not create TermId from {}", self.hpo_id()))?;
        Ok(tid)
    }
    
} 
