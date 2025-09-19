use std::collections::hash_map::Entry;
use std::{collections::HashMap, fmt, fs, sync::Arc};
use ontolius::ontology::{csr::FullCsrOntology, MetadataAware};

use crate::dto::cohort_dto::DiseaseData;
use crate::dto::etl_dto::{EtlColumnType::{self, *}, EtlDto};
use crate::dto::hpo_term_dto::CellValue;
use crate::{dto::{cohort_dto::{CohortData, CohortType, IndividualData, RowData}, etl_dto::ColumnTableDto, hpo_term_dto::HpoTermDuplet}, hpo};



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
                match col.header.column_type {
                    EtlColumnType::SingleHpoTerm | EtlColumnType::MultipleHpoTerm => {
                        col.header.hpo_terms.as_ref()
                    },
                    _ => None
                }
            })
            .flatten()
            .cloned()
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

    /// We check if there is already an entry for some HPO term in some row. If yes, we throw an
    /// error if the two values disagree.
    fn insert_or_validate(map: &mut HashMap<HpoTermDuplet, String>, key: HpoTermDuplet, value: String) -> Result<(), String> {
            match map.entry(key) {
                Entry::Occupied(entry) => {
                    if entry.get() != &value {
                        return Err(format!(
                            "Conflicting values for HPO term {:?}: existing '{}', new '{}'", 
                            entry.key(), 
                            entry.get(), 
                            value
                        ));
                    }
                },
                Entry::Vacant(entry) => {
                    entry.insert(value);
                }
            }
            Ok(())
        }

    /** We expect to get a cell value with this format:
     * HP:0011968-excluded;HP:0008947-observed
     */
    fn insert_multiple_hpo_column(
        map: &mut HashMap<HpoTermDuplet, String>, 
        duplet_list: &[HpoTermDuplet], 
        value: String) -> Result<(), String>{
        println!("Multiple Cell Value: {}", value);
        let observation_list = value.split(";");
        let mut observation_map: HashMap<String, String> = HashMap::new();
        for obs in observation_list {
            let obs_pair: Vec<&str> = obs.split("-").collect();
            if obs_pair.len() != 2 {
                return Err(format!("Malformed observation pair {obs}"))
            }
            observation_map.insert(obs_pair[0].to_string(), obs_pair[1].to_string());

        }
        for hdup in duplet_list {
        let val = observation_map
            .get(hdup.hpo_id())
            .cloned()
            .unwrap_or_else(|| "na".to_string());
            map.insert(hdup.clone(), val);
        }
        Ok(())
    }


    /** TODO */
    pub fn get_row(&self, i: usize, all_hpo_duplets: &[HpoTermDuplet], disease: &DiseaseData) -> Result<RowData, String> {
       
         let individual = self.get_individual(i)?;
         let mut hpo_to_status_map: HashMap<HpoTermDuplet, String> = HashMap::new();
         let mut allele_count_map: HashMap<String, usize> = HashMap::new();
         for col in &self.dto.table.columns {
            if col.header.column_type == SingleHpoTerm {
                if let Some(hpo_terms) = &col.header.hpo_terms {
                    if hpo_terms.len() != 1 {
                        return Err(format!(
                            "Expected exactly one HPO term in SingleHpoTerm header '{}' but found {}", 
                            col.header.original, 
                            hpo_terms.len()
                        ));
                    };
                    let single_term = &hpo_terms[0]; 
                    Self::insert_or_validate(&mut hpo_to_status_map, single_term.clone(), col.values[i].clone())?;
                } else {
                    return Err("Could not extract HpoTermDuplet from Single HPO column".to_string());
                }
            } else if col.header.column_type == MultipleHpoTerm {
                if let Some(hpo_terms) = &col.header.hpo_terms {
                    Self::insert_multiple_hpo_column(&mut hpo_to_status_map, hpo_terms, col.values[i].clone());
                    
                } else {
                    return Err("Could not extract HpoTermDuplet from Multiple HPO column".to_string());
                }
            } else if col.header.column_type == Variant {
                allele_count_map.entry(col.values[i].clone())
                    .and_modify(|count| *count += 1)
                    .or_insert(1);
            }   
         }
         let mut values: Vec<CellValue> = Vec::new();
         for hpo_duplet in all_hpo_duplets {
            println!("HPO dup - {:?}", hpo_duplet);
            match hpo_to_status_map.get(hpo_duplet) {
                Some(status) => {
                    match status.as_str() {
                        "observed" => { values.push(CellValue::Observed);},
                        "excluded" => { values.push(CellValue::Excluded);},
                        "na" => { values.push(CellValue::Na);},
                        _ => { values.push(CellValue::OnsetAge(status.clone()));}
                    }
                }
                None => {
                    values.push(CellValue::Na);
                }
            }
         }
         let row = RowData{
            individual_data: individual,
            disease_id_list: vec![disease.disease_id.clone()],
            allele_count_map,
            hpo_data: values,
        };
               

        Ok(row)
    }

    pub fn get_row_count(&self) -> Result<usize, String> {
        let first_col = self.dto.table.columns.first()
            .ok_or("No columns in table")?;
        
        let n_rows = first_col.values.len();
        
        if !self.dto.table.columns.iter().all(|col| col.values.len() == n_rows) {
            return Err("Inconsistent column lengths".to_string());
        }
        
        Ok(n_rows)
    }


     /// Note that only Mendelian is supported for Excel file bulk imports
    /// Ohter MOIs are too complicated to be reliably imported in this way.
    pub fn get_dto(&self) -> Result<CohortData, String> {
        let hpo_duplets = Self::all_hpo_duplets(&self);
        let arranged_duplets = hpo::arrange_hpo_duplets(self.hpo.clone(), &hpo_duplets)?;
        let disease = match &self.dto.disease {
            Some(d) => d.clone(),
            None => { return Err(format!("Cannot create CohortData if ETL does not have disease data"))},
        };
        let mut row_list: Vec<RowData> = Vec::new();
        let n_rows = self.get_row_count()?;
        for row_index in 0..n_rows {
            let row = self.get_row(row_index, &arranged_duplets, &disease)?;
            row_list.push(row);
        }
        Ok(CohortData { 
            cohort_type: CohortType::Mendelian, 
            disease_list: vec![disease], 
            hpo_headers: arranged_duplets, 
            rows: row_list, 
            hgvs_variants: self.dto.hgvs_variants.clone(), 
            structural_variants: self.dto.structural_variants.clone(), 
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

