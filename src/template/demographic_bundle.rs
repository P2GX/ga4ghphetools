
use std::sync::Arc;
use once_cell::sync::Lazy;

use crate::{dto::validation_errors::ValidationErrors, header::{demographic_header::DemographicHeader, disease_header::DiseaseHeader, individual_header::IndividualHeader}};


static SHARED_HEADER: Lazy<Arc<DemographicHeader>> = Lazy::new(|| {
    Arc::new(DemographicHeader::new())
});


#[derive(Clone, Debug)]
pub struct DemographicBundle {
    header: Arc<DemographicHeader>,
    pub(crate) age_of_onset: String,
    pub(crate) age_at_last_encounter: String,
    pub(crate) deceased: String,
    pub(crate) sex: String
}

impl DemographicBundle {
    pub fn new(age_of_onset: &str,
    age_at_last_encounter: &str,
    deceased: &str,
    sex: &str) -> Self {
        Self { 
            header: SHARED_HEADER.clone(), 
            age_of_onset: age_of_onset.to_string(), 
            age_at_last_encounter: age_at_last_encounter.to_string(), 
            deceased: deceased.to_string(), 
            sex: sex.to_string() 
        }
    }

     // Start index is the index in the template matrix where this block of columns starts
    pub fn from_row(
        row: &Vec<String>,
        start_idx: usize
    ) -> std::result::Result<Self, ValidationErrors> {
        let mut i = start_idx;
        let bundle = Self::new(&row[i], &row[i+1], &row[i+2], &row[i+3]);
        let _ = bundle.do_qc()?;
        Ok(bundle)
    }

    pub fn do_qc(&self) -> Result<(), ValidationErrors> {
        self.header.qc_bundle(self)
    }

    pub fn age_of_onset(&self) -> &str {
        &self.age_of_onset
    }

    pub fn age_at_last_encounter(&self) -> &str {
        &self.age_at_last_encounter
    }

    pub fn deceased(&self) -> &str {
        &self.deceased
    }

    pub fn sex(&self) -> &str {
        &self.sex
    }
    
}