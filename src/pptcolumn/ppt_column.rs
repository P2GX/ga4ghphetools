use std::{
    collections::HashSet,
    fmt::{self, format}, sync::Arc,
};


use crate::{
    error::{self, Error, Result}, header::{age_last_encounter::AgeLastEncounterDuplet, age_of_onset_duplet::AgeOfOnsetDuplet, allele_1_duplet::Allele1Duplet, allele_2_duplet::Allele2Duplet, comment_duplet::CommentDuplet, deceased_duplet::DeceasedDuplet, disease_id_duplet::DiseaseIdDuplet, disease_label_duplet::DiseaseLabelDuplet, gene_symbol_duplet::GeneSymbolDuplet, header_duplet::{HeaderDupletItem, HeaderDupletItemFactory}, hgnc_duplet::HgncDuplet, hpo_separator_duplet::HpoSeparatorDuplet, hpo_term_duplet::HpoTermDuplet, individual_id_duplet::IndividualIdDuplet, pmid_duplet::PmidDuplet, sex_duplet::SexDuplet, title_duplet::TitleDuplet, transcript_duplet::TranscriptDuplet, variant_comment_duplet::VariantCommentDuplet}
};

use crate::header::header_duplet::HeaderDuplet;

use once_cell::sync::Lazy;
use ontolius::TermId;
use phenopackets_dev::schema::v1::core::individual;
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



trait PptCellValidator {
    fn validate(&self, value: &str) -> Result<()>;
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
        let hd = self.get_header_duplet();
        hd.qc_cell(value)
    }
}

/// A structure that contains all of the information of a column in our template
pub struct PptColumn {
    header_duplet: HeaderDuplet,
    column_data: Vec<String>,
}

impl PptColumn {
    pub fn new(
        header_dup: HeaderDuplet,
        column: &[String]) -> Self {
        // the first two columns are the header and do make contain column data
        // note that we have checked that the vector is at least three
        let coldata: Vec<String> = match column.len() {
            0 => Vec::new(),
            _ => column.iter().cloned().collect(),
        };
        PptColumn {
            header_duplet: header_dup,
            column_data: coldata,
        }
    }

    pub fn pmid(col: &Vec<String>) -> Self {
        let pmid = PmidDuplet::new();
        Self::new(pmid.into_enum(), col)
    }

    pub fn title(col: &Vec<String>) -> Self {
        let title = TitleDuplet::new();
        Self::new(title.into_enum(), col)
    }

    pub fn individual_id(col: &Vec<String>) -> Self {
        let individual = IndividualIdDuplet::new();
        Self::new(individual.into_enum(), col)
    }

    pub fn individual_comment(col: &Vec<String>) -> Self {
        let comment = CommentDuplet::new();
        Self::new(comment.into_enum(), col)
    }

    pub fn disease_id(col: &Vec<String>) -> Self {
        let disease_id = DiseaseIdDuplet::new();
        Self::new(disease_id.into_enum(), col)
    }

    pub fn disease_label(col: &Vec<String>) -> Self {
        let disease_label = DiseaseLabelDuplet::new();
        Self::new(disease_label.into_enum(), col)
    }

    pub fn hgnc(col: &Vec<String>) -> Self {
        let hgnc = HgncDuplet::new();
        Self::new(hgnc.into_enum(), col)
    }

    pub fn gene_symbol(col: &Vec<String>) -> Self {
        let gene = GeneSymbolDuplet::new();
        Self::new(gene.into_enum(), col)
    }

    pub fn transcript(col: &Vec<String>) -> Self {
        let transcript = TranscriptDuplet::new();
        Self::new(transcript.into_enum(), col)
    }

    pub fn allele_1(col: &Vec<String>) -> Self {
        let a1 = Allele1Duplet::new();
        Self::new(a1.into_enum(), col)
    }

    pub fn allele_2(col: &Vec<String>) -> Self {
        let a2 = Allele2Duplet::new();
        Self::new(a2.into_enum(), col)
    }

    pub fn variant_comment(col: &Vec<String>) -> Self {
        let vcomment = VariantCommentDuplet::new();
        Self::new(vcomment.into_enum(), col)
    }

    pub fn age_of_onset(col: &Vec<String>) -> Self {
        let age_of_onset = AgeOfOnsetDuplet::new();
        Self::new(age_of_onset.into_enum(), col)
    }

    pub fn age_at_last_encounter(col: &Vec<String>) -> Self {
        let age_of_encounter = AgeLastEncounterDuplet::new();
        Self::new(age_of_encounter.into_enum(), col)
    }

    pub fn deceased(col: &Vec<String>) -> Self {
        let deceased = DeceasedDuplet::new();
        Self::new(deceased.into_enum(), col)
    }

    pub fn sex(col: &Vec<String>) -> Self {
        let sx = SexDuplet::new();
        Self::new(sx.into_enum(), col)
    }

    pub fn separator(col: &Vec<String>) -> Self {
        let sep = HpoSeparatorDuplet::new();
        Self::new(sep.into_enum(), col)
    }

    pub fn hpo_term(name: &str, term_id: &TermId) -> Self {
        let empty_col: Vec<String> = vec![];
        let hpo_dup = HpoTermDuplet::new(name, term_id.to_string());
        Self::new(hpo_dup.into_enum(), &empty_col)
    }

    pub fn is_hpo_column(&self) -> bool {
        matches!(self.header_duplet, HeaderDuplet::HpoTermDuplet(_)) 
    }

    /// Method to be called from PptTemplate::from_string_matrix
    /// generates an HPO column from data taken from an Excel template file
    pub fn hpo_term_from_column(header_dup: &HeaderDuplet, col: &Vec<String>) -> Self {
        let name = header_dup.row1();
        let hpid = header_dup.row2();
        Self::new(header_dup.clone(), col)
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

    
    pub fn get_options_for_header(&self, row: usize, col: usize) -> Vec<String> {
        if self.is_hpo_column() {
            return vec!["edit".to_string()];
        } else {
            return vec!["not editable".to_string()];
        }
    }

    pub fn get_options(&self, row: usize, col: usize, addtl: Vec<String>) -> Vec<String> {
        if row < 2 {
            // special treatment for the first two rows, which make up the header
            return self.get_options_for_header(row, col);
        }
        let hdup = self.header_duplet.as_trait();
        let mut items = hdup.get_options();
        items.extend(addtl);
        items
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

     /// Sets the value of a one of the two header fields
    pub fn set_header_value(&mut self, idx: usize, val: &str) -> Result<()> {
        if idx > 1 {
            return Err(Error::HeaderError { msg: format!("Only index 0 or 1 valid to set HPO header") })
        }
        if self.is_hpo_column() {
            let mut hpo_term_column = self.header_duplet.as_trait_mut()?;
            hpo_term_column.set_value(idx, val)?;
            return Ok(());
        }
        return Err(Error::HeaderError { msg: format!("Only index 0 or 1 valid to set HPO header") })
    }

    /// Sets the value of a phenopacket, whereby the idx is the idx with respect to the phenoapckets
    /// that is, the first two rows (header duplet) are ignored, idx=0 is the first phenopacket row)
    pub fn set_phenopacket_value(&mut self, idx: usize, val: &str) -> Result<()> {
        if idx >= self.phenopacket_count() {
            return Err(Error::row_index_error(idx, self.phenopacket_count()));
        }
        self.header_duplet.qc_cell(val)?;
        self.column_data[idx] = val.to_string();
        Ok(())
    }

    /// Remove leading and trailing whitespace, if any, from the value in row idx
    pub fn trim_value(&mut self, idx: usize) -> Result<()> {
        if idx >= self.phenopacket_count() {
            return Err(Error::row_index_error(idx, self.phenopacket_count()));
        }
        self.column_data[idx] = self.column_data[idx].trim().to_string();
        Ok(())
    }

    /// Remove whitespace at any position from the value in row idx
    pub fn remove_whitespace(&mut self, idx: usize) -> Result<()> {
        if idx >= self.phenopacket_count() {
            return Err(Error::row_index_error(idx, self.phenopacket_count()));
        }
        self.column_data[idx] = self.column_data[idx]
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect();
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
        writeln!(fmt, "Column Type: {:?}", self.header_duplet.row1())?;
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
