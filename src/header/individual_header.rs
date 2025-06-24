use crate::{dto::{template_dto::IndividualBundleDto, validation_errors::ValidationErrors}, header::duplet_item::DupletItem, template::individual_bundle::IndividualBundle};



#[derive(Clone, Debug)]
pub struct IndividualHeader {
    pub pmid: DupletItem,
    pub title: DupletItem,
    pub individual_id: DupletItem,
    pub comment: DupletItem,
}


impl IndividualHeader {
    pub fn new() -> Self {
        Self { 
            pmid: DupletItem::pmid(),
            title: DupletItem::title(), 
            individual_id: DupletItem::individual_id(), 
            comment: DupletItem::comment() 
        }
    }

    /// Perform quality control on the labels of the two header rows for the IndividualBundle.
    pub fn from_matrix(
        matrix: &Vec<Vec<String>>
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
        if verrors.has_error() {
            Err(verrors)
        } else {
            Ok(iheader)
        }
    }

    /// Check an individual bundle for errors.
    pub fn qc_dto(&self, dto: IndividualBundleDto) -> Result<(), ValidationErrors> {
        self.qc_data(&dto.pmid, &dto.title, &dto.individual_id, &dto.comment)
    }

     /// Check an individual bundle for errors.
    pub fn qc_bundle(&self, bundle: &IndividualBundle) -> Result<(), ValidationErrors> {
        self.qc_data(&bundle.pmid, &bundle.title, &bundle.individual_id, &bundle.comment)
    }


    pub fn qc_data(&self, pmid: &str, title: &str, individual_id: &str, comment: &str) -> Result<(), ValidationErrors> {
        let mut verrors = ValidationErrors::new();
        verrors.push_result(self.pmid.qc_data(pmid));
        verrors.push_result(self.title.qc_data(title));
        verrors.push_result(self.individual_id.qc_data(individual_id));
        verrors.push_result(self.comment.qc_data(comment));
        if verrors.has_error() {
            Err(verrors)
        } else {
            Ok(())
        }
    }

}