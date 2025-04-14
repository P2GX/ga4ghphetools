use std::{
    collections::HashSet,
    fmt::{self, format},
};

use super::header_duplet::HeaderDuplet;
use crate::{
    error::{self, Error, Result},
    template::individual_template::TemplateError,
};
use once_cell::sync::Lazy;
use ontolius::TermId;
use polars::error::ErrString;
use regex::Regex;

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
static EXPECTED_H1_FIELDS: [&str; NUMBER_OF_CONSTANT_HEADER_FIELDS] = [
    "PMID",
    "title",
    "individual_id",
    "comment",
    "disease_id",
    "disease_label",
    "HGNC_id",
    "gene_symbol",
    "transcript",
    "allele_1",
    "allele_2",
    "variant.comment",
    "age_of_onset",
    "age_at_last_encounter",
    "deceased",
    "sex",
    "HPO",
];
const EXPECTED_H2_FIELDS: [&str; NUMBER_OF_CONSTANT_HEADER_FIELDS] = [
    "CURIE",
    "str",
    "str",
    "optional",
    "CURIE",
    "str",
    "CURIE",
    "str",
    "str",
    "str",
    "str",
    "optional",
    "age",
    "age",
    "yes/no/na",
    "M:F:O:U",
    "na",
];

trait PptCellValidator {
    fn validate(&self, value: &str) -> Result<()>;
}

#[derive(Clone, Debug, PartialEq)]
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
    NtrRequestColumn,
}

impl Error {
    pub fn pmid_error(val: &str) -> Self {
        Error::PmidError {
            msg: format!("Malformed PMID entry: '{}'", val),
        }
    }

    pub fn title_error(val: &str) -> Self {
        Error::WhiteSpaceError {
            msg: format!("Malformed title: '{}'", val),
        }
    }

    pub fn disease_id_error(val: &str) -> Self {
        Error::DiseaseIdError {
            msg: format!("Malformed disease id '{}'", val),
        }
    }

    pub fn ws_error(val: &str, field: &str) -> Self {
        Error::DiseaseIdError {
            msg: format!("{field} field has whitespace error '{}'", val),
        }
    }
}

impl PptCellValidator for PptColumn {
    fn validate(&self, value: &str) -> Result<()> {
        match &self.column_type {
            ColumnType::PmidColumn => {
                if !PMID_REGEX.is_match(value) {
                    return Err(Error::pmid_error(value));
                }
            }
            ColumnType::TitleColumn => {
                if !NO_LEADING_TRAILING_WS.is_match(value) {
                    return Err(Error::ws_error(value, "title"));
                }
            }
            ColumnType::IndividualIdColumn => {
                if !NO_LEADING_TRAILING_WS.is_match(value) {
                    return Err(Error::ws_error(value, "individual_id"));
                }
            }
            ColumnType::IndividualCommentColumn => {
                return Ok(());
            }
            ColumnType::DiseaseIdColumn => {
                if !DISEASE_ID_REGEX.is_match(value) {
                    return Err(Error::disease_id_error(value));
                }
            }
            ColumnType::DiseaseLabelColumn => {
                if !NO_LEADING_TRAILING_WS.is_match(value) {
                    return Err(Error::ws_error(value, "disease_label"));
                }
            }
            ColumnType::HgncIdColumn => {
                if !HGNC_ID_REGEX.is_match(value) {
                    return Err(Error::HgncError {
                        msg: format!("Malformed HGNC id: '{value}'"),
                    });
                }
            }
            ColumnType::GeneSymbolColumn => {
                if !NO_LEADING_TRAILING_WS.is_match(value) {
                    return Err(Error::ws_error(value, "gene_symbol"));
                }
            }
            ColumnType::TranscriptColumn => {
                if !TRANSCRIPT_REGEX.is_match(value) {
                    return Err(Error::TranscriptError {
                        msg: format!("Malformed transcript: '{value}'"),
                    });
                }
            }
            ColumnType::AlleleOneColumn => {
                if !NO_LEADING_TRAILING_WS.is_match(value) {
                    return Err(Error::ws_error(value, "allele_1"));
                }
            }
            ColumnType::AlleleTwoColumn => {
                if !NO_LEADING_TRAILING_WS.is_match(value) {
                    return Err(Error::ws_error(value, "allele_2"));
                }
            }
            ColumnType::AgeOfOnsetColumn => {
                return Ok(());
            }
            ColumnType::AgeAtLastExaminationColumn => {
                return Ok(()); // TODO BETTER FUNCTION
            }
            ColumnType::DeceasedColumn => {
                if !DECEASED_REGEX.is_match(value) {
                    return Err(Error::DeceasedError {
                        msg: format!("Malformed deceased entry: '{value}'"),
                    });
                }
            }
            ColumnType::SexColumn => {
                if !SEX_REGEX.is_match(value) {
                    return Err(Error::sex_field_error(value));
                }
            }
            ColumnType::SeparatorColumn => {
                if !SEPARATOR_REGEX.is_match(value) {
                    return Err(Error::separator(value));
                }
            }
            _ => {
                return Ok(());
            }
        }
        Ok(())
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
        // the first two columns are the header and do make contain column data
        // note that we have checked that the vector is at least three
        let coldata: Vec<String> = match column.len() {
            0 => Vec::new(),
            _ => column.iter().cloned().collect(),
        };
        PptColumn {
            column_type: column_type,
            header_duplet: hd,
            column_data: coldata,
        }
    }

    pub fn pmid(col: &Vec<String>) -> Self {
        Self::new(ColumnType::PmidColumn, "PMID", "CURIE", col)
    }

    pub fn title(col: &Vec<String>) -> Self {
        Self::new(ColumnType::TitleColumn, "title", "str", col)
    }

    pub fn individual_id(col: &Vec<String>) -> Self {
        Self::new(ColumnType::IndividualIdColumn, "individual_id", "str", col)
    }

    pub fn individual_comment(col: &Vec<String>) -> Self {
        Self::new(
            ColumnType::IndividualCommentColumn,
            "comment",
            "optional",
            col,
        )
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
        Self::new(ColumnType::TranscriptColumn, "transcript", "str", col)
    }

    pub fn allele_1(col: &Vec<String>) -> Self {
        Self::new(ColumnType::AlleleOneColumn, "allele_1", "str", col)
    }

    pub fn allele_2(col: &Vec<String>) -> Self {
        Self::new(ColumnType::AlleleTwoColumn, "allele_2", "str", col)
    }

    pub fn variant_comment(col: &Vec<String>) -> Self {
        Self::new(
            ColumnType::VariantCommentColumn,
            "variant.comment",
            "optional",
            col,
        )
    }

    pub fn age_of_onset(col: &Vec<String>) -> Self {
        Self::new(ColumnType::AgeOfOnsetColumn, "age_of_onset", "age", col)
    }

    pub fn age_at_last_encounter(col: &Vec<String>) -> Self {
        Self::new(
            ColumnType::AgeAtLastExaminationColumn,
            "age_at_last_encounter",
            "age",
            col,
        )
    }

    pub fn deceased(col: &Vec<String>) -> Self {
        Self::new(ColumnType::DeceasedColumn, "deceased", "yes/no/na", col)
    }

    pub fn sex(col: &Vec<String>) -> Self {
        Self::new(ColumnType::SexColumn, "sex", "M:F:O:U", col)
    }

    pub fn separator(col: &Vec<String>) -> Self {
        Self::new(ColumnType::SeparatorColumn, "HPO", "na", col)
    }

    pub fn hpo_term(name: &str, term_id: &TermId) -> Self {
        let empty_col: Vec<String> = vec![];
        Self::new(
            ColumnType::HpoTermColumn,
            name,
            &term_id.to_string(),
            &empty_col,
        )
    }

    /// Method to be called from PptTemplate::from_string_matrix
    /// generates an HPO column from data taken from an Excel template file
    pub fn hpo_term_from_column(header_dup: &HeaderDuplet, col: &Vec<String>) -> Self {
        let name = header_dup.row1();
        let hpid = header_dup.row2();
        Self::new(ColumnType::HpoTermColumn, &name, &hpid, col)
    }

    pub fn get_header_duplet(&self) -> HeaderDuplet {
        self.header_duplet.clone()
    }

    pub fn get(&self, idx: usize) -> Result<String> {
        match idx {
            0 => {
                return Ok(self.header_duplet.row1());
            }
            1 => {
                return Ok(self.header_duplet.row2());
            }
            i if i <= self.column_data.len() + 2 => {
                let col_idx = i - 2;
                match self.column_data.get(col_idx) {
                    Some(data) => {
                        return Ok(data.clone());
                    }
                    None => {
                        return Err(Error::TemplateError {
                            msg: format!("Could not get column data at column index {}", col_idx),
                        });
                    }
                }
            }
            _ => {
                let msg = format!(
                    "Attempted to access column at index {} but column has only {} entries",
                    idx,
                    self.column_data.iter().len()
                );
                return Err(Error::TemplateError { msg });
            }
        }
    }

    pub fn column_type(&self) -> ColumnType {
        self.column_type.clone()
    }

    pub fn get_options_for_header(&self, row: usize, col: usize) -> Vec<String> {
        if self.column_type == ColumnType::HpoTermColumn {
            return vec!["edit".to_string()];
        } else {
            return vec![];
        }
    }

    pub fn get_options(&self, row: usize, col: usize, addtl: Vec<String>) -> Vec<String> {
        if row < 2 {
            // special treatment for the first two rows, which make up the header
            return self.get_options_for_header(row, col);
        }
        match self.column_type {
            ColumnType::HpoTermColumn => {
                let mut items = vec![
                    "observed".to_string(),
                    "excluded".to_string(),
                    "na".to_string(),
                ];
                items.extend(addtl);
                return items;
            }
            ColumnType::SexColumn => {
                return vec![
                    "M".to_string(),
                    "F".to_string(),
                    "O".to_string(),
                    "U".to_string(),
                ];
            }
            ColumnType::DeceasedColumn => {
                return vec!["yes".to_string(), "no".to_string(), "na".to_string()];
            }
            _ => {
                return vec!["edit".to_string()];
            }
            _ => {}
        }

        vec![]
    }

    /// Validate entry according to column specific rules.
    pub fn add_entry<T: Into<String>>(&mut self, value: T) -> Result<()> {
        let val = value.into();
        self.validate(&val)?;
        self.column_data.push(val);
        Ok(())
    }

    pub fn phenopacket_count(&self) -> usize {
        self.column_data.len()
    }

    pub fn nrows(&self) -> usize {
        2 + self.column_data.len()
    }

    pub fn delete_entry_at_row(&mut self, row: usize) {
        self.column_data.remove(row);
    }

    /// Some columns, such as HGNC or disease id, must always have the same content in any given template
    ///
    /// This function checks whether all data cells have the same value. If not, it returns an error
    pub fn get_unique(&self) -> Result<String> {
        let unique_values: HashSet<&String> = self.column_data.iter().collect();
        match unique_values.len() {
            1 => Ok(unique_values.iter().next().unwrap().to_string()),
            _ => {
                let joined = unique_values
                    .iter()
                    .cloned()
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ");
                Err(Error::TemplateError {
                    msg: format!("More than one entry: {joined}"),
                })
            }
        }
    }

    pub fn validate_data(&self) -> Vec<Error> {
        let mut error_list = Vec::new();
        for val in &self.column_data {
            if let Err(e) = self.validate(&val) {
                error_list.push(e);
            }
        }
        error_list
    }

    /// Add a new line to the column
    ///
    /// This function is used to create a new row by calling this function on
    /// all columns in a template.
    pub fn add_blank_field(&mut self) {
        self.column_data.push(String::default());
    }

    pub fn set_value(&mut self, idx: usize, val: impl Into<String>) -> Result<()> {
        if idx >= self.phenopacket_count() {
            return Err(Error::row_index_error(idx, self.phenopacket_count()));
        }
        self.column_data[idx] = val.into();
        Ok(())
    }

    pub fn get_string_column(&self) -> Vec<String> {
        let mut col: Vec<String> = Vec::new();
        let hd = self.get_header_duplet();
        col.push(hd.row1());
        col.push(hd.row2());
        for item in &self.column_data {
            col.push(item.clone());
        }
        col
    }

    pub fn get_string(&self, idx: usize) -> Result<String> {
        if idx == 0 {
            return Ok(self.header_duplet.row1());
        } else if idx == 1 {
            return Ok(self.header_duplet.row2());
        } else {
            let i = idx - 2;
            match self.column_data.get(i) {
                Some(item) => Ok(item.clone()),
                None => Err(Error::row_index_error(idx, self.column_data.len())),
            }
        }
    }
}

impl core::fmt::Display for PptColumn {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> fmt::Result {
        writeln!(fmt, "Column Type: {:?}", self.column_type)?;
        writeln!(fmt, "Header: {}", self.header_duplet)?;
        writeln!(fmt, "Data:")?;
        for (i, value) in self.column_data.iter().enumerate() {
            writeln!(fmt, "  [{}]: {}", i, value)?;
        }
        Ok(())
    }
}

// region:    --- Tests

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use once_cell::sync::Lazy;
    use ontolius::common::hpo;
    type Error = Box<dyn std::error::Error>;
    type Result<T> = core::result::Result<T, Error>; // For tests.

    static PMID_COLUMN: Lazy<PptColumn> = Lazy::new(|| {
        // Create and return the shared test object
        let mut column = PptColumn::pmid(&vec![]);
        column.add_entry("PMID:123");
        column.add_entry("PMID:234");
        column.add_entry("PMID:345");
        column
    });

    static HPO_COLUMN: Lazy<PptColumn> = Lazy::new(|| {
        let entries: Vec<&str> = vec![
            "excluded", "observed", "excluded", "observed", "excluded", "observed",
        ];
        let tid = TermId::from_str("HP:0001166").unwrap();
        let mut hpo_col = PptColumn::hpo_term("Arachnodactyly", &tid);
        for e in entries {
            hpo_col.add_entry(e);
        }
        hpo_col
    });

    static DISEASE_ID_COLUMN: Lazy<PptColumn> = Lazy::new(|| {
        let entries: Vec<&str> = vec!["OMIM:121050", "OMIM:121050", "OMIM:121050", "OMIM:121050"];
        let tid = TermId::from_str("HP:0001166").unwrap();
        let mut disease_col = PptColumn::disease_id(&vec![]);
        for e in entries {
            disease_col.add_entry(e);
        }
        disease_col
    });

    use super::*;

    #[test]
    fn test_pmid_colum() -> Result<()> {
        let pmid_col = &*PMID_COLUMN;
        assert_eq!("PMID", pmid_col.get(0).unwrap());
        assert_eq!("CURIE", pmid_col.get(1).unwrap());
        assert_eq!("PMID:123", pmid_col.get(2).unwrap());
        assert_eq!("PMID:234", pmid_col.get(3).unwrap());
        assert_eq!("PMID:345", pmid_col.get(4).unwrap());
        assert_eq!(3, pmid_col.phenopacket_count());
        assert_eq!(5, pmid_col.nrows());
        Ok(())
    }

    #[test]
    fn test_hpo_colum() -> Result<()> {
        let hpo_col = &*HPO_COLUMN;
        assert_eq!("Arachnodactyly", hpo_col.get(0).unwrap());
        assert_eq!("HP:0001166", hpo_col.get(1).unwrap());
        assert_eq!("excluded", hpo_col.get(2).unwrap());
        assert_eq!("observed", hpo_col.get(3).unwrap());
        assert_eq!("excluded", hpo_col.get(4).unwrap());
        assert_eq!("observed", hpo_col.get(5).unwrap());
        assert_eq!("excluded", hpo_col.get(6).unwrap());
        assert_eq!("observed", hpo_col.get(7).unwrap());
        assert_eq!(6, hpo_col.phenopacket_count());
        assert_eq!(8, hpo_col.nrows());
        Ok(())
    }

    #[test]
    fn test_unique() -> Result<()> {
        let disease_id_col = &*DISEASE_ID_COLUMN;
        assert_eq!("disease_id", disease_id_col.get(0).unwrap());
        assert_eq!("CURIE", disease_id_col.get(1).unwrap());
        assert_eq!("OMIM:121050", disease_id_col.get(2).unwrap());
        let result = disease_id_col.get_unique();
        assert!(result.is_ok());
        let uniq = result.unwrap();
        assert_eq!("OMIM:121050", uniq);

        Ok(())
    }
}

// endregion: --- Tests
