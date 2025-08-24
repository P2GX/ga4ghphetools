//! CellDto
//! This represents the contents of an HPO cell in the cohort, and can be either
//! observed, excluded, na, Age of Onset, or Modifier.

use std::{collections::HashSet, fmt, str::FromStr};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};


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

fn is_valid_age_string(cell_value: &str) -> bool {
        if cell_value.is_empty() {
            return false;
        }
        if ALLOWED_AGE_LABELS.contains(cell_value) {
            return true;
        }
        if ISO8601_RE.is_match(cell_value) {
            return true;
        } 
        if GESTATIONAL_AGE_RE.is_match(cell_value) {
            return true;
        }
        return false;
}

/// TODO implement!
fn is_valid_hpo_modifier(cell_value: &str) -> bool {
    return false;
}


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")] 
pub enum CellValue {
    Observed,
    Excluded,
    Na,
    OnsetAge(String),   // e.g. "P10Y"
    Modifier(String),   // e.g. "HP:0001250"
}

impl CellValue {
    pub fn is_excluded(&self) -> bool {
        matches!(self, CellValue::Excluded)
    }

    pub fn is_observed(&self) -> bool {
        matches!(self, CellValue::Observed)
    }

    pub fn has_onset(&self) -> bool {
        matches!(self, CellValue::OnsetAge(_))
    }

    pub fn has_modifier(&self) -> bool {
        matches!(self, CellValue::Modifier(_))
    }

    pub fn is_ascertained(&self) -> bool {
       ! matches!(self, CellValue::Na)
    }
}


impl FromStr for CellValue {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "observed" => Ok(CellValue::Observed),
            "excluded" => Ok(CellValue::Excluded),
            "na" => Ok(CellValue::Na),
            _ if is_valid_age_string(s) =>  Ok(CellValue::OnsetAge(s.to_string())),
            _ if is_valid_hpo_modifier(s) => Ok(CellValue::Modifier(s.to_string())),
            _ => Err(format!("Malformed HPO cell contents: '{s}'")),
        }
    }

}

impl fmt::Display for CellValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CellValue::Observed => write!(f, "observed"),
            CellValue::Excluded => write!(f, "excluded"),
            CellValue::Na => write!(f, "na"),
            CellValue::OnsetAge(age) => write!(f, "{}", age),
            CellValue::Modifier(hpo_id) => write!(f, "{}", hpo_id),
        }
    }
}


#[cfg(test)]
mod test {
   
    use rstest::rstest;

    use super::*;
   

    // test malformed entries for the HPO cell value
    #[rstest]
    #[case("")]
    #[case("P2")]
    #[case("Adultonset")]
    #[case("?")]
    #[case("alive")]
    #[case("male")]
    #[case("f")]
    #[case("Observed")]
    #[case("yes")]
    #[case("exc.")] 
    fn test_malformed_entry(
        #[case] entry: &str) 
    {
        let result = CellValue::from_str(entry);
        assert!(result.is_err());
        let err = result.err().unwrap();
        let expected_error = format!("Malformed HPO cell contents: '{entry}'");
        assert_eq!(expected_error, err.to_string());
    }

    #[rstest]
    #[case("observed")]
    #[case("excluded")]
    #[case("Adult onset")]
    #[case("na")]
    fn test_valid_entry(
        #[case] entry: &str) 
    {
        let result = CellValue::from_str(entry);
        assert!(result.is_ok(), "Parsing failed for '{}'", entry);
        let cell_value = result.unwrap();
        assert_eq!(entry, cell_value.to_string(), "Round-trip failed for '{}'", entry);
        match cell_value {
            CellValue::Observed => assert_eq!(entry, "observed"),
            CellValue::Excluded => assert_eq!(entry, "excluded"),
            CellValue::Na => assert_eq!(entry, "na"),
            CellValue::OnsetAge(ref age) => assert_eq!(entry, age),
            CellValue::Modifier(ref hpo) => assert!(false, "Modifier not yet implemented"),
        }
    }
 

}
