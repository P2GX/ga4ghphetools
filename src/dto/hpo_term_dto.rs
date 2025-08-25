//! HpoTermDto
//! 
//! A data transfer object to transfer data about an HPO term from some source such as a GUI to this
//! library. The intention is to transfer all information we need to create a GA4GH Phenopacket Schema PhenotypicFeature message.
//! The HpoTermDuplet structure contains the term id and label and is designed for the header. The HpoTermData structure additionally
//! has a value and represents the value of an individual with respect to an HPO term (e.g., observed, P32Y2M, etc.)

use std::str::FromStr;
use ontolius::TermId;
use serde::{Deserialize, Deserializer, Serialize};


/// A structure to represent an HPO term (id and label) in a simple way
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
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




/// A structure to represent the HPO term together with a value.

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HpoTermData {
    term_duplet: HpoTermDuplet,
    entry: String,
}

fn deserialize_term_id<'de, D>(deserializer: D) -> std::result::Result<TermId, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    TermId::from_str(&s).map_err(serde::de::Error::custom)
}


impl HpoTermData {

    pub fn new(
        tid: &str,
        label: &str,
        entry: &str
    ) -> Self {
        let duplet = HpoTermDuplet::new(label, tid);
        Self { 
           term_duplet: duplet,
            entry: entry.into(),
        }
    }

    pub fn from_duplet(
        duplet: HpoTermDuplet,
        entry: &str
    ) -> Self {
        Self { term_duplet: duplet, entry: entry.to_string() }
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
        self.entry == "excluded"
    }

    pub fn is_observed(&self) -> bool {
        self.entry == "observed"
    }

    pub fn is_ascertained(&self) -> bool {
        self.entry != "na"
    }

    pub fn is_not_ascertained(&self) -> bool {
        self.entry == "na"
    }

    pub fn has_onset(&self) -> bool {
        (! self.is_excluded() ) && (! self.is_observed() ) && (self.is_ascertained())
    }

    pub fn onset(&self) -> Result<String, String> {
        match self.has_onset() {
            true => Ok(self.entry.clone()),
            false => Err("Attempt to get onset but DTO does not have onset".to_string())
        }
    }

    pub fn entry(&self) -> &str {
        &self.entry
    }

}


#[cfg(test)]
mod test {
    use super::*;
    use rstest::rstest;

    #[rstest]
    fn test_observed_ctor() {
        let hpo_id = "HP:5200362";
        let hpo_label = "Short NREM sleep";
        let onset = "P29Y";
        let dto = HpoTermData::new(hpo_id, hpo_label, onset);
        assert_eq!(hpo_id, dto.term_id());
        assert_eq!(hpo_label, dto.label());
        assert!(dto.has_onset());
        assert!(! dto.is_excluded());
    }

    #[rstest]
    fn test_excluded_ctor() {
        let hpo_id = "HP:5200362";
        let hpo_label = "Short NREM sleep";
        let dto = HpoTermData::new(hpo_id, hpo_label, "excluded");
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
        let dto = HpoTermData::new(hpo_id, hpo_label, onset);
        assert_eq!(hpo_id, dto.term_id());
        assert_eq!(hpo_label, dto.label());
        assert!(dto.has_onset());
        let hpo_onset = dto.onset().unwrap();
        assert_eq!(onset, hpo_onset);
        assert!(! dto.is_excluded());

    }

}
