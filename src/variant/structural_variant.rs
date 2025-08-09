use ontolius::{term::simple::SimpleMinimalTerm, TermId};
use rand::{distr::Alphanumeric, Rng};
use serde::{Serialize, Deserialize};
use std::{fmt, str::FromStr};
use once_cell::sync::Lazy;
use crate::dto::variant_dto::{VariantValidationDto, VariantValidationType};
const ACCEPTABLE_GENOMES: [&str; 2] = [ "GRCh38",  "hg38"];


/// The frontend will tell us what kind of variant is being sent to the backend for validation using this enumeration
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SvType {
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
}

impl fmt::Display for SvType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            SvType::Del => "DEL",
            SvType::Inv => "INV",
            SvType::Transl => "TRANSL",
            SvType::Dup => "DUP",
            SvType::Ins => "INS",
            SvType::Sv => "SV",
        };
        write!(f, "{}", s)
    }
}

impl FromStr for SvType {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_uppercase().as_str() {
            "DEL" => Ok(SvType::Del),
            "INV" => Ok(SvType::Inv),
            "TRANSL" => Ok(SvType::Transl),
            "DUP" => Ok(SvType::Dup),
            "INS" => Ok(SvType::Ins),
            "SV" => Ok(SvType::Sv),
            _ => Err(()),
        }
    }
}

impl TryFrom<VariantValidationType> for SvType {
    type Error = String;
    fn try_from(vvt: VariantValidationType) -> Result<Self, Self::Error> {
        match vvt {
            VariantValidationType::Del => Ok(Self::Del),
            VariantValidationType::Dup => Ok(Self::Dup),
            VariantValidationType::Ins => Ok(Self::Ins),
            VariantValidationType::Inv => Ok(Self::Inv),
            VariantValidationType::Transl => Ok(Self::Transl),
            VariantValidationType::Sv => Ok(Self::Sv),
            VariantValidationType::Hgvs => Err("Cannot convert ValidationType HGVS into SV type".to_string()),
            VariantValidationType::PreciseSv => Err("Cannot convert ValidationType PreciseSv into SV type".to_string()),
        }
    }
}

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

/// Representation of a "symbolic" SV, such as DEL Ex3-5, that is without precise positions/definition
/// This is common in the literature so we capture this using a label to represent the original description
/// used in the publication, and additional specify the gene symbol, HGNS id of the gene deemed to be most affected
/// by the SV, and the SV type.
/// The identifier is provided by the export function to GHA4GH Phenopacket Schema
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StructuralVariant {
    label: String,
    gene_symbol: String,
    hgnc_id: String,
    sv_type: SvType,
    chromosome: String
}

impl StructuralVariant {
    pub fn new(
        cell_contents: String, 
        gene_symbol: String, 
        gene_id: String, 
        sv_type: SvType, 
        chromosome: String) 
    -> std::result::Result<Self, String> {
        if gene_symbol.is_empty() {
            return Err(format!("Malformed structural variant {cell_contents}: Need to pass a valid gene symbol!"));
        }
        if gene_id.is_empty() {
            return Err(format!("Malformed structural variant {cell_contents}: Need to pass a valid HGNC gene id!"));
        }
        Ok(Self {
            label: cell_contents.trim().to_string(),
            gene_symbol,
            hgnc_id: gene_id,
            sv_type: sv_type,
            chromosome
        })
    }

    

    // Static Constructors for Specific Variants
    pub fn chromosomal_deletion(
        cell_contents: impl Into<String>, 
        gene_symbol: impl Into<String>, 
        gene_id: impl Into<String>, 
        chrom: String) -> std::result::Result<Self, String> {
        Self::new(cell_contents.into(), gene_symbol.into(), gene_id.into(), SvType::Del, chrom)
    }

    pub fn chromosomal_duplication(
        cell_contents: impl Into<String>,  
        gene_symbol: impl Into<String>,  
        gene_id: impl Into<String>,  
        chrom: String) -> std::result::Result<Self, String> {
        Self::new(cell_contents.into(), gene_symbol.into(), gene_id.into(), SvType::Dup, chrom)
    }

    pub fn chromosomal_inversion(
        cell_contents: impl Into<String>,  
        gene_symbol: impl Into<String>,  
        gene_id: impl Into<String>,  
        chrom: String
    ) -> std::result::Result<Self, String> {
        Self::new(cell_contents.into(), gene_symbol.into(), gene_id.into(), SvType::Inv, chrom)
    }

     pub fn chromosomal_insertion(
        cell_contents: impl Into<String>,  
        gene_symbol: impl Into<String>,  
        gene_id: impl Into<String>,  
        chrom: String
    ) -> std::result::Result<Self, String> {
        Self::new(cell_contents.into(), gene_symbol.into(), gene_id.into(), SvType::Ins,chrom)
    }

    pub fn chromosomal_translocation(
        cell_contents: impl Into<String>,  
        gene_symbol: impl Into<String>,  
        gene_id: impl Into<String>,  
        chrom: String
    ) -> std::result::Result<Self, String> {
        Self::new(cell_contents.into(), gene_symbol.into(), gene_id.into(), SvType::Transl, chrom)
    }

    pub fn chromosomal_structure_variation(
        cell_contents: impl Into<String>,  
        gene_symbol: impl Into<String>,  
        gene_id: impl Into<String>,  
        chrom: String
    ) -> std::result::Result<Self, String> {
        Self::new(cell_contents.into(), gene_symbol.into(), gene_id.into(), SvType::Sv, chrom)
    }

    pub fn code_as_chromosomal_structure_variation(
        vv_dto: VariantValidationDto,
        chrom: String
    ) -> std::result::Result<Self, String> {
        Self::chromosomal_structure_variation(vv_dto.variant_string, vv_dto.gene_symbol, vv_dto.hgnc_id,  chrom)
    }


    pub fn code_as_chromosomal_deletion(
        vv_dto: VariantValidationDto,
        chrom: String
    ) -> std::result::Result<Self, String> {
        Self::chromosomal_deletion(vv_dto.variant_string, vv_dto.gene_symbol, vv_dto.hgnc_id, chrom)
    }

    pub fn code_as_chromosomal_inversion(
        vv_dto: VariantValidationDto,
        chrom: String
    ) -> std::result::Result<Self, String> {
        Self::chromosomal_inversion(vv_dto.variant_string, vv_dto.gene_symbol, vv_dto.hgnc_id, chrom)
    }

      pub fn code_as_chromosomal_insertion(
        vv_dto: VariantValidationDto,
        chrom: String
    ) -> std::result::Result<Self, String> {
        Self::chromosomal_insertion(vv_dto.variant_string, vv_dto.gene_symbol, vv_dto.hgnc_id, chrom)
    }

    pub fn code_as_chromosomal_duplication(
        vv_dto: VariantValidationDto,
        chrom: String
    ) -> std::result::Result<Self, String> {
        Self::chromosomal_duplication(vv_dto.variant_string, vv_dto.gene_symbol, vv_dto.hgnc_id, chrom)
    }

    pub fn code_as_chromosomal_translocation(
        vv_dto: VariantValidationDto,
        chrom: String
    ) -> std::result::Result<Self, String> {
        Self::chromosomal_translocation(vv_dto.variant_string, vv_dto.gene_symbol, vv_dto.hgnc_id, chrom)
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

    /* provide a key for the variant that we will use for the HashMap */
    pub fn variant_key(&self) -> String {
        let clean_label: String = self
            .label
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '_' })
            .collect();
        format!("{}_{}_{}", self.gene_symbol, self.sv_type, clean_label )
    } 
}




mod tests {
    use crate::variant::structural_variant::StructuralVariant;

    #[test]
    pub fn test_variant_key() {
        let sv = StructuralVariant::chromosomal_structure_variation("DEL Ex 4", "FBN1", "HGNC:123", "15".to_string()).unwrap();
        let key = sv.variant_key();
        assert_eq!("FBN1_SV_DEL_Ex_4", key);
    }


}