

/*
static EXPECTED_H1_FIELDS: [&str; NUMBER_OF_CONSTANT_HEADER_FIELDS]= [
    "PMID", "title", "individual_id", "comment", "disease_id", "disease_label", 
    "HGNC_id", "gene_symbol", "transcript", "allele_1", "allele_2", "variant.comment", 
    "age_of_onset", "age_at_last_encounter", "deceased", "sex", "HPO"];
/// The constant header fields for the second row of the pyphetools template file
const EXPECTED_H2_FIELDS: [&str; NUMBER_OF_CONSTANT_HEADER_FIELDS]= [
    "CURIE", "str", "str", "optional", "CURIE", "str", 
    "CURIE",  "str", "str", "str", "str", "optional", 
    "age", "age", "yes/no/na", "M:F:O:U", "na"];
     */

use std::fmt::format;

use ontolius::TermId;
use regex::Regex;
use once_cell::sync::Lazy;
use super::header_duplet::HeaderDuplet;


static PMID_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^PMID:\d+$").unwrap());
static NO_LEADING_TRAILING_WS: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\S.*\S$|^\S$").unwrap());
static DISEASE_ID_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(OMIM|MONDO):\d+$").unwrap());
static HGNC_ID_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^HGNC:\d+$").unwrap());
static TRANSCRIPT_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(NM_|ENST)\d+\.\d+$").unwrap());
static DECEASED_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(yes|no|na)").unwrap());
static SEX_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(M|F|O|U)").unwrap());
static SEPARATOR_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"na").unwrap());


/// These fields are always required by our template
const NUMBER_OF_CONSTANT_HEADER_FIELDS: usize = 17; 
static EXPECTED_H1_FIELDS: [&str; NUMBER_OF_CONSTANT_HEADER_FIELDS] =  [
    "PMID", "title", "individual_id", "comment", "disease_id", 
    "disease_label", "HGNC_id", "gene_symbol", "transcript", "allele_1", "allele_2", 
    "variant.comment", "age_of_onset", "age_at_last_encounter", "deceased", "sex", "HPO"
    ];
const EXPECTED_H2_FIELDS: [&str; NUMBER_OF_CONSTANT_HEADER_FIELDS] = [
    "CURIE", "str", "str", "optional", "CURIE", "str", "CURIE", 
    "str", "str", "str", "str", "optional", "age", 
    "age", "yes/no/na", "M:F:O:U", "na"
    ];

trait PptCellValidator {
    fn validate(&self, value: &str) -> bool;
}


pub enum ColumnType {
    PmidColumn,
    TitleColumn,
    IndividualIdColumn,
    IndividualCommentColumn,
    DiseaseIdColumn,
    DiseaseLabelColumn,
    HgncIdColumn,
    GeneSymbolColumn,
    TranscriptColumn,
    AlleleOneColumn,
    AlleleTwoColumn,
    VariantCommentColumn,
    AgeOfOnsetColumn,
    AgeAtLastExaminationColumn,
    DeceasedColumn,
    SexColumn,
    SeparatorColumn,
    HpoTermColumn,
    NtrRequestColumn
}

impl PptCellValidator for PptColumn {
    fn validate(&self, value: &str) -> bool {
        match &self.column_type {
            ColumnType::PmidColumn => PMID_REGEX.is_match(value),
            ColumnType::TitleColumn => NO_LEADING_TRAILING_WS.is_match(value),
            ColumnType::IndividualIdColumn => NO_LEADING_TRAILING_WS.is_match(value),
            ColumnType::IndividualCommentColumn => true,
            ColumnType::DiseaseIdColumn => DISEASE_ID_REGEX.is_match(value),
            ColumnType::DiseaseLabelColumn => NO_LEADING_TRAILING_WS.is_match(value),
            ColumnType::HgncIdColumn => HGNC_ID_REGEX.is_match(value),
            ColumnType::GeneSymbolColumn => NO_LEADING_TRAILING_WS.is_match(value),
            ColumnType::TranscriptColumn => TRANSCRIPT_REGEX.is_match(value),
            ColumnType::AlleleOneColumn => NO_LEADING_TRAILING_WS.is_match(value), //TODO BETTER FUNCTION
            ColumnType::AlleleTwoColumn => NO_LEADING_TRAILING_WS.is_match(value),
            ColumnType::AgeOfOnsetColumn => {
                true // TODO BETTER FUNCTION
            },
            ColumnType::AgeAtLastExaminationColumn => {
                true // TODO BETTER FUNCTION
            }
            ColumnType::DeceasedColumn => DECEASED_REGEX.is_match(value),
            ColumnType::SexColumn => SEX_REGEX.is_match(value),
            ColumnType::SeparatorColumn => SEPARATOR_REGEX.is_match(value),

            _ => false
            
        }
        
    }
}

impl PptCellValidator for crate::pptcolumn::ppt_column::ColumnType {
    fn validate(&self, value: &str) -> bool {
        PMID_REGEX.is_match(value)
    }
}

pub struct PptColumn {
    column_type: ColumnType,
    header_duplet: HeaderDuplet,
    column_data: Vec<String>,
}


impl PptColumn {

    pub fn new(column_type: ColumnType, header1: &str, header2: &str) -> Self {
        let hd = HeaderDuplet::new(header1, header2);
        PptColumn { column_type:column_type, header_duplet: hd, column_data: Vec::new() }
    }


    pub fn pmid() -> Self {
        Self::new(ColumnType::PmidColumn,  "PMID", "CURIE")
    }

    pub fn title() -> Self {
        Self::new(ColumnType::TitleColumn,"title", "str")
    }

    pub fn individual_id() -> Self {
        Self::new(ColumnType::IndividualIdColumn, "individual_id", "str")
    }

    pub fn individual_comment() -> Self {
        Self::new(ColumnType::IndividualCommentColumn, "comment", "optional")
    }

    pub fn disease_id() -> Self {
        Self::new(ColumnType::DiseaseIdColumn, "disease_id", "CURIE")
    }

    pub fn disease_label() -> Self {
        Self::new(ColumnType::DiseaseLabelColumn, "disease_label", "str")
    }

    pub fn hgnc() -> Self {
        Self::new(ColumnType::HgncIdColumn, "HGNC_id", "CURIE")
    }

    pub fn gene_symbol() -> Self {
        Self::new(ColumnType::GeneSymbolColumn, "gene_symbol", "str")
    }

    pub fn transcript() -> Self {
        Self::new(ColumnType::GeneSymbolColumn, "transcript", "str")
    }

    pub fn allele_1() -> Self {
        Self::new(ColumnType::AlleleOneColumn, "allele_1", "str")
    }

    pub fn allele_2() -> Self {
        Self::new(ColumnType::AlleleTwoColumn, "allele_2", "str")
    }

    pub fn variant_comment() -> Self {
        Self::new(ColumnType::VariantCommentColumn, "variant.comment", "optional")
    }

    pub fn age_of_onset() -> Self {
        Self::new(ColumnType::AgeOfOnsetColumn, "age_of_onset", "age")
    }

    pub fn age_at_last_encounter() -> Self {
        Self::new(ColumnType::AgeAtLastExaminationColumn, "age_at_last_encounter", "age")
    }

    pub fn deceased() -> Self {
        Self::new(ColumnType::AgeAtLastExaminationColumn, "deceased", "yes/no/na")
    }

    pub fn sex() -> Self {
        Self::new(ColumnType::SexColumn, "sex", "M:F:O:U")
    }

    pub fn separator() -> Self {
        Self::new(ColumnType::SeparatorColumn, "HPO", "na")
    }

    pub fn hpo_term(name: &str, term_id: &TermId) -> Self {
        Self::new(ColumnType::HpoTermColumn, name, &term_id.to_string())
    } 

    pub fn get(&self, idx: usize) -> Result<String, String> {
        if idx >= self.column_data.len() {
            return Err(format!("Attempted to access column at index {} but column has only {} entries", idx, self.column_data.iter().len()));
        }
        match self.column_data.get(idx).cloned() {
            Some(data) =>  Ok(data.clone()),
            None => Err(format!("Could not get column data"))
        }
    }

}
