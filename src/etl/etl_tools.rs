use std::{collections::HashMap, fmt, fs, sync::Arc};

use ontolius::ontology::{csr::FullCsrOntology, MetadataAware};
use phenopackets::schema::v1::core::Sex;
use crate::dto::cohort_dto::DiseaseData;
use crate::dto::etl_dto::{EtlColumnType::*, EtlDto};
use crate::{dto::{cohort_dto::{CohortData, CohortType, IndividualData, RowData}, etl_dto::{ColumnMetadata, ColumnTableDto}, hpo_term_dto::HpoTermDuplet}, factory::excel, hpo};



pub struct EtlTools {
    /// Reference to the Ontolius Human Phenotype Ontology Full CSR object
    hpo: Arc<FullCsrOntology>,
    /// The data that has been extracted and transformed in a front end
    dto: EtlDto,
}


impl EtlTools {


    pub fn from_dto(
        hpo: Arc<FullCsrOntology>, 
        dto: &EtlDto,
    ) -> Self {
        Self{
            hpo,
            dto: dto.clone(),
        }
    }

    pub fn from_json(
        etl_file_path: &str,
        hpo: Arc<FullCsrOntology>,
    ) -> Result<Self, String> {
        let dto = EtlTools::load_etl_dto_from_json(etl_file_path)?;
        Ok(
            Self {
                hpo,
                dto
            }
        ) 
    }

    pub fn raw_table(&self) -> &EtlDto {
        &self.dto
    }

    // Function to load JSON file and deserialize to ColumnTableDto
    pub fn load_etl_dto_from_json(file_path: &str) -> Result<EtlDto, String> {
        let json_content = fs::read_to_string(file_path)
            .map_err(|e| e.to_string())?;
        let etl_dto: EtlDto = serde_json::from_str(&json_content)
            .map_err(|e| e.to_string())?;
        Ok(etl_dto)
    }



    
    /// Retrieve all HPO Duplets from the Single and Multiple HPO columns
    /// We need this to know how many HPO terms we have altogether for the CohortData
    pub fn all_hpo_duplets(&self) -> Vec<HpoTermDuplet> {
        self.dto.table.columns.iter()
            .filter_map(|col| {
                if let ColumnMetadata::HpoTerms(duplets) = &col.header.metadata {
                    Some(duplets.clone())
                } else {
                    None
                }
            })
            .flatten()
            .collect()
    }

    /// Extract the string value of of table cell
    fn extract_value(values: &[String], i: usize, field: &str) -> Result<String, String> {
        values.get(i)
            .map(|v| v.to_string())
            .ok_or_else(|| format!("Could not extract {} from column", field))
    }

    /// Get the individual Data for row i
    fn get_individual(&self, i: usize) -> Result<IndividualData, String> {
         let pmid = self.dto.pmid.clone().ok_or_else(|| format!("Could not extract pmid for individual {i}"))?;
         let title: String = self.dto.title.clone().ok_or_else(|| format!("Could not extract title for individual {i}"))?;
         let mut individual = IndividualData{ 
            pmid: pmid, 
            title: title, 
            individual_id: String::default(), 
            comment: String::default(), 
            age_of_onset: "na".to_string(), 
            age_at_last_encounter: "na".to_string(), 
            deceased: "na".to_string(), 
            sex: "na".to_string(), 
         };
   
         for col in &self.dto.table.columns {
            match &col.header.column_type {
                Raw | FamilyId | SingleHpoTerm | MultipleHpoTerm |
                GeneSymbol | Variant | Ignore => {}
                PatientId => {
                    individual.individual_id = Self::extract_value(&col.values, i, "individual ID")?;
                }
                AgeOfOnset => {
                    individual.age_of_onset = Self::extract_value(&col.values, i, "age_of_onset")?;
                }
                AgeAtLastEncounter => {
                    individual.age_at_last_encounter = Self::extract_value(&col.values, i, "age_at_last_encounter")?;
                }
                Sex => {
                    individual.sex = Self::extract_value(&col.values, i, "sex")?;
                }
                Deceased => {
                    individual.deceased = Self::extract_value(&col.values, i, "deceased")?;
                }
            }
        }
        if individual.individual_id.len() < 1 {
            return Err(format!("Invalid individual without identifier: {:?}", individual));
        }
        Ok(individual)
    }


    /** TODO */
    pub fn get_row(&self, i: usize) -> Result<RowData, String> {
       
         let individual = self.get_individual(i)?;

         let row = RowData{
            individual_data: individual,
            disease_id_list: todo!(),
            allele_count_map: todo!(),
            hpo_data: todo!(),
        };
               

        todo!()
    }


     /// Note that only Mendelian is supported for Excel file bulk imports
    /// Ohter MOIs are too complicated to be reliably imported in this way.
    pub fn get_dto(&self) -> Result<CohortData, String> {
        let hpo_duplets = Self::all_hpo_duplets(&self);
        let header = hpo::arrange_hpo_duplets(self.hpo.clone(), &hpo_duplets)?;
        Ok(CohortData { 
            cohort_type: CohortType::Mendelian, 
            disease_list: vec![], 
            hpo_headers: header, 
            rows: vec![], 
            hgvs_variants: HashMap::new(), 
            structural_variants: HashMap::new(), 
            phetools_schema_version: CohortData::phenopackets_schema_version(), 
            hpo_version: self.hpo.version().to_string(), 
            cohort_acronym: None 
        })
    }
  
}




impl fmt::Display for ColumnTableDto {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "File: {}", self.file_name)?;
        writeln!(f, "Columns:")?;

        for column in &self.columns {
            let first_value = column.values.first().cloned().unwrap_or_else(|| "<empty>".to_string());
            writeln!(f, "- {}: {}", column.header.original, first_value)?;
        }

        Ok(())
    }
}

