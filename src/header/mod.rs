//! # HeaderDuplet
//! 
//! Each type of HeaderDuplet defines a column type that specifies the values in the first two lines and provides
//! quality control functions for data in the column.
//! 
//! See the [`header_duplet`](mod@crate::header::header_duplet) module for the trait that each HeaderDuplet implements.


mod allele_util;
pub mod duplet_item;
pub mod disease_header;
pub mod gene_variant_header;
pub mod hpo_term_duplet;
pub mod individual_header;