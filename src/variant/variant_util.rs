use std::{fmt, str::FromStr};
use lazy_static::lazy_static;
use crate::variant::acmg::AcmgPathogenicityClassification;
use rand::Rng;
use ontolius::{term::{simple::SimpleMinimalTerm, Term}, TermId};

use super::{hgvs_variant::HgvsVariant, structural_variant::StructuralVariant};


#[derive(Debug, Clone)]
pub enum Genotype {
    Heterozygous,
    Homozygous,
    Hemizygous,
}

impl fmt::Display for Genotype {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let label = match self {
            Genotype::Heterozygous => "heterozygous",
            Genotype::Homozygous => "homozygous",
            Genotype::Hemizygous => "hemizygous",
        };
        write!(f, "{}", label)
    }
}

lazy_static! {
    pub static ref HETEROZYGOUS: SimpleMinimalTerm = SimpleMinimalTerm::new(
        TermId::from_str("GENO:0000135").unwrap(),
        "heterozygous".to_string(),
        vec![], 
        false 
    );

    pub static ref HOMOZYGOUS: SimpleMinimalTerm = SimpleMinimalTerm::new(
        TermId::from_str("GENO:0000136").unwrap(),
        "homozygous".to_string(),
        vec![], 
        false 
    );

    pub static ref HEMIZYGOUS: SimpleMinimalTerm = SimpleMinimalTerm::new(
        TermId::from_str("GENO:0000134").unwrap(),
        "hemizygous".to_string(),
        vec![], 
        false 
    );

}


pub fn generate_id() -> String {
    rand::rng()
        .sample_iter(&rand::distr::Alphanumeric)
        .take(24)
        .map(char::from)
        .collect()
}