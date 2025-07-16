//! HpoTermDto
//! 
//! A data transfer object to transfer data about an HPO term from some source such as a GUI to this
//! library. The intention is to transfer all information we need to create a GA4GH Phenopacket Schema PhenotypicFeature message.


use std::str::FromStr;

use ontolius::TermId;
use serde::{Deserialize, Deserializer, Serialize};
use crate::error::{Error, Result};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HpoTermDto {
    /// String representation of an HPO identifier, e.g., HP:0025234
    term_id: String,
    /// Corresponding HPO label, e.g., Parasomnia
    term_label: String,
    /// Entry: can be observed, excluded, na, or a time String
    entry: String,
}

fn deserialize_term_id<'de, D>(deserializer: D) -> std::result::Result<TermId, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    TermId::from_str(&s).map_err(serde::de::Error::custom)
}


impl HpoTermDto {

    pub fn new(
        tid: impl Into<String>,
        label: impl Into<String>,
        entry: impl Into<String>
    ) -> Self {
        Self { 
            term_id: tid.into(), 
            term_label: label.into(), 
            entry: entry.into(),
        }
    }


    pub fn term_id(&self) -> &str {
        &self.term_id
    }

    pub fn ontolius_term_id(&self) -> std::result::Result<TermId, String> {
        TermId::from_str(&self.term_id)
            .map_err(|_| format!("Could not create TermId from '{}'", self.term_id))
    }

    pub fn label(&self) -> String {
        self.term_label.clone()
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

    pub fn onset(&self) -> Result<String> {
        match self.has_onset() {
            true => Ok(self.entry.clone()),
            false => Err(Error::TemplateError{msg: "Attempt to get onset but DTO does not have onset".to_string()})
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
        let dto = HpoTermDto::new(hpo_id, hpo_label, onset);
        assert_eq!(hpo_id, dto.term_id());
        assert_eq!(hpo_label, dto.label());
        assert!(dto.has_onset());
        assert!(! dto.is_excluded());
    }

    #[rstest]
    fn test_excluded_ctor() {
        let hpo_id = "HP:5200362";
        let hpo_label = "Short NREM sleep";
        let dto = HpoTermDto::new(hpo_id, hpo_label, "excluded");
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
        let dto = HpoTermDto::new(hpo_id, hpo_label, onset);
        assert_eq!(hpo_id, dto.term_id());
        assert_eq!(hpo_label, dto.label());
        assert!(dto.has_onset());
        let hpo_onset = dto.onset().unwrap();
        assert_eq!(onset, hpo_onset);
        assert!(! dto.is_excluded());

    }

}
