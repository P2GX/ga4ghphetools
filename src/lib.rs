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


mod error;
mod header;
mod hpo;
mod persistence;
mod ppkt;
mod phetools_traits;
mod template;
mod variant;

pub mod dto;

pub use template::phetools::PheTools;

