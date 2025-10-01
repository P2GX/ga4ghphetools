
use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::dto::{cohort_dto::DiseaseData, hgvs_variant::HgvsVariant, hpo_term_dto::HpoTermDuplet, structural_variant::StructuralVariant};




/// DTOs for transforming external Excel tables 
/// We ingest an Excel file and transform it column by column to a structure we can use to import phenopackets.
/// Each column will be transformed one by one. Columns start off as RAW and then are changed to the other
///types listed here
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum EtlColumnType {
    Raw,
    FamilyId,
    PatientId,
    SingleHpoTerm,
    MultipleHpoTerm,
    GeneSymbol,
    Variant,
    AgeOfOnset,
    AgeAtLastEncounter,
    Sex,
    Deceased,
    HpoTextMining,
    Ignore
}

/// Allowed values for sex
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum SexCode {
    M,
    F,
    U,
    O,
}

/// Allowed values for the deceased column
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DeceasedCode {
    Yes,
    No,
    Na,
}


/// Tracks original/current header naming and semantic metadata
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EtlColumnHeader {
    pub original: String,
    pub current: Option<String>,
    pub column_type: EtlColumnType,
    pub hpo_terms: Option<Vec<HpoTermDuplet>>,
}

impl EtlColumnHeader {
    pub fn new_raw(original_column_header: &str) -> Self {
        Self { 
            original: original_column_header.to_string(), 
            current: None, 
            column_type: EtlColumnType::Raw, 
            hpo_terms: None 
        }
    }

    pub fn new_hpo_mining() -> Self {
        Self { 
            original: "HPO Text Mining".to_string(), 
            current: None, 
            column_type: EtlColumnType::HpoTextMining, 
            hpo_terms: None 
        }
    }
}

 #[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
 #[serde(rename_all = "camelCase")]
 pub struct ColumnDto {
    /// A unique, randomly generated id that we use to index columns in the front end
    pub id: String,
    /// true if the cell contents have been transformed
    pub transformed: bool,
    pub header: EtlColumnHeader,
    pub values: Vec<String>,
}


impl ColumnDto {
    pub fn new_raw(original_header_contents: &str, size: usize) -> Self {
        Self { 
            id: Uuid::new_v4().to_string(),
            transformed:false, 
            header: EtlColumnHeader::new_raw(original_header_contents), 
            values: Vec::with_capacity(size) 
        }
    }

    pub fn new_hpo_text_mining(size: usize) -> Self {
        Self { 
            id: Uuid::new_v4().to_string(),
            transformed:false, 
            header: EtlColumnHeader::new_hpo_mining(), 
            values: Vec::with_capacity(size) 
        }
    }
}

/// The main structure to represent the actual data from an external table
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ColumnTableDto {
    pub file_name: String,
    pub columns: Vec<ColumnDto>,
   
}


/// The main structure for "deciphering" external data tables (e.g., supplemental tables about cohorts)
/// We only support Mendelian cohorts
/// This represents the product of transformation, and also includes DiseaseData and pub med data
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EtlDto {
    pub table: ColumnTableDto,
    pub disease: Option<DiseaseData>,
    pub pmid: Option<String>,
    pub title: Option<String>,
    /// Validated HGVS variants.
    pub hgvs_variants: HashMap<String, HgvsVariant>,
    /// Validated structural (symbolic) variants
    pub structural_variants: HashMap<String, StructuralVariant>,
}



