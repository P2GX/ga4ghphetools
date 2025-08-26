use crate::{dto::cohort_dto::DiseaseData, header::duplet_item::DupletItem, factory::disease_bundle::DiseaseBundle};



#[derive(Clone, Debug)]
pub struct DiseaseHeader {
    pub disease_id: DupletItem,
    pub disease_label: DupletItem,
}


impl DiseaseHeader {
    pub fn new() -> Self {
        Self { 
            disease_id: DupletItem::disease_id(), 
            disease_label: DupletItem::disease_label() 
        }
    }

     /// Perform quality control on the labels of the two header rows for a Disease Bundle
     /// We need the start index because for melded phenotypes there are two disease bundles
    pub fn from_matrix(
        matrix: &Vec<Vec<String>>,
        start_idx: usize
    ) -> Result<Self, String> {
        let header = DiseaseHeader::new();
        if matrix.len() < 2 {
            return Err(format!("Empty template with {} rows.", matrix.len()));
        }
        let mut i = start_idx;
        header.disease_id.check_column_labels(&matrix, i)?;
        i += 1;
        header.disease_label.check_column_labels(&matrix, i)?;
        Ok(header)
    }

    /// Check an disease bundle for errors.
    pub fn qc_dto(&self, dto: &DiseaseData) -> Result<(), String> {
        self.qc_data(&dto.disease_id, &dto.disease_label)
    }


    /// Check an disease bundle for errors.
    pub fn qc_bundle(&self, bundle: &DiseaseBundle) -> Result<(), String> {
        self.qc_data(&bundle.disease_id, &bundle.disease_label)
    }


    /// Check an disease bundle for errors.
    pub fn qc_data(&self, disease_id: &str, disease_label: &str) -> Result<(), String> {
        self.disease_id.qc_data(disease_id)?;
        self.disease_label.qc_data(disease_label)?;
        Ok(())
    }
}
