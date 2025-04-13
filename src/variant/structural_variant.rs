use ontolius::{term::{simple::SimpleMinimalTerm, MinimalTerm}, Identified, TermId};
use rand::{distr::Alphanumeric, Rng};
use serde::{Serialize, Deserialize};
use std::{collections::HashMap, str::FromStr};
use lazy_static::lazy_static;

const ACCEPTABLE_GENOMES: [&str; 2] = [ "GRCh38",  "hg38"];

lazy_static! {
    pub static ref CHROMOSOMAL_TRANSLOCATION: SimpleMinimalTerm = SimpleMinimalTerm::new(
        TermId::from_str("SO:1000044").unwrap(),
        "chromosomal_translocation".to_string(),
        vec![], 
        false 
    );

    pub static ref CHROMOSOMAL_DELETION: SimpleMinimalTerm = SimpleMinimalTerm::new(
        TermId::from_str("SO:1000029").unwrap(),
        "chromosomal_deletion".to_string(),
        vec![], 
        false 
    );

    pub static ref CHROMOSOMAL_DUPLICATION: SimpleMinimalTerm = SimpleMinimalTerm::new(
        TermId::from_str("SO:1000037").unwrap(),
        "chromosomal_duplication".to_string(),
        vec![], 
        false 
    );

    pub static ref CHROMOSOMAL_INVERSION: SimpleMinimalTerm = SimpleMinimalTerm::new(
        TermId::from_str("SO:1000030").unwrap(),
        "chromosomal_inversion".to_string(),
        vec![], 
        false 
    );

}

#[derive(Serialize, Deserialize, Clone)]
pub struct StructuralVariant {
    variant_id: String,
    label: String,
    gene_symbol: String,
    hgnc_id: String,
    so_id: String,
    so_label: String,
    genotype: Option<String>,
}

impl StructuralVariant {
    // Constructor
    pub fn new(cell_contents: String, gene_symbol: String, gene_id: String, so_term: &SimpleMinimalTerm, variant_id: Option<String>) -> Self {
        let variant_id = variant_id.unwrap_or_else(|| {
            let rand_str: String = rand::rng()
                .sample_iter(&Alphanumeric)
                .take(25)
                .map(char::from)
                .collect();
            format!("var_{}", rand_str)
        });

        if gene_symbol.is_empty() {
            panic!("Need to pass a valid gene symbol!");
        }

        if gene_id.is_empty() {
            panic!("Need to pass a valid HGNC gene id!");
        }

        Self {
            variant_id,
            label: cell_contents.trim().to_string(),
            gene_symbol,
            hgnc_id: gene_id,
            so_id: so_term.identifier().to_string(),
            so_label: so_term.name().to_string(),
            genotype: None,
        }
    }

   

    // Static Constructors for Specific Variants
    pub fn chromosomal_deletion(
        cell_contents: impl Into<String>, 
        gene_symbol: impl Into<String>, 
        gene_id: impl Into<String>, 
        variant_id: Option<String>) -> Self {
        Self::new(cell_contents.into(), gene_symbol.into(), gene_id.into(), &CHROMOSOMAL_DELETION, variant_id)
    }

    pub fn chromosomal_duplication(
        cell_contents: impl Into<String>,  
        gene_symbol: impl Into<String>,  
        gene_id: impl Into<String>,  
        variant_id: Option<String>) -> Self {
        Self::new(cell_contents.into(), gene_symbol.into(), gene_id.into(), &CHROMOSOMAL_DUPLICATION, variant_id)
    }

    pub fn chromosomal_inversion(
        cell_contents: impl Into<String>,  
        gene_symbol: impl Into<String>,  
        gene_id: impl Into<String>,  
        variant_id: Option<String>
    ) -> Self {
        Self::new(cell_contents.into(), gene_symbol.into(), gene_id.into(), &CHROMOSOMAL_INVERSION, variant_id)
    }

    pub fn chromosomal_translocation(
        cell_contents: impl Into<String>,  
        gene_symbol: impl Into<String>,  
        gene_id: impl Into<String>,  
        variant_id: Option<String>
    ) -> Self {
        Self::new(cell_contents.into(), gene_symbol.into(), gene_id.into(), &CHROMOSOMAL_TRANSLOCATION, variant_id)
    }
}
