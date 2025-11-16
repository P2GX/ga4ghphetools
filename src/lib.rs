//! # PheTools
//!
//! A library for curating GA4GH Phenopackets from case or cohort reports.
//! 
//! ## Features
//! 
//! - Human Phenotype Ontology (HPO) text mining
//! - Semiautomated import of external tables (e.g., Supplemental Material) with data on cohorts
//! - Quality control of variant data (HGVS; symbolic structural variants)
//! - Generation of GA4GH Phenopackets
//! - Serialization with bespoke JSON format
//! - Output of aggregate tabular format suitable for HPO phenotype.hpoa pipeline
//! - API for graphical user interface (GUI) curation tools


// for development allow this
#![allow(dead_code)]
#![allow(unused_variables)]
//#![allow(unused_imports)]

mod header;

pub mod age;
pub mod dto;
pub mod etl;
pub mod export;
pub mod hpo;
pub mod hpoa;
pub mod factory;
pub mod persistence;
pub mod ppkt;
pub mod variant;

#[cfg(test)]
pub mod test_utils;