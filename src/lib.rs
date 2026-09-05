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
//! - Serialization of cohorts of phenopackets with bespoke JSON format
//! - Output of aggregate tabular format suitable for HPO phenotype.hpoa pipeline
//! - API for graphical user interface (GUI) curation tools


// for development allow this
//#![allow(dead_code)]
//#![allow(unused_variables)]
//#![allow(unused_imports)]

mod header;

pub mod age;
pub mod dto;
pub mod error;

mod etl;
pub use etl::process_allele_column;
pub use etl::get_cohort_data_from_etl_dto;

mod export;
pub use export::render_html;

mod hpo;
pub use hpo::get_modifiers;
pub use hpo::get_hpo_terms_by_toplevel;

pub mod hpoa;

mod factory;
pub use factory::load_json_cohort;
pub use factory::extract_template_name;
pub use factory::create_new_melded_cohort;
pub use factory::qc_assessment;
pub use factory::sanitize_cohort_data;
pub use factory::sort_rows;
pub use factory::add_hpo_term_to_cohort;
pub use factory::create_new_cohort_data;
pub use factory::add_new_row_to_cohort;
pub use factory::merge_cohort_data_from_etl_dto;


mod persistence;
pub use persistence::initialize_project_dir;

mod ppkt;
pub use ppkt::write_phenopackets;

mod repo;
pub use repo::get_repo_qc;
pub use repo::update_all_ppkt;
pub use repo::compare_two_phenopackets;

mod variant;
pub use variant::validate_hgvs_variant;
pub use variant::validate_structural_variant;
pub use variant::validate_intergenic_variant;
pub use variant::analyze_variants;


#[cfg(feature = "tauri")]
mod tauri;
#[cfg(feature = "tauri")]
pub use tauri::parent_child::get_hpo_parent_and_children_terms;



#[cfg(test)]
pub mod test_utils;