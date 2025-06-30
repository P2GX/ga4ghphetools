use std::sync::Arc;
use once_cell::sync::Lazy;

use crate::{dto::{template_dto::IndividualBundleDto, validation_errors::ValidationErrors}, header::individual_header::IndividualHeader};


static SHARED_HEADER: Lazy<Arc<IndividualHeader>> = Lazy::new(|| {
    Arc::new(IndividualHeader::new())
});

#[derive(Clone, Debug)]
pub struct IndividualBundle {
    header: Arc<IndividualHeader>,
    pub(crate) pmid: String,
    pub(crate) title: String,
    pub(crate) individual_id: String,
    pub(crate) comment: String,
    pub(crate) age_of_onset: String,
    pub(crate) age_at_last_encounter: String,
    pub(crate) deceased: String,
    pub(crate) sex: String
}

impl IndividualBundle {
    pub fn new(
        pmid: &str, 
        title: &str,
        individual_id: &str,
        comment: &str,
        age_of_onset: &str,
        age_at_last_encounter: &str,
        deceased: &str,
        sex: &str) 
    -> Self {
        Self { 
            header: SHARED_HEADER.clone(), 
            pmid: pmid.to_string(), 
            title: title.to_string(), 
            individual_id: individual_id.to_string(), 
            comment: comment.to_string(),
            age_of_onset: age_of_onset.to_string(),
            age_at_last_encounter: age_at_last_encounter.to_string(),
            deceased: deceased.to_string(),
            sex: sex.to_string()
        }
    }

    /// Start idx is the index of the first demographic entry.
    /// We should consider changing the format to put the demographics right after individual.
    pub fn from_row(
        row: &Vec<String>,
        start_idx: usize
    ) -> std::result::Result<Self, ValidationErrors> {
        let  i = start_idx;
        let bundle = Self::new(&row[0], &row[1], &row[2], &row[3], &row[i], &row[i+1], &row[i+2], &row[i+3]);
        println!("from row - {:?}", bundle);
        bundle.do_qc()?;
        Ok(bundle)
    }

    pub fn do_qc(&self) -> Result<(), ValidationErrors> {
        self.header.qc_bundle(self)
    }

    pub fn pmid(&self) -> &str {
        &self.pmid
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn individual_id(&self) -> &str {
        &self.individual_id
    }

    pub fn comment(&self) -> &str {
        &self.comment
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

    pub fn from_dto(dto: IndividualBundleDto) -> Self {
        Self { 
            header: SHARED_HEADER.clone(), 
            pmid: dto.pmid, 
            title: dto.title, 
            individual_id: dto.individual_id, 
            comment: dto.comment, 
            age_of_onset: dto.age_of_onset, 
            age_at_last_encounter: dto.age_at_last_encounter, 
            deceased: dto.deceased, 
            sex: dto.sex 
        }
    }

}