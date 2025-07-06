use ontolius::{term::{simple::SimpleMinimalTerm, MinimalTerm}, Identified, TermId};
use rand::{distr::Alphanumeric, Rng};
use serde::{Serialize, Deserialize};
use std::{collections::HashMap, str::FromStr};
use once_cell::sync::Lazy;
use crate::{dto::variant_dto::VariantDto, error::{Error, Result}};
const ACCEPTABLE_GENOMES: [&str; 2] = [ "GRCh38",  "hg38"];

pub const DELETION: &str = "DEL";
pub const TRANSLOCATION: &str = "TRANSL";
pub const DUPLICATION: &str = "DUP";
pub const INVSERSION: &str = "INV";



static CHROMOSOMAL_STRUCTURE_VARIATION: Lazy<SimpleMinimalTerm> = Lazy::new(|| {
    SimpleMinimalTerm::new(
        TermId::from_str("SO:1000183").unwrap(),
        "chromosome_structure_variation".to_string(),
        vec![], 
        false 
    )
});

static CHROMOSOMAL_TRANSLOCATION: Lazy<SimpleMinimalTerm> = Lazy::new(|| {
    SimpleMinimalTerm::new(
        TermId::from_str("SO:1000044").unwrap(),
        "chromosomal_translocation".to_string(),
        vec![], 
        false 
    )
});

static CHROMOSOMAL_DELETION: Lazy<SimpleMinimalTerm> = Lazy::new(|| {
    SimpleMinimalTerm::new(
        TermId::from_str("SO:1000029").unwrap(),
        "chromosomal_deletion".to_string(),
        vec![], 
        false 
    )
});

static CHROMOSOMAL_DUPLICATION: Lazy<SimpleMinimalTerm> = Lazy::new(|| {
    SimpleMinimalTerm::new(
        TermId::from_str("SO:1000037").unwrap(),
        "chromosomal_duplication".to_string(),
        vec![], 
        false 
    )
});

static CHROMOSOMAL_INVERSION: Lazy<SimpleMinimalTerm> = Lazy::new(|| {
    SimpleMinimalTerm::new(
        TermId::from_str("SO:1000030").unwrap(),
        "chromosomal_inversion".to_string(),
        vec![], 
        false 
    )
});

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
    pub fn new(
        cell_contents: String, 
        gene_symbol: String, 
        gene_id: String, 
        so_term: &SimpleMinimalTerm, 
        variant_id: Option<String>) -> std::result::Result<Self, String> {
        let variant_id = variant_id.unwrap_or_else(|| {
            let rand_str: String = rand::rng()
                .sample_iter(&Alphanumeric)
                .take(25)
                .map(char::from)
                .collect();
            format!("var_{}", rand_str)
        });

        if gene_symbol.is_empty() {
            return Err(format!("Malformed structural variant {cell_contents}: Need to pass a valid gene symbol!"));
        }

        if gene_id.is_empty() {
            return Err(format!("Malformed structural variant {cell_contents}: Need to pass a valid HGNC gene id!"));
        }

        Ok(Self {
            variant_id,
            label: cell_contents.trim().to_string(),
            gene_symbol,
            hgnc_id: gene_id,
            so_id: so_term.identifier().to_string(),
            so_label: so_term.name().to_string(),
            genotype: None,
        })
    }



    // Static Constructors for Specific Variants
    pub fn chromosomal_deletion(
        cell_contents: impl Into<String>, 
        gene_symbol: impl Into<String>, 
        gene_id: impl Into<String>, 
        variant_id: Option<String>) -> std::result::Result<Self, String> {
        Self::new(cell_contents.into(), gene_symbol.into(), gene_id.into(), &CHROMOSOMAL_DELETION, variant_id)
    }

    pub fn chromosomal_duplication(
        cell_contents: impl Into<String>,  
        gene_symbol: impl Into<String>,  
        gene_id: impl Into<String>,  
        variant_id: Option<String>) -> std::result::Result<Self, String> {
        Self::new(cell_contents.into(), gene_symbol.into(), gene_id.into(), &CHROMOSOMAL_DUPLICATION, variant_id)
    }

    pub fn chromosomal_inversion(
        cell_contents: impl Into<String>,  
        gene_symbol: impl Into<String>,  
        gene_id: impl Into<String>,  
        variant_id: Option<String>
    ) -> std::result::Result<Self, String> {
        Self::new(cell_contents.into(), gene_symbol.into(), gene_id.into(), &CHROMOSOMAL_INVERSION, variant_id)
    }

    pub fn chromosomal_translocation(
        cell_contents: impl Into<String>,  
        gene_symbol: impl Into<String>,  
        gene_id: impl Into<String>,  
        variant_id: Option<String>
    ) -> std::result::Result<Self, String> {
        Self::new(cell_contents.into(), gene_symbol.into(), gene_id.into(), &CHROMOSOMAL_TRANSLOCATION, variant_id)
    }

    pub fn chromosomal_structure_variation(
        cell_contents: impl Into<String>,  
        gene_symbol: impl Into<String>,  
        gene_id: impl Into<String>,  
        variant_id: Option<String>
    ) -> std::result::Result<Self, String> {
        Self::new(cell_contents.into(), gene_symbol.into(), gene_id.into(), &CHROMOSOMAL_STRUCTURE_VARIATION, variant_id)
    }

    pub fn code_as_chromosomal_structure_variation(
        allele: &str,
        dto: &VariantDto
    ) -> std::result::Result<Self, String> {
        Self::chromosomal_structure_variation(allele, dto.gene_symbol(), dto.hgnc_id(), None)
    }


    pub fn code_as_chromosomal_deletion(
        allele: &str, 
        dto: &VariantDto
    ) -> std::result::Result<Self, String> {
        Self::chromosomal_deletion(allele, dto.gene_symbol(), dto.hgnc_id(), None)
    }

    pub fn code_as_chromosomal_inversion(
        allele: &str, 
        dto: &VariantDto
    ) -> std::result::Result<Self, String> {
        Self::chromosomal_inversion(allele, dto.gene_symbol(), dto.hgnc_id(), None)
    }

    pub fn code_as_chromosomal_duplication(
        allele: &str, 
        dto: &VariantDto
    ) -> std::result::Result<Self, String> {
        Self::chromosomal_duplication(allele, dto.gene_symbol(), dto.hgnc_id(), None)
    }

    pub fn code_as_chromosomal_translocation(
        allele: &str, 
        dto: &VariantDto
    ) -> std::result::Result<Self, String> {
        Self::chromosomal_translocation(allele, dto.gene_symbol(), dto.hgnc_id(), None)
    }

    pub fn variant_id(&self) -> &str {
        &self.variant_id
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn gene_symbol(&self) -> &str {
        &self.gene_symbol
    }

    pub fn hgnc_id(&self) -> &str {
        &self.hgnc_id
    }

    pub fn so_id(&self) -> &str {
        &self.so_id
    }

    pub fn so_label(&self) -> &str {
        &self.so_label
    }

    pub fn genotype(&self) -> Option<&str> {
        self.genotype.as_deref()
    }

    
}
