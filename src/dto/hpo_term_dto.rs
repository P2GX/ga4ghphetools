//! HpoTermDto
//! 
//! A data transfer object to transfer data about an HPO term from some source such as a GUI to this
//! library. The intention is to transfer all information we need to create a GA4GH Phenopacket Schema PhenotypicFeature message.
//! The HpoTermDuplet structure contains the term id and label and is designed for the header. The HpoTermData structure additionally
//! has a value and represents the value of an individual with respect to an HPO term (e.g., observed, P32Y2M, etc.)

use std::str::FromStr;
use ontolius::TermId;
use serde::{Deserialize, Serialize};
use std::fmt;


use crate::age;


/// A structure to represent an HPO term (id and label) in a simple way
#[derive(Clone, Debug, Default, Hash, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HpoTermDuplet {
    pub hpo_label: String,
    pub hpo_id: String,
}


impl HpoTermDuplet {
    pub fn new(label: impl Into<String>, identifier: impl Into<String>) -> Self {
        Self { hpo_label: label.into(), hpo_id: identifier.into() }
    }

    pub fn hpo_id(&self) -> &str {
        &self.hpo_id
    }

    pub fn hpo_label(&self) -> &str {
        &self.hpo_label
    }

    pub fn to_term_id(&self) -> std::result::Result<TermId, String> {
        let tid = TermId::from_str(&self.hpo_id).map_err(|_| format!("Could not create TermId from {}", self.hpo_id()))?;
        Ok(tid)
    }
    
} 



/*
pub static ISO8601_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^P(?:(\d+)Y)?(?:(\d+)M)?(?:(\d+)D)?$").expect("valid ISO 8601 regex")
});

/// Regex for gestational age format
pub static GESTATIONAL_AGE_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"G\d+w[0-6]d").expect("valid gestational age regex")
}); */



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

    pub fn is_valid_cell_value(s: &str) -> bool {
          match s {
            "observed" => true,
            "excluded" => true,
            "na" => true,
            _ if age::is_valid_age_string(s) =>  true,
            _ if is_valid_hpo_modifier(s) => true,
            _ => false,
        }
    }
}


impl FromStr for CellValue {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "observed" => Ok(CellValue::Observed),
            "excluded" => Ok(CellValue::Excluded),
            "na" => Ok(CellValue::Na),
            _ if age::is_valid_age_string(s) =>  Ok(CellValue::OnsetAge(s.to_string())),
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



/// A structure to represent the HPO term together with a value.

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HpoTermData {
    pub term_duplet: HpoTermDuplet,
    pub entry: CellValue,
}


impl HpoTermData {

    pub fn new(
        term_duplet: HpoTermDuplet,
        entry: CellValue
    ) -> Result<Self, String> {
        Ok(Self { 
           term_duplet: term_duplet,
            entry 
        })
    }

    pub fn from_duplet(
        duplet: HpoTermDuplet,
        entry: &str
    ) -> Result<Self, String> {
        Ok(Self { 
            term_duplet: duplet, 
            entry: CellValue::from_str(entry)? 
        })
    }

     pub fn from_str(
        term_id: &str,
        term_label: &str,
        entry: &str
    ) -> Result<Self, String> {
        Ok(Self { 
           term_duplet: HpoTermDuplet::new(term_label, term_id),
            entry : CellValue::from_str(entry)?
        })
    }


    pub fn term_id(&self) -> &str {
        &self.term_duplet.hpo_id()
    }

    pub fn ontolius_term_id(&self) -> std::result::Result<TermId, String> {
        self.term_duplet.to_term_id()
    }

    pub fn label(&self) -> &str {
        self.term_duplet.hpo_label()
    }

    pub fn is_excluded(&self) -> bool {
        self.entry == CellValue::Excluded
    }

    pub fn is_observed(&self) -> bool {
        self.entry == CellValue::Observed
    }

    pub fn is_ascertained(&self) -> bool {
        self.entry != CellValue::Na
    }

    pub fn is_not_ascertained(&self) -> bool {
        self.entry ==  CellValue::Na
    }

    pub fn has_onset(&self) -> bool {
        matches!(self.entry, CellValue::OnsetAge{..})
    }

    pub fn onset_value(&self) -> Option<&str> {
    if let CellValue::OnsetAge(s) = &self.entry {
        Some(s)
    } else {
        None
    }
}

    pub fn entry(&self) -> String {
        self.entry.to_string()
    }

}


#[cfg(test)]
mod test {
    use super::*;
    use rstest::rstest;



    #[rstest]
    fn test_cell_value_type() {
        let cv = CellValue::from_str("P32Y").unwrap();
        println!("{:?}", cv);
        assert!(matches!(cv, CellValue::OnsetAge(..)))
    }


    #[rstest]
    fn test_observed_ctor() {
        let hpo_id = "HP:5200362";
        let hpo_label = "Short NREM sleep";
        let onset = "P29Y";
        let dto = HpoTermData::from_str(hpo_id, hpo_label, onset).unwrap();
        assert_eq!(hpo_id, dto.term_id());
        assert_eq!(hpo_label, dto.label());
        println!("{:?}", dto);
       // assert!(dto.has_onset());
        assert!(! dto.is_excluded());
    }

    #[rstest]
    fn test_excluded_ctor() {
        let hpo_id = "HP:5200362";
        let hpo_label = "Short NREM sleep";
        let dto = HpoTermData::from_str(hpo_id, hpo_label, "excluded").unwrap();
        assert_eq!(hpo_id, dto.term_id());
        assert_eq!(hpo_label, dto.label());
        assert!(! dto.has_onset());
        assert!(dto.is_excluded());
    }

    #[rstest]
    fn test_with_onset_ctor() {
        let hpo_id = "HP:5200362";
        let hpo_label = "Short NREM sleep";
        let onset = "Young adult onset";
        let dto = HpoTermData::from_str(hpo_id, hpo_label, onset).unwrap();
        assert_eq!(hpo_id, dto.term_id());
        assert_eq!(hpo_label, dto.label());
        assert!(dto.has_onset());
        let hpo_onset = dto.onset_value().unwrap();
        assert_eq!(onset, hpo_onset);
        assert!(! dto.is_excluded());

    }


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
