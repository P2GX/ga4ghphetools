use core::{result::Result, todo};
use std::sync::Arc;
use once_cell::sync::Lazy;

use crate::{dto::cohort_dto::{CohortData, CohortType, DiseaseData}, header::disease_header::DiseaseHeader};


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
    ) -> std::result::Result<Self, String> {
        let i = start_idx;
        let bundle = Self::new(&row[i], &row[i+1]);
        bundle.do_qc()?;
        Ok(bundle)
    }

    
    pub fn do_qc(&self) -> Result<(), String> {
        self.header.qc_bundle(self)
    }

    pub fn to_dto(&self) -> DiseaseData {
        DiseaseData::new(&self.disease_id, &self.disease_label)
    }

    pub fn from_dto(dto: DiseaseData) -> Self {
        Self { header: SHARED_HEADER.clone(), 
            disease_id: dto.disease_id, 
            disease_label: dto.disease_label
        }
    }


    pub fn from_cohort_dto(cohort_dto: &CohortData) -> Result<Vec<Self>, String> {
        match cohort_dto.template_type() {
            CohortType::Mendelian => {
                let disease_dto_list: Vec<DiseaseData> = cohort_dto.get_disease_dto_list()?;
                let disease_bundle_list = Self::from_dto_list(disease_dto_list);
                Ok(disease_bundle_list)
        },
            CohortType::Melded => todo!(),
            CohortType::Digenic => todo!(),
        }
    }

    pub fn from_dto_list(dto_list: Vec<DiseaseData>) -> Vec<Self> {
        dto_list.into_iter()
            .map(|dto| Self::from_dto(dto))
            .collect()
    }
    /// Create a list of DiseaseBundle objects from a DiseaseGeneData (this is what we expect to get from the frontend)
    pub fn from_disease_gene_dto(dto: DiseaseData) -> Vec<Self> {        
        Self::from_dto_list(vec![dto])
    }

}


#[cfg(test)]
mod test {
    use rstest::{fixture, rstest};
    use crate::factory::disease_bundle::DiseaseBundle;


    #[fixture]
    fn disease_id() -> &'static str {
        return "OMIM:605407";
    }

    #[fixture]
    fn disease_label() -> &'static str {
        return "Segawa syndrome, recessive";
    }

    #[rstest]
    fn test_valid_disease_bundle(
        disease_id: &str,
        disease_label: &str) 
    {
        let db = DiseaseBundle::new(disease_id, disease_label);
        let result = db.do_qc();
        assert!(result.is_ok());
    }


    #[rstest]
    #[case( "MIM:135100", "Disease id has invalid prefix: 'MIM:135100'")]
    #[case( "OMIM: 135100", "Contains stray whitespace: 'OMIM: 135100'")]
    #[case( "OMIM:13510", "OMIM identifiers must have 6 digits: 'OMIM:13510'")]
    fn test_malformed_disease_id(
        disease_id: &str,
        disease_label: &str,
        #[case] entry: &str,
        #[case] expected_error_msg: &str) 
    {
        let db = DiseaseBundle::new(entry, disease_label);
        let result = db.do_qc();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(expected_error_msg, err);
    }

    #[rstest]
    #[case( "Fibrodysplasia ossificans progressiva ", "Trailing whitespace in 'Fibrodysplasia ossificans progressiva '")]
    fn test_malformed_malformed_label(
        disease_id: &str,
        disease_label: &str,
        #[case] entry: &str,
        #[case] expected_error_msg: &str) 
    {
        let db = DiseaseBundle::new(disease_id, entry);
        let result = db.do_qc();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(expected_error_msg, err);
    }


}

