use std::sync::Arc;
use once_cell::sync::Lazy;

use crate::{dto::validation_errors::ValidationErrors, header::individual_header::IndividualHeader};


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
}

impl IndividualBundle {
    pub fn new(
        pmid: &str, 
        title: &str,
        individual_id: &str,
        comment: &str) 
    -> Self {
        Self { 
            header: SHARED_HEADER.clone(), 
            pmid: pmid.to_string(), 
            title: title.to_string(), 
            individual_id: individual_id.to_string(), 
            comment: comment.to_string() 
        }
    }

    pub fn from_row(
        row: &Vec<String>
    ) -> std::result::Result<Self, ValidationErrors> {
        let bundle = Self::new(&row[0], &row[1], &row[2], &row[3]);
        let _ = bundle.do_qc()?;
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


}