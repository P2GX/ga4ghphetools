use std::{fmt, str::FromStr};
use lazy_static::lazy_static;
use crate::variant::acmg::AcmgPathogenicityClassification;

use ontolius::{term::{simple::SimpleMinimalTerm, Term}, TermId};


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

pub trait Variant {
    //fn to_ga4gh_variant_interpretation(&self, acmg: Option<&str>) -> GA4GHVariantInterpretation;

    fn set_heterozygous(&mut self);
    fn set_homozygous(&mut self);
    fn set_hemizygous(&mut self);
   

    fn get_genotype_term(gt: Option<&Genotype>) -> Option<SimpleMinimalTerm> {
        match gt {
            Some(Genotype::Heterozygous) => Some(HETEROZYGOUS.clone()),
            Some(Genotype::Homozygous) => Some(HOMOZYGOUS.clone()),
            Some(Genotype::Hemizygous) => Some(HEMIZYGOUS.clone()),
            None => None,
        }
    }
}
