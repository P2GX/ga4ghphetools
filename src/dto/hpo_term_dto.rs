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
use std::ops::Deref;


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



#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CellValue {
    // 👇 This un-nests the CellValue JSON so it looks exactly like your old data
    #[serde(flatten)]
    pub entry: CellValueInner,

    // 👇 Safely defaults to empty if missing from old files
    #[serde(default)]
    pub modifiers: Vec<String>,
}

/// HpoCell will automatically act like a CellValue when you 
/// try to access its inner methods or match against it.
impl Deref for CellValue {
    type Target = CellValueInner;

    fn deref(&self) -> &Self::Target {
        &self.entry
    }
}

impl From<CellValueInner> for CellValue {
    fn from(entry: CellValueInner) -> Self {
        CellValue {
            entry,
            modifiers: Vec::new(),
        }
    }
}

impl FromStr for CellValue {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
       let cv_inner = match s {
            "observed" => Ok(CellValueInner::Observed),
            "excluded" => Ok(CellValueInner::Excluded),
            "na" => Ok(CellValueInner::Na),
            _ if age::is_valid_age_string(s) =>  Ok(CellValueInner::OnsetAge(s.to_string())),
            _ => Err(format!("Malformed HPO cell contents: '{s}'")),
        };
        Ok(CellValue { entry: cv_inner?, modifiers: Vec::default()})
    }
}


#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")] 
pub enum CellValueInner {
    Observed,
    Excluded,
    Na,
    OnsetAge(String),   // e.g. "P10Y"
}

impl CellValueInner {
    pub fn is_excluded(&self) -> bool {
        matches!(self, CellValueInner::Excluded)
    }

    pub fn is_observed(&self) -> bool {
        matches!(self, CellValueInner::Observed)
    }

    pub fn has_onset(&self) -> bool {
        matches!(self, CellValueInner::OnsetAge(_))
    }

    pub fn is_ascertained(&self) -> bool {
       ! matches!(self, CellValueInner::Na)
    }

    pub fn is_valid_cell_value(s: &str) -> bool {
          match s {
            "observed" => true,
            "excluded" => true,
            "na" => true,
            _ if age::is_valid_age_string(s) =>  true,
            _ => false,
        }
    }
}


impl FromStr for CellValueInner {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "observed" => Ok(CellValueInner::Observed),
            "excluded" => Ok(CellValueInner::Excluded),
            "na" => Ok(CellValueInner::Na),
            _ if age::is_valid_age_string(s) =>  Ok(CellValueInner::OnsetAge(s.to_string())),
            _ => Err(format!("Malformed HPO cell contents: '{s}'")),
        }
    }
}

impl fmt::Display for CellValueInner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CellValueInner::Observed => write!(f, "observed"),
            CellValueInner::Excluded => write!(f, "excluded"),
            CellValueInner::Na => write!(f, "na"),
            CellValueInner::OnsetAge(age) => write!(f, "{}", age),
        }
    }
}

impl fmt::Display for CellValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let entry_str = match &self.entry {
            CellValueInner::Observed => "observed".to_string(),
            CellValueInner::Excluded => "excluded".to_string(),
            CellValueInner::Na => "na".to_string(),
            CellValueInner::OnsetAge(age) => age.clone(),
        };
        if self.modifiers.is_empty() {
            write!(f, "{}", entry_str)
        } else {
            // Joins ["mod1", "mod2"] into "mod1, mod2"
            let mods_str = self.modifiers.join(", "); 
            write!(f, "{} ({})", entry_str, mods_str)
        }
    }
}



impl CellValue {
    pub fn na() -> Self {
        Self {
            entry: CellValueInner::Na,
            modifiers: Vec::default(),
        }
    }

    pub fn observed() -> Self {
        Self {
            entry: CellValueInner::Observed,
            modifiers: Vec::default(),
        }
    }

    pub fn excluded() -> Self {
        Self {
            entry: CellValueInner::Excluded,
            modifiers: Vec::default(),
        }
    }

    pub fn onset(onset: impl Into<String>) -> Self {
         Self {
            entry: CellValueInner::OnsetAge(onset.into()),
            modifiers: Vec::default(),
        }
    }

    pub fn has_modifier(&self) -> bool {
        ! self.modifiers.is_empty()
    }

    pub fn modifers(&self) -> &[String] {
        &self.modifiers
    }
}



/// A structure to represent the HPO term together with a value.
/// Optionally, a one or more HPO modifiers can be added.
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
            entry,
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
            entry : CellValue::from_str(entry)?,
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
        self.entry.entry == CellValueInner::Excluded
    }

    pub fn is_observed(&self) -> bool {
        self.entry.entry == CellValueInner::Observed
    }

    pub fn is_ascertained(&self) -> bool {
        self.entry.entry != CellValueInner::Na
    }

    pub fn is_not_ascertained(&self) -> bool {
        self.entry.entry ==  CellValueInner::Na
    }

    pub fn has_onset(&self) -> bool {
        matches!(self.entry.entry, CellValueInner::OnsetAge{..})
    }

    pub fn onset_value(&self) -> Option<&str> {
        if let CellValueInner::OnsetAge(s) = &self.entry.entry {
            Some(s)
        } else {
            None
        }
    }

    pub fn entry(&self) -> String {
        self.entry.to_string()
    }

    pub fn modifiers(&self) -> Vec<String> {
        self.entry.modifiers.clone()
    }

    pub fn has_modifier(&self) -> bool {
       ! self.entry.modifiers.is_empty()
    }

}


#[cfg(test)]
mod test {
    use super::*;
    use rstest::rstest;



    #[rstest]
    fn test_cell_value_type() {
        let cv = CellValueInner::from_str("P32Y").unwrap();
        println!("{:?}", cv);
        assert!(matches!(cv, CellValueInner::OnsetAge(..)))
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
        let result = CellValueInner::from_str(entry);
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
        let result = CellValueInner::from_str(entry);
        assert!(result.is_ok(), "Parsing failed for '{}'", entry);
        let cell_value = result.unwrap();
        assert_eq!(entry, cell_value.to_string(), "Round-trip failed for '{}'", entry);
        match cell_value {
            CellValueInner::Observed => assert_eq!(entry, "observed"),
            CellValueInner::Excluded => assert_eq!(entry, "excluded"),
            CellValueInner::Na => assert_eq!(entry, "na"),
            CellValueInner::OnsetAge(ref age) => assert_eq!(entry, age),
        }
    }

    /// Our EtlDto will include cells that have JSON content similar to the below
    /// Here we just perform a sanity test.
    #[rstest]
    fn matest_deserilaize()  {
        // Example JSON string from the frontend
        let json_cell = r#"
        [
            { "termDuplet": { "hpoLabel": "Vomiting", "hpoId": "HP:0002013" }, "entry": { "type": "Observed" } },
            { "termDuplet": { "hpoLabel": "Nausea", "hpoId": "HP:0002015" }, "entry": { "type": "OnsetAge", "data": "P3Y" } }
        ]
        "#;

        let hpo_terms: Vec<HpoTermData> = serde_json::from_str(json_cell).unwrap();
        assert_eq!(2, hpo_terms.len());
    }

    #[rstest]
    fn test_modifier() {
        // Severe  HP:0012828
        let duplet = HpoTermDuplet::new("Cardiomyopathy", "HP:0001638");
        let cval = CellValue{ entry: CellValueInner::Observed, modifiers: vec!["HP:0012828".to_string()]  };
        let hpo_data = HpoTermData { 
            term_duplet: duplet, 
            entry: cval
        };
        assert!(hpo_data.has_modifier());
        let modfr = hpo_data.modifiers();
        assert_eq!(1, modfr.len());
        assert_eq!("HP:0012828", modfr[0]);
    }

 

}
