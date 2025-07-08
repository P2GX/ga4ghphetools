//! DupletItem
//! 
//! These structures represent the duplet headers of our template
//!    "PMID", "title", "individual_id", "comment", "disease_id", "disease_label", "HGNC_id",	"gene_symbol", 
//!     "transcript", "allele_1", "allele_2", "variant.comment", "age_of_onset", "age_at_last_encounter", 
//!      "deceased", "sex", "HPO", "Clinodactyly of the 5th finger", (etc., arbitrarily many HPO columns)


use std::{collections::HashSet, fmt::format};
use once_cell::sync::Lazy;
use regex::Regex;

use crate::{dto::template_dto::HeaderDupletDto, header::allele_util, hpo::age_util};




static FORBIDDEN_CHARS: Lazy<HashSet<char>> = Lazy::new(|| {
    ['/', '\\', '(', ')'].iter().copied().collect()
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
    SEX,
    HpoSeparator,
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

    fn error_str(&self, h1: &str, h2: &str) -> Result<(), String> {
        if h1 != self.row1 || h2 != self.row2 {
            let column_name = self.get_column_name();
            Err(format!("{}: Expected '{}'/'{}' but got '{}'/'{}'",
                column_name, self.row1, self.row2, h1, h2))
        } else {
            Ok(())
        }
    }

    pub fn check_column_labels(
        &self,
        matrix: &[Vec<String>],
        column: usize,
    ) -> Result<(), String> {
        let actual_row1 = matrix.first()
            .and_then(|row| row.get(column))
            .ok_or_else(|| format!("Missing row 0 or column {}", column))?;
        if actual_row1 != &self.row1 {
            return Err(format!(
                "Row 0, column {}: Expected '{}' but got '{}'",
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
            return Err("Empty CURIE".to_string());
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
        if cell_contents.chars().last().is_some_and(|c| c.is_whitespace()) {
            Err(format!("Trailing whitespace in '{}'", cell_contents))
        } else if cell_contents.chars().next().is_some_and(|c| c.is_whitespace()) {
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
            Err("Value must not be empty".to_string())
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
        if cell_value.is_empty() {
            return Err("Empty age string not allowed (use na)".to_string());
        }
        if cell_value == "na" {
            return Ok(());
        }
        if ALLOWED_AGE_LABELS.contains(cell_value) {
            return Ok(());
        }
        if ISO8601_RE.is_match(cell_value) {
            return Ok(());
        } 
        if GESTATIONAL_AGE_RE.is_match(cell_value) {
            return Ok(());
        }
        Err(format!("Malformed age string '{}'", cell_value))
    }


    fn check_pmid(&self, cell_contents: &str) -> Result<(), String> {
        Self::check_valid_curie(cell_contents)?;
        if !cell_contents.starts_with("PMID") {
            return Err(format!("Invalid PubMed prefix: '{}'", cell_contents));
        }
        Ok(())
    }

    fn check_title(&self, cell_contents: &str) -> Result<(), String> {
        Self::check_white_space(cell_contents)?;
        Self::check_empty(cell_contents)?;
        Ok(())
    }

    fn check_individual_id(&self, cell_contents: &str) -> Result<(), String> {
        Self::check_forbidden_chars(cell_contents)?;
        Self::check_empty(cell_contents)?;
        Self::check_white_space(cell_contents)?;
        Ok(())
    }

    fn check_comment(&self, cell_contents: &str) -> Result<(), String> {
        Self::check_forbidden_chars(cell_contents)?;
        Ok(())
    }

    fn check_disease_id(&self, cell_contents: &str) -> Result<(), String> {
        Self::check_valid_curie(cell_contents)?;
        if !(cell_contents.starts_with("OMIM") || cell_contents.starts_with("MONDO")) {
            return Err(format!("Disease id has invalid prefix: '{}'", cell_contents));
        }
        if cell_contents.starts_with("OMIM:") && cell_contents.len() != 11 {
            return Err(format!("OMIM identifiers must have 6 digits: '{}'", cell_contents));
        }
        Ok(())
    }

    fn check_disease_label(&self, cell_contents: &str) -> Result<(), String> {
        Self::check_empty(cell_contents)?;
        Self::check_white_space(cell_contents)?;
        Ok(())
    }

    fn check_hgnc_id(&self, cell_contents: &str) -> Result<(), String> {
        Self::check_valid_curie(cell_contents)?;
        if ! cell_contents.starts_with("HGNC")  {
            return Err(format!("HGNC id has invalid prefix: '{}'", cell_contents));
        };
        Ok(())
    }

    fn check_gene_symbol(&self, cell_contents: &str) -> Result<(), String> {
        Self::check_empty(cell_contents)?;
        Self::check_white_space(cell_contents)?;
        if cell_contents.contains(" ") {
            return Err(format!("Gene symbol must not contain whitespace: '{cell_contents}'"));
        }
        Ok(())
    }

    fn check_transcript(&self, cell_contents: &str) -> Result<(), String> {
        Self::check_empty(cell_contents)?;
        if ! cell_contents.starts_with("ENST") && ! cell_contents.starts_with("NM_") {
            return Err(format!("Unrecognized transcript prefix '{cell_contents}'"));
        }  
        if ! cell_contents.contains(".") {
            return Err(format!("Transcript '{}' is missing a version", cell_contents));
        } 
        if let Some((before_last, last)) = cell_contents.rsplit_once('.') {
            if before_last.is_empty() {
                return Err(format!("Malformed transcript: '{}'", cell_contents));
            }
            if ! last.chars().all(|c| c.is_ascii_digit()) {
                return Err(format!("Malformed transcript version: '{}'", cell_contents));
            }
        }
        Ok(())
    }

    fn check_allele1(&self, cell_contents: &str) -> Result<(), String> {
        Self::check_empty(cell_contents)?;
        Self::check_white_space(cell_contents)?;
        if cell_contents.starts_with("c.")|| cell_contents.starts_with("n."){
            if ! allele_util::is_plausible_hgvs(cell_contents) {
                return Err(format!("Malformed HGVS string '{cell_contents}'"));
            }
        } else {
            Self::check_valid_structural(cell_contents)?;
        }
        Ok(())
    }

    fn check_allele2(&self, cell_contents: &str) -> Result<(), String> {
        Self::check_empty(cell_contents)?;
        Self::check_white_space(cell_contents)?;
        if cell_contents == "na" {
            return Ok(());
        } else if cell_contents.starts_with("c.") || cell_contents.starts_with("n."){
            allele_util::check_valid_hgvs(cell_contents)?;
        } else {
            Self::check_valid_structural(cell_contents)?;
        }
        Ok(())
    }
    
    fn check_variant_comment(&self, cell_contents: &str) -> Result<(), String> {
        Self::check_tab(cell_contents)?;
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

    fn check_separator(&self, cell_contents: &str) -> Result<(), String> {
        if cell_contents != "na" {
            Err(format!("Separator value must be 'na' but was '{}'", cell_contents))
        } else {
            Ok(())
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
            DupletType::HpoSeparator => self.check_separator(cell_contents)?,
        };
        Ok(())
    }


    fn get_column_name(&self) -> &str {
        match self.duplet_type {
            DupletType::PMID => "PMID",
            DupletType::TITLE => "title",
            DupletType::INDIVIDUALID => "individual_id",
            DupletType::COMMENT => "comment",
            DupletType::DISEASEID => "disease_id",
            DupletType::DISEASELABEL => "disease_label",
            DupletType::HGNCID => "HGNC_id",
            DupletType::GENESYMBOL => "gene_symbol",
            DupletType::TRANSCRIPT => "transcript",
            DupletType::ALLELE1 => "allele_1",
            DupletType::ALLELE2 => "allele_2",
            DupletType::VARIANTCOMMENT => "variant.comment",
            DupletType::AGEOFONSET => "age_of_onset",
            DupletType::AGEATLASTENCOUNTER => "age_at_last_encounter",
            DupletType::DECEASED => "deceased",
            DupletType::SEX => "sex",
            DupletType::HpoSeparator => "HPO",
        }
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
    use rstest::rstest;
    use crate::header::allele_util::check_valid_hgvs;

    #[test]
    fn test_pmid()  {
        let row1 = vec!["PMID".to_string(), "title".to_string(), "individual_id".to_string()];
        let row2 = vec!["CURIE".to_string(), "str".to_string(), "str".to_string()];
        let matrix = vec![row1, row2];

        let pmid_duplet = DupletItem::pmid();
        let result = pmid_duplet.check_column_labels(&matrix, 0);
        assert!(result.is_ok());
    }


    #[rstest]
    #[case("c.6231dup", true)]
    #[case("c.6231_6233dup", true)]
    #[case("c.1932T>A", true)]
    #[case("c.417_418insA", true)]
    #[case("c.112_115delinsG", true)]
    #[case("c.76_78del", true)]  // you allow just 'del' in your logic
    #[case("c.76A>G", true)]
    #[case("c.1177del", true)]
    #[case("c.76_78ins", false)] // missing inserted sequence
    #[case("g.123456A>T", false)] // wrong prefix
    #[case("c.6231inv", false)]   // unsupported type
    #[case("c.", false)]          // incomplete
    fn test_check_valid_hgvs(#[case] input: &str, #[case] should_pass: bool) {
        let validity = check_valid_hgvs(input);
        //assert_eq!(validity, should_pass, "Failed on input: {}", input);
    }


    #[test]
    fn wtf() {
        let re = Regex::new(r"^(c|n)\.\d+(?:_\d+)?dup$").unwrap();
        let test = "c.6231dup";
        //println!("Match? {}", DUPLICATION_RE.is_match(test));
         let validity = check_valid_hgvs(test);
         assert!(validity.is_ok());
    
   // assert_eq!(validity, true, "Failed on input: '{}'", test);
    }

    
}

// endregion: --- Testsq