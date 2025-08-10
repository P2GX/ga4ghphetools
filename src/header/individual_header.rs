use crate::{dto::{cohort_dto::IndividualBundleDto, validation_errors::ValidationErrors}, header::duplet_item::DupletItem, template::individual_bundle::IndividualBundle};



#[derive(Clone, Debug)]
pub struct IndividualHeader {
    pub pmid: DupletItem,
    pub title: DupletItem,
    pub individual_id: DupletItem,
    pub comment: DupletItem,
    pub age_of_onset: DupletItem,
    pub age_at_last_encounter: DupletItem,
    pub deceased: DupletItem,
    pub sex: DupletItem
}


impl IndividualHeader {
    pub fn new() -> Self {
        Self { 
            pmid: DupletItem::pmid(),
            title: DupletItem::title(), 
            individual_id: DupletItem::individual_id(), 
            comment: DupletItem::comment(),
            age_of_onset: DupletItem::age_of_onset(), 
            age_at_last_encounter: DupletItem::age_at_last_encounter() ,
            deceased: DupletItem::deceased(),
            sex: DupletItem::sex()
        }
    }

    /// Perform quality control on the labels of the two header rows for the IndividualBundle.
    pub fn from_matrix(
        matrix: &Vec<Vec<String>>,
        demographics_start_idx: usize
    ) -> Result<Self, ValidationErrors> {
        let mut verrors = ValidationErrors::new();
        let iheader = IndividualHeader::new();
        if matrix.len() < 2 {
            verrors.push_str(format!("Empty template with {} rows.", matrix.len()));
        }
        verrors.push_result(iheader.pmid.check_column_labels(&matrix, 0));
        verrors.push_result(iheader.title.check_column_labels(&matrix, 1));
        verrors.push_result(iheader.individual_id.check_column_labels(&matrix, 2));
        verrors.push_result(iheader.comment.check_column_labels(&matrix, 3));
        let mut i = demographics_start_idx;
        verrors.push_result(iheader.age_of_onset.check_column_labels(&matrix, i));
        i += 1;
        verrors.push_result(iheader.age_at_last_encounter.check_column_labels(&matrix, i));
        i += 1;
        verrors.push_result(iheader.deceased.check_column_labels(&matrix, i));
        i += 1;
        verrors.push_result(iheader.sex.check_column_labels(&matrix, i));
        if verrors.has_error() {
            Err(verrors)
        } else {
            Ok(iheader)
        }
    }

    /// Check an individual bundle for errors.
    pub fn qc_dto(&self, dto: IndividualBundleDto) -> Result<(), ValidationErrors> {
        self.qc_data(&dto.pmid, &dto.title, &dto.individual_id, &dto.comment, &dto.age_of_onset, &dto.age_at_last_encounter, &dto.deceased, &dto.sex)
    }

     /// Check an individual bundle for errors.
    pub fn qc_bundle(&self, bundle: &IndividualBundle) -> Result<(), ValidationErrors> {
        self.qc_data(&bundle.pmid, &bundle.title, &bundle.individual_id, &bundle.comment, &bundle.age_of_onset, &bundle.age_at_last_encounter, &bundle.deceased, &bundle.sex)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn qc_data(&self, 
        pmid: &str, 
        title: &str, 
        individual_id: &str, 
        comment: &str,
        age_of_onset: &str, 
        age_at_last_encounter: &str, 
        deceased: &str, 
        sex: &str) 
    -> Result<(), ValidationErrors> {
        let mut verrors = ValidationErrors::new();
        verrors.push_result(self.pmid.qc_data(pmid));
        verrors.push_result(self.title.qc_data(title));
        verrors.push_result(self.individual_id.qc_data(individual_id));
        verrors.push_result(self.comment.qc_data(comment));
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