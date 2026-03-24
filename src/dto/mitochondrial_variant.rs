
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
enum Tissue {
    Heart,
    SkeletalMuscle,
    Fibroblast,
}


#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TissueAlleleFraction {
    /// tissue in which mt was measured (UBERON id)
    tissue: Tissue,
    /// percentage of mitochondria with variant
    percentage: f32
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
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