use std::collections::HashSet;

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use crate::dto::{hgvs_variant::HgvsVariant, structural_variant::StructuralVariant};

/// A Data Transfer Object for information about a Variant that we want to validate.
/// There are currently two categories of variant
/// 1. HGVS: "Small" variants, such as single nucleotide variants, that are represented with Human Genome Variation Society (HGVS) nomenclature, e.g., c. 123G>T
/// 2. Structural variant: "Large" variants, such as chromosomal deletions, that are represented by free text (DEL of exon 5) and Sequence Ontology (SO) codes
/// As technology and genomic data science progress, it is possible that publications and databases will have more precise notation about many "large"
/// variants, but the genetics literature contains lots of data with imprecise, non-standardized descriptions of structural variants that we want to capture.
/// This struct encapsulates all of the data we expect to get from the front end about either of the variant categories
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VariantDto {
    /// either an HGVS String (e.g., c.123T>G) or a SV String: DEL: deletion of exon 5
    pub variant_string: String,
    /// Key to be use in the HashMap
    pub variant_key: Option<String>,
    /// transcript of reference for the gene of interest (usually MANE) with version number, e.g. NM_000123.2 
    pub transcript: String,
    /// HUGO Gene Nomenclature Committee identifier, e.g., HGNC:123
    pub hgnc_id: String,
    /// Symbol recommended by HGNC, e.g. FBN1
    pub gene_symbol: String,
    /// type of variant category
    pub variant_type: VariantType,
    /// Was this variant validated in the backend?
    pub is_validated: bool,   
    /// How many alleles were reported with this variant in the cohort?
    pub count: u32,
}

impl VariantDto {
     pub fn hgvs_c(
        hgvs: &str, 
        transcript: &str,
        hgnc: &str,
        symbol: &str
    ) -> Self {
        Self {
            variant_string: hgvs.to_string(),
            variant_key: None,
            transcript: transcript.to_string(),
            hgnc_id: hgnc.to_string(),
            gene_symbol: symbol.to_string(),
            variant_type: VariantType::Hgvs,
            is_validated: false,
            count: 0
        }
    }

    /// This is designed to get an SV definition from a legacy template. 
    /// We assign it to the generic SV class. Users can edit this in the front end to 
    /// specify a specific kind of SV. We are not able to do this automatically from the
    /// legacy excel files. TODO: This should be removed once we have processed the legacy excel files.
    pub fn sv(
        label: &str, 
        transcript: &str,
        hgnc: &str,
        symbol: &str,
        sv_type: VariantType
    ) -> Self {
        Self {
            variant_string: label.to_string(),
            variant_key: None,
            transcript: transcript.to_string(),
            hgnc_id: hgnc.to_string(),
            gene_symbol: symbol.to_string(),
            variant_type: sv_type,
            is_validated: false,
            count: 0
        }
    }

    /// Create a VariantDto object for an intergenic variant with gene information
    /// that is, we associated with variant with a certain gene even through it is not located in a transcript
    pub fn hgvs_g(
        hgvs: &str, 
        hgnc: &str,
        symbol: &str
    ) -> Self {
        Self {
            variant_string: hgvs.to_string(),
            variant_key: None,
            transcript: String::default(),
            hgnc_id: hgnc.to_string(),
            gene_symbol: symbol.to_string(),
            variant_type: VariantType::IntergenicHgvs,
            is_validated: false,
            count: 0
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

    pub fn is_validated(&self) -> bool {
        self.is_validated
    }

    pub fn is_structural(&self) -> bool {
        self.variant_type == VariantType::Sv
    }

    pub fn clone_validated(&self) -> Self {
        Self { 
            is_validated: true, 
            ..self.clone()
        }
    }

    pub fn clone_unvalidated(&self) -> Self {
        Self { 
            is_validated: false,
            ..self.clone()
        }
    }

    /// Sort - c. comes first, then n., then intergenic (g.), then structural
    fn variant_string_sort_key(s: &str) -> u8 {
        if s.starts_with("c.") {
            0
        } else if s.starts_with("n.") {
            1
        } else if s.starts_with("g"){
            2
        } else {
            3
        }
    }

    pub fn sort_variant_dtos(variants: &mut [VariantDto]) {
        variants.sort_by(|a, b| {
            let rank_a = Self::variant_string_sort_key(&a.variant_string);
            let rank_b = Self::variant_string_sort_key(&b.variant_string);

            match rank_a.cmp(&rank_b) {
                std::cmp::Ordering::Equal => a.variant_string.cmp(&b.variant_string),
                other => other,
            }
        });
    }

    pub fn is_sv(&self) -> bool {
        IMPRECISE_SV_TYPE_SET.contains(&self.variant_type)
    }

    pub fn is_hgvs(&self) -> bool {
        self.variant_type == VariantType::Hgvs
    }

    pub fn is_intergenic_hgvs(&self) -> bool {
        self.variant_type == VariantType::IntergenicHgvs
    }

    pub fn from_hgvs(hgvs: &HgvsVariant, allele_key: &str) -> Self {
        Self {
            variant_string: hgvs.hgvs().to_string(),
            variant_key: Some(allele_key.to_string()),
            transcript: hgvs.transcript().to_string(),
            hgnc_id: hgvs.hgnc_id().to_string(),
            gene_symbol: hgvs.symbol().to_string(),
            variant_type: VariantType::Hgvs,
            is_validated: false,
            count: 0,
        }
    }

    pub fn not_validated(allele_key: &str) -> Self {
        Self {
            variant_string: format!("na:{}",allele_key),
            variant_key: Some(allele_key.to_string()),
            transcript: String::default(),
            hgnc_id: String::default(),
            gene_symbol: String::default(),
            variant_type: VariantType::Unknown,
            is_validated: false,
            count: 1,
        }
    }

    pub fn from_sv(sv: &StructuralVariant, allele_key: &str) -> Self {
        Self {
            variant_string: sv.label().to_string(),
            variant_key: Some(allele_key.to_string()),
            transcript: sv.transcript().to_string(),
            hgnc_id: sv.hgnc_id().to_string(),
            gene_symbol: sv.gene_symbol().to_string(),
            variant_type: VariantType::Sv,
            is_validated: true,
            count: 0    
        }
    }

    


}


/// The frontend will tell us what kind of variant is being sent to the backend for validation using this enumeration. Note that this DTO is not part of the data model that
/// is serialized or transformed to phenopackets. It is intended to facilitate the
/// flow of information about variants from the back to front end and vice versa.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "UPPERCASE")]
pub enum VariantType {
    /// Small variant represented as HGVS, must start with c. or n.
    Hgvs,
    /// Intergenic HGVS, e.g., promoter or enhancer
    IntergenicHgvs,
    /// chromosomal_deletion
    Del ,
    /// chromosomal_inversion
    Inv,
    /// chromosomal_translocation
    Transl,
    /// chromosomal_duplication
    Dup, 
    /// structural_variation, not specific subtype
    Sv,
    /// structual variant with precise specifications (not implemented yet)
    PreciseSv,
    /// Not yet known or identified.
    Unknown,
}

static IMPRECISE_SV_TYPE_SET: Lazy<HashSet<VariantType>> = Lazy::new(|| {
    let mut sv_set: HashSet<VariantType> = HashSet::new();
    sv_set.insert(VariantType::Del);
    sv_set.insert(VariantType::Inv);
    sv_set.insert(VariantType::Dup);
    sv_set.insert(VariantType::Transl);
    sv_set.insert(VariantType::Sv);
    sv_set
});


#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StructuralVariantDto {
    variant_id: String,
    label: String,
    gene_symbol: String,
    hgnc_id: String,
    so_id: String,
    so_label: String,
    variant_key: String,
}

impl VariantDto {
    

}


