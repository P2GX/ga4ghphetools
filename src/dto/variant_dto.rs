use std::env::var;

use serde::{Deserialize, Serialize};



/// A Data Transfer Object for information about a Variant that we want to validate.
/// There are currently two categories of variant
/// 1. HGVS: "Small" variants, such as single nucleotide variants, that are represented with Human Genome Variation Society (HGVS) nomenclature, e.g., c. 123G>T
/// 2. Structural variant: "Large" variants, such as chromosomal deletions, that are represented by free text (DEL of exon 5) and Sequence Ontology (SO) codes
/// As technology and genomic data science progress, it is possible that publicatiohs and databases will have more precise notation about many "large"
/// variants, but the genetics literature contains lots of data with imprecide, non-standardized descriptions of structural variants that we want to capture.
/// This struct encapsulates all of the data we expect to get from the front end about either of the variant categories
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct VariantDto {
    /// either an HGVS String (e.g., c.123T>G) or a SV String: DEL: deletion of exon 5
    variant_string: String,
    /// transcript of reference for the gene of interest (usually MANE) with version number, e.g. NM_000123.2
    transcript: String,
    /// HUGO Gene Nomenclature Committee identifier, e.g., HGNC:123
    hgnc_id: String,
    /// Symbol recommended by HGNC, e.g. FBN1
    gene_symbol: String
}

impl VariantDto {
    pub fn new(
        variant_string: impl Into<String>,
        transcript: impl Into<String>,
        hgnc_id: impl Into<String>,
        gene_symbol: impl Into<String>,
    ) -> Self {
        Self { 
            variant_string: variant_string.into(), 
            transcript: transcript.into(), 
            hgnc_id: hgnc_id.into(), 
            gene_symbol: gene_symbol.into() 
        }
    }

    pub fn variant_string(&self) -> &str {
        &self.variant_string
    }

    pub fn transcript(&self) -> &str {
        &self.transcript
    }

    pub fn hgnc_id(&self) -> &str {
        &self.hgnc_id
    }

    pub fn gene_symbol(&self) -> &str {
        &self.gene_symbol
    }

}