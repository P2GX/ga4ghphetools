//! # HeaderDuplet
//! 
//! Each type of HeaderDuplet defines a column type that specifies the values in the first two lines and provides
//! quality control functions for data in the column.
//! 
//! See the [`header_duplet`](mod@crate::header::header_duplet) module for the trait that each HeaderDuplet implements.

pub mod age_of_onset_duplet;
pub mod age_last_encounter_duplet;
mod age_util;
pub mod allele_1_duplet;
pub mod allele_2_duplet;
mod allele_util;
pub mod comment_duplet;
pub mod deceased_duplet;
pub mod disease_id_duplet;
pub mod disease_label_duplet;
pub mod gene_symbol_duplet;
pub mod header_duplet;
pub mod hgnc_duplet;
pub mod hpo_separator_duplet;
pub mod hpo_term_duplet;
pub mod individual_id_duplet;
pub mod pmid_duplet;
pub mod sex_duplet;
pub mod title_duplet;
pub mod transcript_duplet;
pub mod variant_comment_duplet;