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


// for development allow this
#![allow(dead_code)]
#![allow(unused_variables)]
//#![allow(unused_imports)]

mod header;
mod persistence;





pub mod age;
pub mod dto;
pub mod etl;
pub mod hpo;
pub mod hpoa;
pub use factory::phetools::PheTools;
pub mod factory;
pub mod ppkt;
pub mod variant;

