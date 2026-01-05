
use core::fmt;
use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::dto::{cohort_dto::DiseaseData, hgvs_variant::HgvsVariant, hpo_term_dto::HpoTermDuplet, intergenic_variant::IntergenicHgvsVariant, structural_variant::StructuralVariant};


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")] // ensures JSON uses "raw", "transformed", "error"
pub enum EtlCellStatus {
    Raw,
    Transformed,
    Error,
    Ignored,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EtlCellValue {
    pub original: String,
    pub current: String,
    pub status: EtlCellStatus,
    pub error: Option<String>,
}

impl EtlCellValue {
    pub fn new() -> Self {
        Self {
            original: String::default(),
            current: String::default(),
            status: EtlCellStatus::Raw,
            error: None,
        }
    }

    /// Normalizes a single character for ETL ingestion.
    ///
    /// - Unicode whitespace → ASCII space
    /// - Unicode dash variants → ASCII hyphen-minus
    fn normalize_char(c: char) -> char {
        match c {
            // Dash-like characters
            '\u{2010}' // Hyphen
            | '\u{2011}' // Non-breaking hyphen
            | '\u{2012}' // Figure dash
            | '\u{2013}' // En dash
            | '\u{2014}' // Em dash
            | '\u{2015}' // Horizontal bar
            | '\u{2212}' // Minus sign
            | '\u{FE58}' // Small em dash
            | '\u{FE63}' // Small hyphen-minus
            | '\u{FF0D}' // Fullwidth hyphen-minus
                => '-',

            // Any Unicode whitespace
            c if c.is_whitespace() => ' ',
            // ZERO WIDTH SPACE-not handled by above
            '\u{200B}' => ' ', 
            _ => c,
        }
    }


    /// Creates a new `EtlCellValue` from a string-like input, normalizing
    /// Unicode whitespace.
    ///
    /// # Whitespace normalization
    ///
    /// This constructor performs the following normalization steps on the
    /// input string:
    ///
    /// 1. Converts **all Unicode whitespace characters** (including non-breaking
    ///    space `U+00A0`, zero-width spaces, tabs, and newlines) into a single
    ///    ASCII space (`' '`).
    /// 2. Trims leading and trailing whitespace.
    /// 3. Collapses consecutive whitespace into a single space.
    ///
    /// This behavior is intentional and is designed to make the value robust
    /// against common data-ingestion artifacts originating from spreadsheets,
    /// HTML, PDFs, or copy-and-paste operations.
    pub fn from_string<S>(val: S) -> Self
    where
        S: Into<String>,
    {

        let original = val
            .into()
            .chars()
            .map(Self::normalize_char)
            .collect::<String>()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");

        println!("orig={original}");

        Self {
            original,
            current: String::default(),
            status: EtlCellStatus::Raw,
            error: None,
        }
    }


}


impl fmt::Display for EtlCellValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(err) = &self.error {
            write!(
                f,
                "Original: {} - Transformed: {} - Status: {:?} - Error: {}",
                self.original, self.current, self.status, err
            )
        } else {
            write!(
                f,
                "Original: {} - Transformed: {} - Status: {:?}",
                self.original, self.current, self.status
            )
        }
    }
}


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
    pub header: EtlColumnHeader,
    pub values: Vec<EtlCellValue>,
}


impl ColumnDto {
    pub fn new_raw(original_header_contents: &str, size: usize) -> Self {
        Self { 
            id: Uuid::new_v4().to_string(),
            header: EtlColumnHeader::new_raw(original_header_contents), 
            values: Vec::with_capacity(size) 
        }
    }

    pub fn new_hpo_text_mining(size: usize) -> Self {
        Self { 
            id: Uuid::new_v4().to_string(),
            header: EtlColumnHeader::new_hpo_mining(), 
            values: vec![EtlCellValue::new(); size],
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
    pub intergenic_variants: HashMap<String, IntergenicHgvsVariant>,
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_non_breaking_space() {
        let v = EtlCellValue::from_string("foo\u{00A0}bar");
        assert_eq!(v.original, "foo bar");
    }

    #[test]
    fn collapses_multiple_whitespace() {
        let v = EtlCellValue::from_string(" foo\t\n  bar ");
        assert_eq!(v.original, "foo bar");
    }

    #[test]
    fn normalizes_unicode_dashes() {
        let v = EtlCellValue::from_string("TP53\u{2013}related\u{2014}disease");
        assert_eq!(v.original, "TP53-related-disease");
    }

    #[test]
    fn normalizes_minus_sign() {
        let v = EtlCellValue::from_string("−5");
        assert_eq!(v.original, "-5");
    }

    #[test]
    fn preserves_ascii_hyphen() {
        let v = EtlCellValue::from_string("A-B");
        assert_eq!(v.original, "A-B");
    }

    #[test]
    fn removes_zero_width_space() {
        let v = EtlCellValue::from_string("foo\u{200B}bar");
        assert_eq!(v.original, "foo bar");
    }

    #[test]
    fn input_only_whitespace_becomes_empty() {
        let v = EtlCellValue::from_string("\u{00A0}\t\n");
        assert_eq!(v.original, "");
    }

    #[test]
    fn mixed_realistic_excel_input() {
        let v = EtlCellValue::from_string(
            "  BRCA1\u{00A0}\u{2013}\u{00A0}associated\u{2009}cancer  "
        );
        assert_eq!(v.original, "BRCA1 - associated cancer");
    }
}
