
 use serde::{Deserialize, Serialize};

use crate::dto::{cohort_dto::DiseaseData, hpo_term_dto::HpoTermDuplet};




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

/// Metadata that can be associated with a column header.
/// Payload varies depending on `EtlColumnType`.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", tag = "kind", content = "data")]
pub enum ColumnMetadata {
    /// Default, for columns we will not use for transformation
    Raw,
    /// List of all HPO Terms used in the cells of the template
    HpoTerms(Vec<HpoTermDuplet>),
    /// Gene and transcript of reference for current column
    GeneTranscript { gene_symbol: String, transcript_id: String },
    /// Male, Female, Other, Unknown: MFOU
    Sex { code: SexCode },
    /// Deceased: yes, no, na
    Deceased { code: DeceasedCode },
    FreeText(String),
}

/// Tracks original/current header naming and semantic metadata
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EtlColumnHeader {
    pub original: String,
    pub current: Option<String>,
    pub column_type: EtlColumnType,
    pub metadata: ColumnMetadata,
}

impl EtlColumnHeader {
    pub fn new_raw(original_column_header: &str) -> Self {
        Self { 
            original: original_column_header.to_string(), 
            current: None, 
            column_type: EtlColumnType::Raw, 
            metadata: ColumnMetadata::Raw 
        }
    }
}

 #[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
 #[serde(rename_all = "camelCase")]
 pub struct ColumnDto {
    pub transformed: bool,
    pub header: EtlColumnHeader,
    pub values: Vec<String>,
}


impl ColumnDto {
    pub fn new_raw(original_header_contents: &str, size: usize) -> Self {
        Self { 
            transformed:false, 
            header: EtlColumnHeader::new_raw(original_header_contents), 
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
pub struct EtlDto {
    pub table: ColumnTableDto,
    pub disease: DiseaseData,
    pub pmid: String,
    pub title: String,
}



