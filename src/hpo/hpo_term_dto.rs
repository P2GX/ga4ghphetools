//! HpoTermDto
//! 
//! A data transfer object to transfer data about an HPO term from some source such as a GUI to this
//! library. The intention is to transfer all information we need to create a GA4GH Phenopacket Schema PhenotypicFeature message.

use reqwest::header::Entry;

use crate::template::excel::read_excel_to_dataframe;


#[derive(Debug, Clone)]
pub struct HpoTermDto {
    /// String representation of an HPO identifier, e.g., HP:0025234
    term_id: String,
    /// Corresponding HPO label, e.g., Parasomnia
    term_label: String,
    /// Optional String representing age of onset, e.g., P2Y3M, Congenital onset, G34w2d
    age_of_onset: Option<String>,
    /// True only if the terms was explicitly excluded
    excluded: bool,
    /// True if the corresponding entry was empty or "na"
    ascertained: bool
}

impl HpoTermDto {

    fn observed(
        tid: impl Into<String>,
        label: impl Into<String>,
    ) -> Self {
        Self { 
            term_id: tid.into(), 
            term_label: label.into(), 
            age_of_onset: None, 
            excluded: false,
            ascertained: true,
        }
    }

    fn observed_with_onset(
        tid: impl Into<String>,
        label: impl Into<String>,
        onset: impl Into<String>
    ) -> Self {
        Self { 
            term_id: tid.into(), 
            term_label: label.into(), 
            age_of_onset: Some(onset.into()), 
            excluded: false,
            ascertained: true
        }
    }

    fn excluded(
        tid: impl Into<String>,
        label: impl Into<String>,
    ) -> Self {
        Self { 
            term_id: tid.into(), 
            term_label: label.into(), 
            age_of_onset: None, 
            excluded: true,
            ascertained: true
        }
    }

    fn not_ascertained(
        tid: impl Into<String>,
        label: impl Into<String>,
    ) -> Self {
        Self { 
            term_id: tid.into(), 
            term_label: label.into(), 
            age_of_onset: None, 
            excluded: false,
            ascertained: false
        }
    }

    /// For data transfer from an input table, e.g., from a GUI. This function does not check for validity, that is
    /// done elsewhere
    pub fn from_table_entry(
        tid: impl Into<String>,
        label: impl Into<String>,
        entry: impl Into<String>
    ) -> Self
    {
        let entry = entry.into();
        match entry.as_str() {
            "na"  => HpoTermDto::not_ascertained(tid, label),
            "observed" => HpoTermDto::observed(tid, label),
            "excluded" => HpoTermDto::excluded(tid, label),
            _ => HpoTermDto::observed_with_onset(tid, label, entry)
        }
    }


    pub fn term_id(&self) -> String {
        self.term_id.clone()
    }

    pub fn label(&self) -> String {
        self.term_label.clone()
    }

    pub fn is_excluded(&self) -> bool {
        self.excluded
    }

    pub fn has_onset(&self) -> bool {
        self.age_of_onset.is_some()
    }

    pub fn onset(&self) -> Option<String> {
        self.age_of_onset.clone()
    }

}


#[cfg(test)]
mod test {
    use crate::{error::Error, header::{header_duplet::HeaderDupletItem, hpo_term_duplet::HpoTermDuplet}};
    use super::*;
    use ontolius::common::hpo;
    use rstest::rstest;
   

    #[rstest]
    fn test_observed_ctor() {
        let hpo_id = "HP:5200362";
        let hpo_label = "Short NREM sleep";
        let dto = HpoTermDto::observed(hpo_id, hpo_label);
        assert_eq!(hpo_id, dto.term_id());
        assert_eq!(hpo_label, dto.label());
        assert!(! dto.has_onset());
        assert!(! dto.is_excluded());
    }

    #[rstest]
    fn test_excluded_ctor() {
        let hpo_id = "HP:5200362";
        let hpo_label = "Short NREM sleep";
        let dto = HpoTermDto::excluded(hpo_id, hpo_label);
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
        let dto = HpoTermDto::observed_with_onset(hpo_id, hpo_label, onset);
        assert_eq!(hpo_id, dto.term_id());
        assert_eq!(hpo_label, dto.label());
        assert!(dto.has_onset());
        let hpo_onset = dto.onset().unwrap();
        assert_eq!(onset, hpo_onset);
        assert!(! dto.is_excluded());

    }

}
