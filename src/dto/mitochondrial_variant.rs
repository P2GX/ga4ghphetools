
use std::cmp::Ordering;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
enum Tissue {
    Heart,
    SkeletalMuscle,
    Fibroblast,
}


#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, PartialOrd)]
#[serde(rename_all = "camelCase")]
pub struct TissueAlleleFraction {
    /// tissue in which mt was measured (UBERON id)
    tissue: Tissue,
    /// percentage of mitochondria with variant
    percentage: f32
}

impl Eq for TissueAlleleFraction {}

impl Ord for TissueAlleleFraction {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Sort by tissue first
        self.tissue.cmp(&other.tissue)
            // Then sort by percentage using total_cmp
            .then_with(|| self.percentage.total_cmp(&other.percentage))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MitochondrialVariant {
    /// Genome build, e.g., hg38
    assembly: String,
    /// Position on the chromosome
    position: u32,
    /// Reference allele
    ref_allele: String,
    /// Alternate allele
    alt_allele: String,
    /// Gene symbol, e.g., FBN1
    symbol: String,
    /// HUGO Gene Nomenclature Committee identifier, e.g., HGNC:3603
    hgnc_id: String,
    /// HGVS Nomenclature, e.g., m.8242G>T
    hgvs: String,
    /// ...
    plasmy: Option<Vec<TissueAlleleFraction>>,
    /// Key to specify this variant in the HGVS HashMap of the CohortDto
    variant_key: String 
}


/// Sort by position, then ref, then alt
impl Ord for MitochondrialVariant {
    fn cmp(&self, other: &Self) -> Ordering {
        self.position.cmp(&other.position)
         .then_with(|| self.ref_allele.cmp(&other.ref_allele))
            .then_with(|| self.alt_allele.cmp(&other.alt_allele))
           
    }
}

impl PartialOrd for MitochondrialVariant {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}