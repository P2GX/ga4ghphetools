//! DupletItemField checks
//! 
//! Functions to qc fields of the CohortData


use std::collections::HashSet;
use std::sync::LazyLock;
use regex::Regex;

use crate::dto::cohort_data::{GeneVariantData, IndividualData};




static FORBIDDEN_CHARS: LazyLock<HashSet<char>> = LazyLock::new(|| {
    ['/', '\\'].iter().copied().collect()
});

/// We allow RefSeq (NM_, NR_) or Ensembl (ENST) transcript identifiers
static ALLOWED_TRANSCRIPT_PREFIXES: &[&str] = &[
    "NM_",
    "NR_",
    "ENST",
];

pub static ALLOWED_DECEASED_ITEMS: LazyLock<HashSet<String>> = LazyLock::new(||{
    let mut hset = HashSet::new();
    hset.insert("yes".to_string());
    hset.insert("no".to_string());
    hset.insert("na".to_string());
    hset
});

pub static ALLOWED_SEX_ITEMS: LazyLock<HashSet<String>> = LazyLock::new(||{
    let mut hset = HashSet::new();
    hset.insert("M".to_string());
    hset.insert("F".to_string());
    hset.insert("O".to_string());
    hset.insert("U".to_string());
    hset
});

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


pub(crate) fn check_disease_id(cell_contents: &str) -> Result<(), String> {
    check_valid_curie(cell_contents)?;
    if !(cell_contents.starts_with("OMIM") || cell_contents.starts_with("MONDO")) {
        return Err(format!("Disease id has invalid prefix: '{}'", cell_contents));
    }
    if cell_contents.starts_with("OMIM:") && cell_contents.len() != 11 {
        return Err(format!("OMIM identifiers must have 6 digits: '{}'", cell_contents));
    }
    Ok(())
}

pub(crate) fn check_disease_label(cell_contents: &str) -> Result<(), String> {
    check_empty(cell_contents)?;
    check_white_space(cell_contents)?;
    Ok(())
}



pub(crate) fn check_pmid(s: &str) -> Result<(), String> {
    check_valid_curie(s)?;
    if !s.starts_with("PMID") {
        return Err(format!("Invalid PubMed prefix: '{}'", s));
    }
    Ok(())
}
pub(crate) fn check_title(s: &str) -> Result<(), String> {
    check_white_space(s)?;
    check_empty(s)
}
pub(crate) fn check_individual_id(s: &str) -> Result<(), String> {
    check_forbidden_chars(s)?;
    check_empty(s)?;
    check_white_space(s)
}
pub(crate) fn check_comment(s: &str) -> Result<(), String> {
    check_forbidden_chars(s)
}
pub(crate) fn check_age_string(s: &str) -> Result<(), String> { 
     if s.is_empty() {
            return Err("Empty age string not allowed (use na)".to_string());
        }
    if crate::age::is_valid_age_string(s) {
        Ok(())
    } else {
        Err(format!("Malformed age string '{}'", s))
    }
}



pub(crate) fn check_deceased(s: &str) -> Result<(), String> {
    if ALLOWED_DECEASED_ITEMS.contains(s) { Ok(()) } else { Err(format!("Malformed deceased entry: '{}'", s)) }
}
pub(crate) fn check_sex(s: &str) -> Result<(), String> {
    if ALLOWED_SEX_ITEMS.contains(s) { Ok(()) } else { Err(format!("Malformed sex entry: '{}'", s)) }
}

/// Same order as the old IndividualHeader::qc_data.
pub(crate) fn check_individual_data(d: &IndividualData) -> Result<(), String> {
    check_pmid(&d.pmid)?;
    check_title(&d.title)?;
    check_individual_id(&d.individual_id)?;
    check_comment(&d.comment)?;
    check_age_string(&d.age_of_onset)?;
    check_age_string(&d.age_at_last_encounter)?;
    check_deceased(&d.deceased)?;
    check_sex(&d.sex)
}



/// We will be sending all HGVS variants to variant validator. Here, we just do 
/// a rough screening to reject some obvious mistakes.
/// - must start with c./n.
/// - must not contain whitespace
/// - must contain at least one digit
/// - If has '>', must have bases before and after
/// - If ins insertion, must have bases after 'ins'
pub fn is_plausible_hgvs(hgvs: &str) -> bool {
    if !(hgvs.starts_with("c.") || hgvs.starts_with("n.")) {
        return false;
    }
    if hgvs.contains(char::is_whitespace) {
        return false;
    }
    if !hgvs.chars().any(|c| c.is_ascii_digit()) {
        return false;
    }
    if let Some(pos) = hgvs.find('>') {
        // get the characters before and after '>'
        let (before, after) = (&hgvs[..pos], &hgvs[pos + 1..]);
        if !before.chars().rev().take_while(|c| c.is_ascii_alphabetic()).all(|c| "ACGT".contains(c)) {
            return false;
        }
        if !after.chars().take_while(|c| c.is_ascii_alphabetic()).all(|c| "ACGT".contains(c)) {
            return false;
        }
    }

    if let Some(pos) = hgvs.find("ins") {
        let after = &hgvs[pos + 3..]; // after 'ins'
        if after.is_empty() || !after.chars().all(|c| "ACGT".contains(c)) {
            return false;
        }
    }
    true
}


pub(crate) fn check_hgnc_id(s: &str) -> Result<(), String> {
    check_valid_curie(s)?;
    if !s.starts_with("HGNC") {
        return Err(format!("HGNC id has invalid prefix: '{}'", s));
    }
    Ok(())
}

pub(crate) fn check_gene_symbol(s: &str) -> Result<(), String> {
    check_empty(s)?;
    check_white_space(s)?;
    if s.contains(' ') {
        return Err(format!("Gene symbol must not contain whitespace: '{s}'"));
    }
    Ok(())
}

pub(crate) fn check_transcript(s: &str) -> Result<(), String> {
    check_empty(s)?;
    if !ALLOWED_TRANSCRIPT_PREFIXES.iter().any(|p| s.starts_with(p)) {
        return Err(format!("Unrecognized transcript prefix '{s}'"));
    }
    if !s.contains('.') {
        return Err(format!("Transcript '{}' is missing a version", s));
    }
    if let Some((before_last, last)) = s.rsplit_once('.') {
        if before_last.is_empty() {
            return Err(format!("Malformed transcript: '{}'", s));
        }
        if last.is_empty() || !last.chars().all(|c| c.is_ascii_digit()) {
            return Err(format!("Malformed transcript version: '{}'", s));
        }
    }
    Ok(())
}

pub(crate) fn check_gene_variant_data(d: &GeneVariantData) -> Result<(), String> {
    check_hgnc_id(&d.hgnc_id)?;
    check_gene_symbol(&d.gene_symbol)?;
    check_transcript(&d.transcript)?;
    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    // ---------- check_valid_curie ----------

    #[rstest]
    #[case("OMIM:135100")]
    #[case("MONDO:0007947")]
    #[case("HGNC:3603")]
    fn curie_valid(#[case] s: &str) {
        assert!(check_valid_curie(s).is_ok());
    }

    #[rstest]
    #[case("", "Empty CURIE")]
    #[case("OMIM135100", "Invalid CURIE with no colon: 'OMIM135100'")]
    #[case(":135100", "Invalid CURIE with no prefix: ':135100'")]
    #[case("OMIM:", "Invalid CURIE with no suffix: 'OMIM:'")]
    #[case("OMIM: 135100", "Contains stray whitespace: 'OMIM: 135100'")]
    #[case(" OMIM:135100", "Contains stray whitespace: ' OMIM:135100'")]
    #[case("OMIM:13510a", "Invalid CURIE with non-digit characters in suffix: 'OMIM:13510a'")]
    fn curie_invalid(#[case] s: &str, #[case] expected: &str) {
        assert_eq!(expected, check_valid_curie(s).unwrap_err());
    }

    /// The message for multiple colons is missing its closing quote in the source.
    /// This test only pins that such input is rejected, not the exact text.
    #[test]
    fn curie_multiple_colons_rejected() {
        let err = check_valid_curie("OMIM:1:2").unwrap_err();
        assert!(err.starts_with("Invalid CURIE with more than one colon"));
    }

    // ---------- check_empty ----------

    #[test]
    fn empty_rejects_empty_string() {
        assert_eq!("Value must not be empty", check_empty("").unwrap_err());
    }

    #[rstest]
    #[case("x")]
    #[case(" ")] // whitespace is not "empty" for this check
    fn empty_accepts_non_empty(#[case] s: &str) {
        assert!(check_empty(s).is_ok());
    }

    // ---------- check_white_space ----------

    #[rstest]
    #[case("")]
    #[case("Marfan syndrome")]
    #[case("a b c")]
    fn white_space_valid(#[case] s: &str) {
        assert!(check_white_space(s).is_ok());
    }

    #[rstest]
    #[case("Marfan ", "Trailing whitespace in 'Marfan '")]
    #[case("Marfan\t", "Trailing whitespace in 'Marfan\t'")]
    #[case(" Marfan", "Leading whitespace in ' Marfan'")]
    #[case("Marfan  syndrome", "Consecutive whitespace in 'Marfan  syndrome'")]
    // a lone space is both leading and trailing; trailing is reported first
    #[case(" ", "Trailing whitespace in ' '")]
    fn white_space_invalid(#[case] s: &str, #[case] expected: &str) {
        assert_eq!(expected, check_white_space(s).unwrap_err());
    }

    // ---------- check_forbidden_chars ----------

    #[rstest]
    #[case("")]
    #[case("patient_1")]
    #[case("Smith-2020, II.3")]
    fn forbidden_chars_valid(#[case] s: &str) {
        assert!(check_forbidden_chars(s).is_ok());
    }

    #[rstest]
    #[case("a/b", '/')]
    #[case("a\\b", '\\')]
    fn forbidden_chars_invalid(#[case] s: &str, #[case] c: char) {
        let err = check_forbidden_chars(s).unwrap_err();
        assert_eq!(format!("Forbidden character '{c}' found in label '{s}'"), err);
    }

    // ---------- check_disease_id ----------

    #[rstest]
    #[case("OMIM:605407")]
    #[case("OMIM:135100")]
    #[case("MONDO:0007947")]
    fn disease_id_valid(#[case] s: &str) {
        assert!(check_disease_id(s).is_ok());
    }

    #[rstest]
    #[case("MIM:135100", "Disease id has invalid prefix: 'MIM:135100'")]
    #[case("omim:135100", "Disease id has invalid prefix: 'omim:135100'")]
    #[case("ORPHA:558", "Disease id has invalid prefix: 'ORPHA:558'")]
    #[case("OMIM: 135100", "Contains stray whitespace: 'OMIM: 135100'")]
    #[case("OMIM:13510", "OMIM identifiers must have 6 digits: 'OMIM:13510'")]
    #[case("OMIM:1351000", "OMIM identifiers must have 6 digits: 'OMIM:1351000'")]
    #[case("", "Empty CURIE")]
    #[case("OMIM135100", "Invalid CURIE with no colon: 'OMIM135100'")]
    fn disease_id_invalid(#[case] s: &str, #[case] expected: &str) {
        assert_eq!(expected, check_disease_id(s).unwrap_err());
    }

    /// MONDO identifiers are not length-checked, only prefix and numeric suffix.
    #[test]
    fn disease_id_mondo_length_not_restricted() {
        assert!(check_disease_id("MONDO:1").is_ok());
    }

    /// Known gap: the prefix test is `starts_with("OMIM")`, so a prefix that merely
    /// begins with OMIM or MONDO is accepted. Remove #[ignore] after tightening it.
    #[test]
    #[ignore = "known gap: prefix is matched with starts_with, not exactly"]
    fn disease_id_prefix_must_match_exactly() {
        assert!(check_disease_id("OMIMX:123456").is_err());
        assert!(check_disease_id("MONDOX:0007947").is_err());
    }

    // ---------- check_disease_label ----------

    #[test]
    fn disease_label_valid() {
        assert!(check_disease_label("Segawa syndrome, recessive").is_ok());
    }

    #[rstest]
    #[case("", "Value must not be empty")]
    #[case("Fibrodysplasia ossificans progressiva ", "Trailing whitespace in 'Fibrodysplasia ossificans progressiva '")]
    #[case(" Fibrodysplasia", "Leading whitespace in ' Fibrodysplasia'")]
    #[case("Segawa  syndrome", "Consecutive whitespace in 'Segawa  syndrome'")]
    fn disease_label_invalid(#[case] s: &str, #[case] expected: &str) {
        assert_eq!(expected, check_disease_label(s).unwrap_err());
    }

    #[rstest]
    #[case("PMID29482508", "Invalid CURIE with no colon: 'PMID29482508'")]
    #[case("PMID: 29482508", "Contains stray whitespace: 'PMID: 29482508'")]
    #[case("", "Empty CURIE")]
    #[case("MID:29482508", "Invalid PubMed prefix: 'MID:29482508'")]
    fn pmid_invalid(#[case] s: &str, #[case] expected: &str) {
        assert_eq!(expected, check_pmid(s).unwrap_err());
    }

    #[rstest]
    #[case("individual\\1", "Forbidden character '\\' found in label 'individual\\1'")]
    #[case("individual  A", "Consecutive whitespace in 'individual  A'")]
    #[case("", "Value must not be empty")]
    fn individual_id_invalid(#[case] s: &str, #[case] expected: &str) {
        assert_eq!(expected, check_individual_id(s).unwrap_err());
    }

    #[test]
    fn title_consecutive_whitespace() {
        assert_eq!("Consecutive whitespace in 'a  b'", check_title("a  b").unwrap_err());
    }

    #[rstest]
    #[case("P2")]
    #[case("P22")]
    #[case("Adultonset")]
    fn age_invalid(#[case] s: &str) {
        assert_eq!(format!("Malformed age string '{s}'"), check_age_string(s).unwrap_err());
    }

    #[rstest]
    #[case("P2Y")]
    #[case("P12Y6M")]
    #[case("Adult onset")]
    #[case("na")]
    #[case("G32w3d")]
    fn age_valid(#[case] s: &str) { assert!(check_age_string(s).is_ok()); }

    #[rstest]
    #[case("?", "Malformed deceased entry: '?'")]
    #[case("alive", "Malformed deceased entry: 'alive'")]
    fn deceased_invalid(#[case] s: &str, #[case] expected: &str) {
        assert_eq!(expected, check_deceased(s).unwrap_err());
    }

    #[rstest]
    #[case("male", "Malformed sex entry: 'male'")]
    #[case("f", "Malformed sex entry: 'f'")]
    fn sex_invalid(#[case] s: &str, #[case] expected: &str) {
        assert_eq!(expected, check_sex(s).unwrap_err());
    }

    #[test]
    fn individual_data_valid_and_first_error_wins() {
        let ok = IndividualData::new("PMID:29482508", "A title", "individual A", "comment",
                                    "P2Y", "Adult onset", "no", "F");
        assert!(check_individual_data(&ok).is_ok());
        let bad = IndividualData::new("PMID29482508", "A title", "individual A", "comment",
                                    "P2Y", "Adult onset", "no", "X"); // two errors
        assert_eq!("Invalid CURIE with no colon: 'PMID29482508'", check_individual_data(&bad).unwrap_err());
    }

    #[rstest]
    #[case("HGNC:3603")]
    fn hgnc_valid(#[case] s: &str) { assert!(check_hgnc_id(s).is_ok()); }

    #[rstest]
    #[case("", "Empty CURIE")]
    #[case("HGNC3603", "Invalid CURIE with no colon: 'HGNC3603'")]
    #[case("HGNC: 3603", "Contains stray whitespace: 'HGNC: 3603'")]
    #[case("HGNC:36a", "Invalid CURIE with non-digit characters in suffix: 'HGNC:36a'")]
    #[case("NCBI:1234", "HGNC id has invalid prefix: 'NCBI:1234'")]
    fn hgnc_invalid(#[case] s: &str, #[case] expected: &str) {
        assert_eq!(expected, check_hgnc_id(s).unwrap_err());
    }

    #[rstest]
    #[case("FBN1")]
    #[case("MT-ND1")]
    fn gene_symbol_valid(#[case] s: &str) { assert!(check_gene_symbol(s).is_ok()); }

    #[rstest]
    #[case("", "Value must not be empty")]
    #[case(" FBN1", "Leading whitespace in ' FBN1'")]
    #[case("FBN1 ", "Trailing whitespace in 'FBN1 '")]
    #[case("FB  N1", "Consecutive whitespace in 'FB  N1'")]
    #[case("FBN 1", "Gene symbol must not contain whitespace: 'FBN 1'")]
    fn gene_symbol_invalid(#[case] s: &str, #[case] expected: &str) {
        assert_eq!(expected, check_gene_symbol(s).unwrap_err());
    }

    #[rstest]
    #[case("NM_000138.5")]
    #[case("NR_024540.1")]
    #[case("ENST00000316623.10")]
    fn transcript_valid(#[case] s: &str) { assert!(check_transcript(s).is_ok()); }

    #[rstest]
    #[case("", "Value must not be empty")]
    #[case("XM_000138.5", "Unrecognized transcript prefix 'XM_000138.5'")]
    #[case("NM_000138", "Transcript 'NM_000138' is missing a version")]
    #[case("NM_000138.x", "Malformed transcript version: 'NM_000138.x'")]
    #[case("NM_000138.", "Malformed transcript version: 'NM_000138.'")]
    fn transcript_invalid(#[case] s: &str, #[case] expected: &str) {
        assert_eq!(expected, check_transcript(s).unwrap_err());
    }

}