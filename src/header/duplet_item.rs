//! DupletItem
//! 
//! These structures represent the duplet headers of our template
//!    "PMID", "title", "individual_id", "comment", "disease_id", "disease_label", "HGNC_id",	"gene_symbol", 
//!     "transcript", "allele_1", "allele_2", "variant.comment", "age_of_onset", "age_at_last_encounter", 
//!      "deceased", "sex", "HPO",	"Clinodactyly of the 5th finger", (etc., arbitrarily many HPO columns)


use std::{collections::HashSet, fmt::format};
use once_cell::sync::Lazy;
use regex::Regex;

use crate::{dto::template_dto::HeaderDupletDto, header::age_util};




static FORBIDDEN_CHARS: Lazy<HashSet<char>> = Lazy::new(|| {
    ['/', '\\', '(', ')'].iter().copied().collect()
});


pub static HGVS_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"c.[\d_]+(.*)").unwrap()
});

pub static SUBSTITUTION_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"([ACGT]+)([>]{1}[ACGT]+)$").unwrap()
});

pub static INSERTION_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"ins[ACGT]+$").unwrap()
});

pub static DELINS_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^c\.(\d+_\d+)delins[A-Za-z0-9]+$").unwrap()
});

pub static ALLOWED_STRUCTURAL_PREFIX: Lazy<HashSet<String>> = Lazy::new(|| {
    ["DEL", "DUP", "INV", "INS", "TRANSL"]
        .iter()
        .map(|s| s.to_string())
        .collect()
});

pub static ALLOWED_AGE_LABELS: Lazy<HashSet<String>> = Lazy::new(|| {
    [
        "Late onset",
        "Middle age onset",
        "Young adult onset",
        "Late young adult onset",
        "Intermediate young adult onset",
        "Early young adult onset",
        "Adult onset",
        "Juvenile onset",
        "Childhood onset",
        "Infantile onset",
        "Neonatal onset",
        "Congenital onset",
        "Antenatal onset",
        "Embryonal onset",
        "Fetal onset",
        "Late first trimester onset",
        "Second trimester onset",
        "Third trimester onset",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect()
});

/// Regex for ISO 8601 durations
pub static ISO8601_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^P(?:(\d+)Y)?(?:(\d+)M)?(?:(\d+)D)?$").expect("valid ISO 8601 regex")
});

/// Regex for gestational age format
pub static GESTATIONAL_AGE_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"G\d+w[0-6]d").expect("valid gestational age regex")
});

pub static ALLOWED_DECEASED_ITEMS: Lazy<HashSet<String>> = Lazy::new(||{
    let mut hset = HashSet::new();
    hset.insert("yes".to_string());
    hset.insert("no".to_string());
    hset.insert("na".to_string());
    hset
});

pub static ALLOWED_SEX_ITEMS: Lazy<HashSet<String>> = Lazy::new(||{
    let mut hset = HashSet::new();
    hset.insert("M".to_string());
    hset.insert("F".to_string());
    hset.insert("O".to_string());
    hset.insert("U".to_string());
    hset
});

#[derive(Clone, Debug)]
pub enum DupletType {
    PMID,
    TITLE,
    INDIVIDUALID,
    COMMENT,
    DISEASEID,
    DISEASELABEL,
    HGNCID,
    GENESYMBOL,
    TRANSCRIPT,
    ALLELE1,
    ALLELE2,
    VARIANTCOMMENT,
    AGEOFONSET,
    AGEATLASTENCOUNTER,
    DECEASED,
    SEX
}



#[derive(Clone, Debug)]
pub struct DupletItem {
    pub row1: String, 
    pub row2: String,
    duplet_type: DupletType,
}

impl DupletItem {
    pub fn new(h1: &str, h2: &str, dtype: DupletType) -> Self {
        Self { row1: h1.to_string(), row2: h2.to_string(), duplet_type: dtype }
    }

    pub fn check_column_labels(
        &self,
        matrix: &Vec<Vec<String>>,
        column: usize,
    ) -> Result<(), String> {
        let actual_row1 = matrix
            .get(0)
            .and_then(|row| row.get(column))
            .ok_or_else(|| format!("Missing row 0 or column {}", column))?;

        if actual_row1 != &self.row1 {
            return Err(format!(
                "Row 0, column {} expected '{}', found '{}'",
                column, self.row1, actual_row1
            ));
        }

        let actual_row2 = matrix
            .get(1)
            .and_then(|row| row.get(column))
            .ok_or_else(|| format!("Missing row 1 or column {}", column))?;

        if actual_row2 != &self.row2 {
            return Err(format!(
                "Row 1, column {} expected '{}', found '{}'",
                column, self.row2, actual_row2
            ));
        }

        Ok(())
    }


    /// A valid curie must have a non-empty prefix and a non-empty numeric suffic
    /// white-space is not allowed.
    fn check_valid_curie(s: &str) -> Result<(), String> {
        if s.is_empty() {
            return Err(format!("Empty CURIE"));
        } else if let Some(pos) = s.find(':') {
            if s.chars().any(|c| c.is_whitespace()) {
                return Err(format!("Contains stray whitespace: '{}'", s));
            } else if s.matches(':').count() != 1 {
                return Err(format!("Invalid CURIE with more than one colon: '{}", s));
            } else if pos == 0 {
                return Err(format!("Invalid CURIE with no prefix: '{}'", s));
            } else if pos == s.len() - 1 {
                return Err(format!("Invalid CURIE with no suffix: '{}'", s));
            } else if let Some((_prefix, suffix)) = s.split_once(':') {
                if !suffix.chars().all(char::is_numeric) {
                    return Err(format!("Invalid CURIE with non-digit characters in suffix: '{}'", s));
                }
            }
        } else {
            return Err(format!("Invalid CURIE with no colon: '{}'", s));
        }
        Ok(())
    }

    /// A valid label does not begin with or end with a white space
    /// Valid labels also may not contain /,\, (,  ), or perdiod (".").
    fn check_white_space(cell_contents: &str) -> Result<(), String> {
        if cell_contents.chars().last().map_or(false, |c| c.is_whitespace()) {
            return Err( format!("Trailing whitespace in '{}'", cell_contents));
        } else if cell_contents.chars().next().map_or(false, |c| c.is_whitespace()) {
            return Err(format!("Leading whitespace in '{}'", cell_contents));
        } else if cell_contents.contains("  ") {
            return Err(format!("Consecutive whitespace in '{}'", cell_contents));
        } else {
            Ok(())
        }
    }

    /// Some ColumnTypes do not allow empty cells.
    pub fn check_empty(cell_contents: &str) -> Result<(), String> {
        if cell_contents.is_empty() {
            Err(format!("Value must not be empty"))
        } else {
            Ok(())
        }
    }

    /// These characters are not allowed in the individual id field
    fn check_forbidden_chars(value: &str) -> Result<(), String> {
        match value.chars().find(|&c| FORBIDDEN_CHARS.contains(&c)) {
            Some(fc) => Err(format!("Forbidden character '{fc}' found in label '{value}'")),
            None => Ok(()),
        }
    }


    fn check_valid_hgvs(value: &str) -> Result<(), String>  {
        // if we get here, there was a non-empty string that starts with "c."
        if let Some(captures) = HGVS_RE.captures(value) {
            if let Some(matched_substr) = captures.get(1) {
                // we now have either G>T, del, insT (etc), or delinsT (etc)
                let remaining_hgvs = matched_substr.as_str();
                if SUBSTITUTION_RE.is_match(remaining_hgvs) {
                    return Ok(());
                } else if INSERTION_RE.is_match(remaining_hgvs) {
                    return Ok(());
                } else if remaining_hgvs == "del" {
                    return Ok(());
                } else if DELINS_RE.is_match(remaining_hgvs) {
                    return Ok(());
                }
                return Err(format!("Malformed HGVS '{value}'"));
            }
        }
        return Err(format!("Malformed HGVS '{value}'"));
    }

    fn check_tab(cell_contents: &str) -> Result<(), String> {
        if cell_contents.contains('\t') {
            Err(format!("Cell '{cell_contents}' must not contain a tab character"))
        } else {
            Ok(())
        }
    }


    fn check_valid_structural(value: &str) -> Result<(), String>  {
        let parts: Vec<&str> = value.split(':').collect();
        let prefix = parts[0];
        let suffix = parts[1..].join(":"); // in case the original string contains ":"
        let structural_var = suffix.trim();
        match  ALLOWED_STRUCTURAL_PREFIX.contains(prefix) {
            true => Ok(()),
            false => Err(format!("Malformed structural variant '{value}'")),
        }
    }

    fn check_valid_age_string(cell_value: &str) -> Result<(), String> {
        // empty not allowed
        if cell_value.is_empty() {
            return Err(format!("Empty age string not allowed (use na)"));
        }
        // but na is OK
        if cell_value == "na" {
            return Ok(());
        }
        // check for match to HPO Onset terms
        if ALLOWED_AGE_LABELS.contains(cell_value) {
            return Ok(());
        }
        // check for match to ISO (601)
        if ISO8601_RE.is_match(cell_value) {
            return Ok(());
        }

        if GESTATIONAL_AGE_RE.is_match(cell_value) {
            return Ok(());
        }

        Err(format!("Malformed age string '{}'", cell_value))
    }


    fn check_pmid(&self, cell_contents: &str) -> Result<(), String> {
        let _ = Self::check_valid_curie(cell_contents)?;
        if !cell_contents.starts_with("PMID") {
            return Err(format!("Invalid PubMed prefix: '{}'", cell_contents));
        }
        Ok(())
    }

    fn check_title(&self, cell_contents: &str) -> Result<(), String> {
        let _ = Self::check_white_space(cell_contents)?;
        let _ = Self::check_empty(cell_contents)?;
        Ok(())
    }

    fn check_individual_id(&self, cell_contents: &str) -> Result<(), String> {
        let _ = Self::check_forbidden_chars(cell_contents)?;
        let _ = Self::check_empty(cell_contents)?;
        let _ = Self::check_white_space(cell_contents)?;
        Ok(())
    }

    fn check_comment(&self, cell_contents: &str) -> Result<(), String> {
        let _ = Self::check_forbidden_chars(cell_contents)?;
        Ok(())
    }

    fn check_disease_id(&self, cell_contents: &str) -> Result<(), String> {
        let _ = Self::check_valid_curie(cell_contents)?;
        if !(cell_contents.starts_with("OMIM") || cell_contents.starts_with("MONDO")) {
            return Err(format!("Disease id has invalid prefix: '{}'", cell_contents));
        }
        if cell_contents.starts_with("OMIM:") {
            if cell_contents.len() != 11 {
                return Err(format!("OMIM identifiers must have 6 digits: '{}'", cell_contents));
            }
        }
        Ok(())
    }

    fn check_disease_label(&self, cell_contents: &str) -> Result<(), String> {
        let _ = Self::check_empty(cell_contents)?;
        let _ = Self::check_white_space(cell_contents)?;
        Ok(())
    }

    fn check_hgnc_id(&self, cell_contents: &str) -> Result<(), String> {
        let _ = Self::check_valid_curie(cell_contents)?;
        if ! cell_contents.starts_with("HGNC")  {
            return Err(format!("HGNC id has invalid prefix: '{}'", cell_contents));
        };
        Ok(())
    }

    fn check_gene_symbol(&self, cell_contents: &str) -> Result<(), String> {
        let _ = Self::check_empty(cell_contents)?;
        let _ = Self::check_white_space(cell_contents)?;
        if cell_contents.contains(" ") {
            return Err(format!("Gene symbol must not contain whitespace: '{cell_contents}'"));
        }
        Ok(())
    }

    fn check_transcript(&self, cell_contents: &str) -> Result<(), String> {
        let _ = Self::check_empty(cell_contents)?;
        if ! cell_contents.starts_with("ENST") && ! cell_contents.starts_with("NM_") {
            return Err(format!("Unrecognized transcript prefix '{cell_contents}'"));
        }  
        if ! cell_contents.contains(".") {
            return Err(format!("Transcript '{}' is missing a version", cell_contents));
        } 
        if let Some((before_last, last)) = cell_contents.rsplit_once('.') {
            if before_last.is_empty() {
                return Err(format!("Maformed transcript: '{}'", cell_contents));
            }
            if ! last.chars().all(|c| c.is_ascii_digit()) {
                return Err(format!("Maformed transcript version: '{}'", cell_contents));
            }
        }
        Ok(())
    }

    fn check_allele1(&self, cell_contents: &str) -> Result<(), String> {
        let _ = Self::check_empty(cell_contents)?;
        let _ = Self::check_white_space(cell_contents)?;
        if cell_contents.starts_with("c.") {
            Self::check_valid_hgvs(cell_contents)?;
        } else {
            Self::check_valid_structural(cell_contents)?;
        }
        Ok(())
    }

    fn check_allele2(&self, cell_contents: &str) -> Result<(), String> {
        let _ = Self::check_empty(cell_contents)?;
        let _ = Self::check_white_space(cell_contents)?;
        if cell_contents == "na" {
            return Ok(());
        } else if cell_contents.starts_with("c.") {
            Self::check_valid_hgvs(cell_contents)?;
        } else {
            Self::check_valid_structural(cell_contents)?;
        }
        Ok(())
    }
    fn check_variant_comment(&self, cell_contents: &str) -> Result<(), String> {
        let _ = Self::check_tab(cell_contents)?;
        Ok(())
    }

    fn check_deceased(&self, cell_contents: &str) -> Result<(), String> {
        match ALLOWED_DECEASED_ITEMS.contains(cell_contents) {
            true => Ok(()),
            false => Err(format!("Malformed deceased entry: '{}'", cell_contents))
        }
    }

    fn check_sex(&self, cell_contents: &str) -> Result<(), String> {
        match ALLOWED_SEX_ITEMS.contains(cell_contents) {
            true => Ok(()),
            false => Err(format!("Malformed sex entry: '{}'", cell_contents))
        }
    }
    

    pub fn qc_data(&self, cell_contents: &str) -> Result<(), String> {
        match self.duplet_type {
            DupletType::PMID => self.check_pmid(cell_contents)?,
            DupletType::TITLE => self.check_title(cell_contents)?,
            DupletType::INDIVIDUALID => self.check_individual_id(cell_contents)?,
            DupletType::COMMENT => self.check_comment(cell_contents)?,
            DupletType::DISEASEID => self.check_disease_id(cell_contents)?,
            DupletType::DISEASELABEL => self.check_disease_label(cell_contents)?,
            DupletType::HGNCID => self.check_hgnc_id(cell_contents)?,
            DupletType::GENESYMBOL => self.check_gene_symbol(cell_contents)?,
            DupletType::TRANSCRIPT => self.check_transcript(cell_contents)?,
            DupletType::ALLELE1 => self.check_allele1(cell_contents)?,
            DupletType::ALLELE2 => self.check_allele2(cell_contents)?,
            DupletType::VARIANTCOMMENT => self.check_variant_comment(cell_contents)?,
            DupletType::AGEOFONSET => Self::check_valid_age_string(cell_contents)?,
            DupletType::AGEATLASTENCOUNTER => Self::check_valid_age_string(cell_contents)?,
            DupletType::DECEASED => self.check_deceased(cell_contents)?,
            DupletType::SEX => self.check_sex(cell_contents)?,
        };
        Ok(())
    }

    pub fn row1(&self) -> &str {
        &self.row1
    }

    pub fn row2(&self) -> &str {
        &self.row2
    }


    /// PubMed identifier
    pub fn pmid() -> Self {
        DupletItem::new("PMID", "CURIE", DupletType::PMID)
    }

    pub fn title() -> Self {
        DupletItem::new("title", "str", DupletType::TITLE)
    }

    pub fn individual_id() -> Self {
        DupletItem::new("individual_id", "str", DupletType::INDIVIDUALID)
    }

    pub fn comment() -> Self {
        DupletItem::new("comment", "optional", DupletType::COMMENT)
    }

    pub fn disease_id() -> Self {
        DupletItem::new("disease_id", "CURIE", DupletType::DISEASEID)
    }

    pub fn disease_label() -> Self {
        DupletItem::new("disease_label", "str", DupletType::DISEASELABEL)
    }

    pub fn hgnc_id() -> Self {
        DupletItem::new("HGNC_id", "CURIE", DupletType::HGNCID)
    }

    pub fn gene_symbol() -> Self {
        DupletItem::new("gene_symbol", "str", DupletType::GENESYMBOL)
    }

    pub fn transcript() -> Self {
        DupletItem::new("transcript", "str", DupletType::TRANSCRIPT)
    }

    pub fn allele1() -> Self {
        DupletItem::new("allele_1", "str", DupletType::ALLELE1)
    }

    pub fn allele2() -> Self {
        DupletItem::new("allele_2", "str", DupletType::ALLELE2)
    }
    
    pub fn variant_comment() -> Self {
        DupletItem::new("variant.comment", "optional", DupletType::VARIANTCOMMENT)
    }

    pub fn age_of_onset() -> Self {
        DupletItem::new("age_of_onset", "age", DupletType::AGEOFONSET)
    }

    pub fn age_at_last_encounter() -> Self {
        DupletItem::new("age_at_last_encounter", "age", DupletType::AGEATLASTENCOUNTER)
    }

    pub fn deceased() -> Self {
        DupletItem::new("deceased", "yes/no/na", DupletType::DECEASED)
    }

    pub fn sex() -> Self {
        DupletItem::new("sex", "M:F:O:U", DupletType::SEX)
    }
 
}



// region:    --- Tests

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_pmid()  {
        let row1 = vec!["pmid".to_string(), "title".to_string(), "individual_id".to_string()];
        let row2 = vec!["CURIE".to_string(), "str".to_string(), "str".to_string()];
        let matrix = vec![row1, row2];

        let pmid_duplet = DupletItem::pmid();
        let result = pmid_duplet.check_column_labels(&matrix, 0);
        assert!(result.is_ok());
    
        
    }
}

// endregion: --- Testsq