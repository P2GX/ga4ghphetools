use crate::{dto::{template_dto::{DemographicDto, DiseaseDto, IndividualBundleDto}, validation_errors::ValidationErrors}, header::duplet_item::DupletItem, template::demographic_bundle::DemographicBundle};



#[derive(Clone, Debug)]
pub struct DemographicHeader {
    pub age_of_onset: DupletItem,
    pub age_at_last_encounter: DupletItem,
    pub deceased: DupletItem,
    pub sex: DupletItem
}

impl DemographicHeader {
    pub fn new() -> Self {
        Self { 
            age_of_onset: DupletItem::age_of_onset(), 
            age_at_last_encounter: DupletItem::age_at_last_encounter() ,
            deceased: DupletItem::deceased(),
            sex: DupletItem::sex()
        }
    }

    /// Perform quality control on the labels of the two header rows for a Disease Bundle
     /// We need the start index because for melded phenotypes there are two disease bundles
    pub fn from_matrix(
        matrix: &Vec<Vec<String>>,
        start_idx: usize
    ) -> Result<Self, ValidationErrors> {
        let header = DemographicHeader::new();
        let mut verrors = ValidationErrors::new();
        if matrix.len() < 2 {
            verrors.push_str(format!("Empty template with {} rows.", matrix.len()));
        }
        let mut i = start_idx;
        verrors.push_result(header.age_of_onset.check_column_labels(&matrix, i));
        i += 1;
        verrors.push_result(header.age_at_last_encounter.check_column_labels(&matrix, i));
        i += 1;
        verrors.push_result(header.deceased.check_column_labels(&matrix, i));
        i += 1;
        verrors.push_result(header.sex.check_column_labels(&matrix, i));
        if verrors.has_error() {
            Err(verrors)
        } else {
            Ok(header)
        }
    }

     /// Check a demographic bundle for errors.
    pub fn qc_dto(&self, dto: &DemographicDto) -> Result<(), ValidationErrors> {
        self.qc_data(&dto.age_of_onset, &dto.age_at_last_encounter, &dto.deceased, &dto.sex)
    }

    pub fn qc_bundle(&self, bundle: &DemographicBundle) -> Result<(), ValidationErrors> {
        self.qc_data(&bundle.age_of_onset, &bundle.age_at_last_encounter, &bundle.deceased, &bundle.sex)
    }


     /// Check a demographic bundle for errors.
    pub fn qc_data(&self, age_of_onset: &str, age_at_last_encounter: &str, deceased: &str, sex: &str) -> Result<(), ValidationErrors> {
        let mut verrors = ValidationErrors::new();
        verrors.push_result(self.age_of_onset.qc_data(age_of_onset));
        verrors.push_result(self.age_at_last_encounter.qc_data(age_at_last_encounter));
        verrors.push_result(self.deceased.qc_data(deceased));
        verrors.push_result(self.sex.qc_data(sex));
        if verrors.has_error() {
            Err(verrors)
        } else {
            Ok(())
        }
    }
}
	