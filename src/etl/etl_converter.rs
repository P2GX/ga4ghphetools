use crate::{dto::{etl_dto::ColumnTableDto, hpo_term_dto::HpoTermDuplet}, etl::etl_tools::EtlTools};






pub struct EtlConverter {
    pub etl_data_frame: ColumnTableDto,
    all_hpo_terms: Vec<HpoTermDuplet>
}


impl EtlConverter {

    pub fn new(etl_file_path: &str) -> Result<Self, String> {
         let table = EtlTools::load_column_table_from_json(etl_file_path)?;
          
          Ok(Self { etl_data_frame: table, all_hpo_terms: vec![] })
          
    }
    
}






#[cfg(test)]
mod test {
    use crate::{dto::cohort_dto::{DiseaseData, GeneTranscriptData}, etl::etl_converter};
    use ontolius::{io::OntologyLoaderBuilder};
  
    use super::*;
    use std::{fs::File, io::BufReader};
    use rstest::{fixture, rstest};



    #[rstest]
    #[ignore = "local file"]
    fn test_conversion() {
        let template_json = "/Users/robin/Desktop/HPOstuff/Phenoboard/NEDFHCA-intermediate.json";
        let result = EtlConverter::new(template_json);
        assert!(result.is_ok());
        let etl_converter = result.unwrap();
        println!("{:?}", etl_converter.etl_data_frame);
    }


}