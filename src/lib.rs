//! # PheTools
//!
//! Users interact with the library via the PheTools structure.
//! The library does not expose custom datatypes, and errors are translated
//! into strings to simplify the use of phetools in applications
//! 
//! ## Features
//! 
//! - Quality assessment of phenopackets template files
//! - Generation of GA4GH Phenopackets
//! - API for curation tools



mod header;
mod persistence;


mod variant;


pub mod age;
pub mod dto;
pub mod etl;
pub mod hpo;
pub mod hpoa;
pub use factory::phetools::PheTools;
pub mod factory;
pub mod ppkt;

