use std::sync::Arc;
use once_cell::sync::Lazy;

use crate::{dto::{template_dto::DiseaseDto, validation_errors::ValidationErrors}, header::{disease_header::DiseaseHeader, individual_header::IndividualHeader}};


static SHARED_HEADER: Lazy<Arc<DiseaseHeader>> = Lazy::new(|| {
    Arc::new(DiseaseHeader::new())
});


#[derive(Clone, Debug)]
pub struct DiseaseBundle {
    header: Arc<DiseaseHeader>,
    pub(crate) disease_id: String,
    pub(crate) disease_label: String,
}


impl DiseaseBundle {
    pub fn new(
        disease_id: &str, 
        disease_label: &str) 
    -> Self {
        Self { 
            header: SHARED_HEADER.clone(), 
            disease_id: disease_id.to_string(), 
            disease_label: disease_label.to_string() 
        }
    }

    // Start index is the index in the template matrix where this block of columns starts
    pub fn from_row(
        row: &Vec<String>,
        start_idx: usize
    ) -> std::result::Result<Self, ValidationErrors> {
        let mut i = start_idx;
        let bundle = Self::new(&row[i], &row[i+1]);
        let _ = bundle.do_qc()?;
        Ok(bundle)
    }

    pub fn do_qc(&self) -> Result<(), ValidationErrors> {
        self.header.qc_bundle(self)
    }

    pub fn to_dto(&self) -> DiseaseDto {
        DiseaseDto::new(&self.disease_id, &self.disease_label)
    }

    pub fn from_dto(dto: DiseaseDto) -> Self {
        Self { header: SHARED_HEADER.clone(), 
            disease_id: dto.disease_id, 
            disease_label: dto.disease_label
        }
    }

    pub fn from_dto_list(dto_list: Vec<DiseaseDto>) -> Vec<Self> {
        dto_list.into_iter()
            .map(|dto| Self::from_dto(dto))
            .collect()
    }

}