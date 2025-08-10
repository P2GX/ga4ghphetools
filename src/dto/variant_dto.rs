use std::collections::HashSet;

use once_cell::sync::Lazy;
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
    gene_symbol: String,
    /// Have we validated this variant in the backend?
    validated: bool,
    is_structural: bool,
}

/// The frontend will tell us what kind of variant is being sent to the backend for validation using this enumeration
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "UPPERCASE")]
pub enum VariantValidationType {
    /// Small variant represented as HGVS, must start with c. or n.
    Hgvs,
    /// chromosomal_deletion
    Del ,
    /// chromosomal_inversion
    Inv,
    /// chromosomal_translocation
    Transl,
    /// chromosomal_duplication
    Dup, 
    /// chromosomal insertion (other than duplication)
    Ins,
    /// structural_variation, not specific subtype
    Sv,
    /// structual variant with precise specifications (not implemented yet)
    PreciseSv
}

static IMPRECISE_SV_TYPE_SET: Lazy<HashSet<VariantValidationType>> = Lazy::new(|| {
    let mut sv_set: HashSet<VariantValidationType> = HashSet::new();
    sv_set.insert(VariantValidationType::Del);
    sv_set.insert(VariantValidationType::Inv);
    sv_set.insert(VariantValidationType::Dup);
    sv_set.insert(VariantValidationType::Transl);
    sv_set.insert(VariantValidationType::Ins);
    sv_set.insert(VariantValidationType::Sv);
    sv_set
});


/// A Data Transfer Object for information about a Variant that we want to validate.
/// There are currently two categories of variant
/// 1. HGVS: "Small" variants, such as single nucleotide variants, that are represented with Human Genome Variation Society (HGVS) nomenclature, e.g., c. 123G>T
/// 2. Structural variant: "Large" variants, such as chromosomal deletions, that are represented by free text (DEL of exon 5) and Sequence Ontology (SO) codes
/// As technology and genomic data science progress, it is possible that publicatiohs and databases will have more precise notation about many "large"
/// variants, but the genetics literature contains lots of data with imprecide, non-standardized descriptions of structural variants that we want to capture.
/// This struct encapsulates all of the data we expect to get from the front end about either of the variant categories
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VariantValidationDto {
    /// either an HGVS String (e.g., c.123T>G) or a SV String: DEL: deletion of exon 5
    pub variant_string: String,
    /// transcript of reference for the gene of interest (usually MANE) with version number, e.g. NM_000123.2 
    pub transcript: String,
    /// HUGO Gene Nomenclature Committee identifier, e.g., HGNC:123
    pub hgnc_id: String,
    /// Symbol recommended by HGNC, e.g. FBN1
    pub gene_symbol: String,
    /// type of variant category
    pub validation_type: VariantValidationType
}

impl VariantValidationDto {
    pub fn hgvs_c(
        hgvs: &str, 
        transcript: &str,
        hgnc: &str,
        symbol: &str
    ) -> Self {
        Self { 
            variant_string: hgvs.to_string(), 
            transcript: transcript.to_string(), 
            hgnc_id: hgnc.to_string(), 
            gene_symbol: symbol.to_string(), 
            validation_type: VariantValidationType::Hgvs
        }
    }

    /// This is designed to get an SV definition from a legacy template. 
    /// We assign it to the generic SV class. Users can edit this in the front end to 
    /// specify a specific kind of SV. We are not able to do this automatically from the
    /// legacy excel files. TODO: This should be removed once we have processed the legacy excel files.
    pub fn sv(
        hgvs: &str, 
        transcript: &str,
        hgnc: &str,
        symbol: &str
    ) -> Self {
        Self { 
            variant_string: hgvs.to_string(), 
            transcript: transcript.to_string(), 
            hgnc_id: hgnc.to_string(), 
            gene_symbol: symbol.to_string(), 
            validation_type: VariantValidationType::Sv
        }
    }

    pub fn is_hgvs(&self) -> bool {
        return self.validation_type == VariantValidationType::Hgvs
    }

    pub fn is_sv(&self) -> bool {
        return IMPRECISE_SV_TYPE_SET.contains(&self.validation_type);
    }
}


#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StructuralVariantDto {
    variant_id: String,
    label: String,
    gene_symbol: String,
    hgnc_id: String,
    so_id: String,
    so_label: String,
}

impl VariantDto {
    pub fn new_hgvs(
        variant_string: impl Into<String>,
        transcript: impl Into<String>,
        hgnc_id: impl Into<String>,
        gene_symbol: impl Into<String>,
    ) -> Self {
        Self { 
            variant_string: variant_string.into(), 
            transcript: transcript.into(), 
            hgnc_id: hgnc_id.into(), 
            gene_symbol: gene_symbol.into(),
            validated: false,
            is_structural: false
        }
    }

    pub fn new_sv(
        variant_string: impl Into<String>,
        transcript: impl Into<String>,
        hgnc_id: impl Into<String>,
        gene_symbol: impl Into<String>,
    ) -> Self {
        Self { 
            variant_string: variant_string.into(), 
            transcript: transcript.into(), 
            hgnc_id: hgnc_id.into(), 
            gene_symbol: gene_symbol.into(),
            validated: false,
            is_structural: true
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

    pub fn validated(&self) -> bool {
        self.validated
    }

    pub fn is_structural(&self) -> bool {
        self.is_structural
    }

    pub fn clone_validated(&self) -> Self {
        Self { 
            variant_string:  self.variant_string.clone(), 
            transcript: self.transcript.clone(), 
            hgnc_id: self.hgnc_id.clone(), 
            gene_symbol: self.gene_symbol.clone(), 
            validated: true, 
            is_structural: self.is_structural
        }
    }

    pub fn clone_unvalidated(&self) -> Self {
        Self { 
            variant_string:  self.variant_string.clone(), 
            transcript: self.transcript.clone(), 
            hgnc_id: self.hgnc_id.clone(), 
            gene_symbol: self.gene_symbol.clone(), 
            validated: false, 
            is_structural: self.is_structural
        }
    }

    fn variant_string_sort_key(s: &str) -> u8 {
        if s.starts_with("c.") {
            0
        } else if s.starts_with("n.") {
            1
        } else {
            2
        }
    }


    pub fn numerical_key(&self) -> u32 {
        if self.is_structural() {
            return 0;
        }
        if self.variant_string.len() < 2 {
            return 0;
        }
        self.variant_string[2..]  // skip "c."
            .chars()
            .take_while(|c| c.is_ascii_digit())
            .collect::<String>()
            .parse::<u32>()
            .unwrap_or(0) // fallback if we cannot parse
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

}


#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct VariantListDto {
    pub variant_dto_list: Vec<VariantDto>
}


impl VariantListDto {
    pub fn new(dto_list: Vec<VariantDto>) -> Self {
        Self { variant_dto_list: dto_list }
    }
    
}