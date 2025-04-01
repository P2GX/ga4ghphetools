



use std::{collections::HashSet, fmt::format};

use ontolius::TermId;
use regex::Regex;
use once_cell::sync::Lazy;
use super::header_duplet::HeaderDuplet;
use crate::{error::{self, Error, Result}, individual_template::TemplateError};

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

    pub fn new(column_type: ColumnType, header1: &str, header2: &str, column: &[String]) -> Self {
        let hd = HeaderDuplet::new(header1, header2);
        PptColumn { column_type:column_type, header_duplet: hd, column_data: column.to_vec() }
    }

    pub fn pmid(col: &Vec<String>) -> Self {
        Self::new(ColumnType::PmidColumn,  "PMID", "CURIE", col)
    }

    pub fn title(col: &Vec<String>) -> Self {
        Self::new(ColumnType::TitleColumn,"title", "str", col)
    }

    pub fn individual_id(col: &Vec<String>) -> Self {
        Self::new(ColumnType::IndividualIdColumn, "individual_id", "str", col)
    }

    pub fn individual_comment(col: &Vec<String>) -> Self {
        Self::new(ColumnType::IndividualCommentColumn, "comment", "optional", col)
    }

    pub fn disease_id(col: &Vec<String>) -> Self {
        Self::new(ColumnType::DiseaseIdColumn, "disease_id", "CURIE", col)
    }

    pub fn disease_label(col: &Vec<String>) -> Self {
        Self::new(ColumnType::DiseaseLabelColumn, "disease_label", "str", col)
    }

    pub fn hgnc(col: &Vec<String>) -> Self {
        Self::new(ColumnType::HgncIdColumn, "HGNC_id", "CURIE", col)
    }

    pub fn gene_symbol(col: &Vec<String>) -> Self {
        Self::new(ColumnType::GeneSymbolColumn, "gene_symbol", "str", col)
    }

    pub fn transcript(col: &Vec<String>) -> Self {
        Self::new(ColumnType::GeneSymbolColumn, "transcript", "str", col)
    }

    pub fn allele_1(col: &Vec<String>) -> Self {
        Self::new(ColumnType::AlleleOneColumn, "allele_1", "str", col)
    }

    pub fn allele_2(col: &Vec<String>) -> Self {
        Self::new(ColumnType::AlleleTwoColumn, "allele_2", "str", col)
    }

    pub fn variant_comment(col: &Vec<String>) -> Self {
        Self::new(ColumnType::VariantCommentColumn, "variant.comment", "optional", col)
    }

    pub fn age_of_onset(col: &Vec<String>) -> Self {
        Self::new(ColumnType::AgeOfOnsetColumn, "age_of_onset", "age", col)
    }

    pub fn age_at_last_encounter(col: &Vec<String>) -> Self {
        Self::new(ColumnType::AgeAtLastExaminationColumn, "age_at_last_encounter", "age", col)
    }

    pub fn deceased(col: &Vec<String>) -> Self {
        Self::new(ColumnType::AgeAtLastExaminationColumn, "deceased", "yes/no/na", col)
    }

    pub fn sex(col: &Vec<String>) -> Self {
        Self::new(ColumnType::SexColumn, "sex", "M:F:O:U", col)
    }

    pub fn separator(col: &Vec<String>) -> Self {
        Self::new(ColumnType::SeparatorColumn, "HPO", "na", col)
    }

    pub fn hpo_term(name: &str, term_id: &TermId) -> Self {
        let empty_col: Vec<String> = vec![];
        Self::new(ColumnType::HpoTermColumn, name, &term_id.to_string(), &empty_col)
    } 

    pub fn hpo_term_from_column(col: &Vec<String>) -> Self {
        let name = &col[0];
        let hpid = &col[1];
        let rest: &[String] = &col[2..];
        Self::new(ColumnType::HpoTermColumn, &name, &hpid, rest)

    }

    pub fn get(&self, idx: usize) -> Result<String> {
        if idx >= self.column_data.len() {
            let msg = format!("Attempted to access column at index {} but column has only {} entries", idx, self.column_data.iter().len());
            return Err(Error::TemplateError { msg });
        }
        match self.column_data.get(idx).cloned() {
            Some(data) =>  Ok(data.clone()),
            None => Err(Error::TemplateError {msg: format!("Could not get column data")})
        }
    }

    pub fn phenopacket_count(&self) -> usize {
        self.column_data.len()
    }

    /// Some columns, such as HGNC or disease id, must always have the same content in any given template
    /// 
    /// This function checks whether all data cells have the same value. If not, it returns an error
    pub fn get_unique(&self) -> Result<String> {
        let unique_values: HashSet<&String> = self.column_data.iter().collect();
        match unique_values.len() {
            1 => Ok(unique_values.iter().next().unwrap().clone().to_string()),
            _ => {
                let joined = unique_values.iter().cloned().cloned().collect::<Vec<_>>().join(", ");
                Err(Error::TemplateError {
                    msg: format!("More than one entry: {joined}"),
                })
            }
        }
    }

}
