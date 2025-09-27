//! StructuralVariant
//! Representation of a "symbolic structural variant", e.g., DEL Ex 5-6, in a specified gene.
//! This representation is common in the Human Genetics literature. It will be preferable
//! to capture more precise information when possible.

use phenopackets::schema::v2::core::OntologyClass;
use serde::{Serialize, Deserialize};
use std::{fmt, str::FromStr};
use once_cell::sync::Lazy;
use crate::dto::variant_dto::{VariantDto, VariantType};


/// The frontend will tell us what kind of variant is being sent to the backend for validation using this enumeration
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum SvType {
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
}

impl fmt::Display for SvType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            SvType::Del => "DEL",
            SvType::Inv => "INV",
            SvType::Transl => "TRANSL",
            SvType::Dup => "DUP",
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
            "SV" => Ok(SvType::Sv),
            _ => Err(()),
        }
    }
}

impl TryFrom<VariantType> for SvType {
    type Error = String;
    fn try_from(vvt: VariantType) -> Result<Self, Self::Error> {
        match vvt {
            VariantType::Del => Ok(Self::Del),
            VariantType::Dup => Ok(Self::Dup),
            VariantType::Inv => Ok(Self::Inv),
            VariantType::Transl => Ok(Self::Transl),
            VariantType::Sv => Ok(Self::Sv),
            VariantType::Hgvs => Err("Cannot convert ValidationType HGVS into SV type".to_string()),
            VariantType::PreciseSv => Err("Cannot convert ValidationType PreciseSv into SV type".to_string()),
            VariantType::Unknown => Err("Cannot convert unknown into SvType".to_string())
        }
    }
}

static CHROMOSOMAL_STRUCTURE_VARIATION: Lazy<OntologyClass> = Lazy::new(|| {
    OntologyClass {
        id: "SO:1000183".to_string(),
        label: "chromosome_structure_variation".to_string(),
    }
});

static CHROMOSOMAL_TRANSLOCATION: Lazy<OntologyClass> = Lazy::new(|| {
    OntologyClass{
        id: "SO:1000044".to_string(),
        label: "chromosomal_translocation".to_string(),
    }
});

static CHROMOSOMAL_DELETION: Lazy<OntologyClass> = Lazy::new(|| {
    OntologyClass {
        id: "SO:1000029".to_string(),
        label: "chromosomal_deletion".to_string(),
    }
});

static CHROMOSOMAL_DUPLICATION: Lazy<OntologyClass> = Lazy::new(|| {
     OntologyClass {
        id: "SO:1000037".to_string(),
        label:"chromosomal_duplication".to_string(),
     }
});

static CHROMOSOMAL_INVERSION: Lazy<OntologyClass> = Lazy::new(|| {
    OntologyClass{
        id: "SO:1000030".to_string(),
        label: "chromosomal_inversion".to_string(),
    }
});

/// Representation of a "symbolic" SV, such as DEL Ex3-5, that is without precise positions/definition
/// This is common in the literature so we capture this using a label to represent the original description
/// used in the publication, and additional specify the gene symbol, HGNS id of the gene deemed to be most affected
/// by the SV, and the SV type.
/// The identifier is provided by the export function to GHA4GH Phenopacket Schema
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StructuralVariant {
    /// An unstructured description of the SV, e.g., DEL Ex5-7 (taken from original publication)
    label: String,
    /// HGNC-approvated symbol of the gene affected or deemed most affected by the SV
    gene_symbol: String,
    /// Transcript of reference for the gene
    transcript: String,
    /// HGNC identifier of the gene of reference
    hgnc_id: String,
    /// Category of structural variant
    sv_type: SvType,
    /// Chromosome on which the gene is located
    chromosome: String,
    /// Key used to specify variant in HashMap. We will additionally use the key
    /// as the variant ID when exporting to GA4GH phenopacket.
    variant_key: String,
}

impl StructuralVariant {
    pub fn new(
        cell_contents: String, 
        gene_symbol: String, 
        transcript: String,
        gene_id: String, 
        sv_type: SvType, 
        chromosome: String,
        ) 
    -> std::result::Result<Self, String> {
        if gene_symbol.is_empty() {
            return Err(format!("Malformed structural variant {cell_contents}: Need to pass a valid gene symbol!"));
        }
        if gene_id.is_empty() {
            return Err(format!("Malformed structural variant {cell_contents}: Need to pass a valid HGNC gene id!"));
        }
        let label: String = cell_contents
            .trim()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '_' })
            .collect();
        let v_key = Self::generate_variant_key(&label, &gene_symbol, sv_type);
        Ok(Self {
            label: label,
            gene_symbol,
            transcript,
            hgnc_id: gene_id,
            sv_type: sv_type,
            chromosome,
            variant_key: v_key
        })
    }

     /* provide a key for the variant that we will use for the HashMap */
    pub fn generate_variant_key(label: &str, symbol: &str, sv_type: SvType) -> String {
        format!("{}_{}_{}", symbol, sv_type, label )
    } 

    // Static Constructors for Specific Variants
    pub fn chromosomal_deletion(
        cell_contents: impl Into<String>, 
        gene_symbol: impl Into<String>, 
        transcript: impl Into<String>,
        gene_id: impl Into<String>, 
        chrom: String) -> std::result::Result<Self, String> {
        Self::new(cell_contents.into(), gene_symbol.into(), transcript.into(), gene_id.into(), SvType::Del, chrom)
    }

    pub fn chromosomal_duplication(
        cell_contents: impl Into<String>,  
        gene_symbol: impl Into<String>, 
        transcript: impl Into<String>, 
        gene_id: impl Into<String>,  
        chrom: String) -> std::result::Result<Self, String> {
        Self::new(cell_contents.into(), gene_symbol.into(), transcript.into(), gene_id.into(), SvType::Dup, chrom)
    }

    pub fn chromosomal_inversion(
        cell_contents: impl Into<String>,  
        gene_symbol: impl Into<String>,  
        transcript: impl Into<String>,
        gene_id: impl Into<String>,  
        chrom: String
    ) -> std::result::Result<Self, String> {
        Self::new(cell_contents.into(), gene_symbol.into(), transcript.into(), gene_id.into(), SvType::Inv, chrom)
    }

    pub fn chromosomal_translocation(
        cell_contents: impl Into<String>,  
        gene_symbol: impl Into<String>, 
        transcript: impl Into<String>, 
        gene_id: impl Into<String>,  
        chrom: String
    ) -> std::result::Result<Self, String> {
        Self::new(cell_contents.into(), gene_symbol.into(), transcript.into(), gene_id.into(), SvType::Transl, chrom)
    }

    pub fn chromosomal_structure_variation(
        cell_contents: impl Into<String>,  
        gene_symbol: impl Into<String>, 
        transcript: impl Into<String>, 
        gene_id: impl Into<String>,  
        chrom: String
    ) -> std::result::Result<Self, String> {
        Self::new(cell_contents.into(), gene_symbol.into(), transcript.into(), gene_id.into(), SvType::Sv, chrom)
    }

    pub fn code_as_chromosomal_structure_variation(
        vv_dto: VariantDto,
        chrom: String
    ) -> std::result::Result<Self, String> {
        Self::chromosomal_structure_variation(vv_dto.variant_string, vv_dto.gene_symbol, vv_dto.transcript, vv_dto.hgnc_id,  chrom)
    }


    pub fn code_as_chromosomal_deletion(
        vv_dto: VariantDto,
        chrom: String
    ) -> std::result::Result<Self, String> {
        Self::chromosomal_deletion(vv_dto.variant_string, vv_dto.gene_symbol, vv_dto.transcript, vv_dto.hgnc_id, chrom)
    }

    pub fn code_as_chromosomal_inversion(
        vv_dto: VariantDto,
        chrom: String
    ) -> std::result::Result<Self, String> {
        Self::chromosomal_inversion(vv_dto.variant_string, vv_dto.gene_symbol, vv_dto.transcript, vv_dto.hgnc_id, chrom)
    }

    pub fn code_as_chromosomal_duplication(
        vv_dto: VariantDto,
        chrom: String
    ) -> std::result::Result<Self, String> {
        Self::chromosomal_duplication(vv_dto.variant_string, vv_dto.gene_symbol, vv_dto.transcript, vv_dto.hgnc_id, chrom)
    }

    pub fn code_as_chromosomal_translocation(
        vv_dto: VariantDto,
        chrom: String
    ) -> std::result::Result<Self, String> {
        Self::chromosomal_translocation(vv_dto.variant_string, vv_dto.gene_symbol, vv_dto.transcript, vv_dto.hgnc_id, chrom)
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn gene_symbol(&self) -> &str {
        &self.gene_symbol
    }

    pub fn transcript(&self) -> &str {
        &self.transcript
    }

    pub fn hgnc_id(&self) -> &str {
        &self.hgnc_id
    }

    pub fn variant_key(&self) -> &str {
        &self.variant_key
    }

    /// Return true iff the variant is X chromosomal
    /// We use this to determine if the variant is hemizygous
    pub fn is_x_chromosomal(&self) -> bool {
        return self.chromosome.contains("X")
    }

    pub fn get_sequence_ontology_term(&self) -> OntologyClass {
        match &self.sv_type {
            SvType::Del => CHROMOSOMAL_DELETION.clone(),
            SvType::Inv => CHROMOSOMAL_INVERSION.clone(),
            SvType::Transl => CHROMOSOMAL_TRANSLOCATION.clone(),
            SvType::Dup => CHROMOSOMAL_DUPLICATION.clone(),
            SvType::Sv => CHROMOSOMAL_STRUCTURE_VARIATION.clone(),
        }
    }

   
}




mod tests {

    #[test]
    pub fn test_variant_key() {
        let sv = crate::dto::structural_variant::StructuralVariant::chromosomal_structure_variation(
            "DEL Ex 4", 
            "FBN1",
            "NM_000138.5", 
            "HGNC:123", 
            "15".to_string()
        ).unwrap();
        assert_eq!("FBN1_SV_DEL_Ex_4", sv.variant_key);
    }


    #[test]
    pub fn test_sv_ingest() {
        // test ingest of the following SV from a legacy Excel file
        let cell_contents = "NC_000014.9:g.21220392_21352183del(NM_004500.4:c.-82945_366-7272del)";
        let sv = crate::dto::structural_variant::StructuralVariant::chromosomal_structure_variation(
            cell_contents, 
            "HNRNPC",
            "NM_004500.4", 
            "HGNC:5035", 
            "15".to_string()
        ).unwrap();
        assert_eq!("HNRNPC_SV_NC_000014_9_g_21220392_21352183del_NM_004500_4_c__82945_366_7272del_", sv.variant_key);
    }


}