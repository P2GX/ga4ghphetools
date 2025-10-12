use std::{collections::{HashMap, HashSet}, sync::Arc};

use ontolius::ontology::csr::FullCsrOntology;
use rand::rand_core::impls;
use serde::{Deserialize,Serialize};

use crate::dto::{cohort_dto::{CohortData, DiseaseData}, hpo_term_dto::HpoTermDuplet};


#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum RenderCellType {
    IndividualData,
    Pmid,
    Observed,
    Excluded,
    OnsetAge,
    Na,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderCell {
    pub cell_contents: String,
    pub cell_type: RenderCellType
}

impl RenderCell {
    pub fn new_pmid(pmid: String) -> Self {
        Self { cell_contents: pmid, cell_type: RenderCellType::Pmid }
    }
    pub fn new_individual(id: String) -> Self {
        Self { cell_contents: id, cell_type: RenderCellType::IndividualData }
    }
    pub fn observed() -> Self {
        Self { cell_contents: "observed".to_string(), cell_type: RenderCellType::Observed }
    }
    pub fn excluded() -> Self {
        Self { cell_contents: "excluded".to_string(), cell_type: RenderCellType::Excluded }
    }
    pub fn onset(onset: String) -> Self {
        Self { cell_contents: onset, cell_type: RenderCellType::OnsetAge }
    }
    pub fn na() -> Self {
        Self { cell_contents: "na".to_string(), cell_type: RenderCellType::Na }
    }
}



/// Data class for rendering HTML
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TopLevelHpoRenderer {
    pub top_level_name: String,
    pub hpo_header: Vec<HpoTermDuplet>,
    pub rows: Vec<Vec<RenderCell>>
}


impl TopLevelHpoRenderer  {
    pub fn new(
        top_level_name: &str,
        hpo_header: &Vec<HpoTermDuplet>,
        rows: Vec<Vec<RenderCell>>) -> Self {
        Self { 
            top_level_name: top_level_name.to_string(), 
            hpo_header: hpo_header.clone(), 
            rows: rows 
        }
    }
}



/// Data class for rendering HTML
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CohortRenderer {
    pub acronym: String,
    pub hpo_version: String,
    pub phetools_schema_version: String, 
    pub cohort_type: String,
    pub n_phenopackets: usize,
    pub n_distinct_hpo_terms: usize,
    pub disease_list: Vec<DiseaseData>,
    pub top_level_list: Vec<TopLevelHpoRenderer>,
}





impl CohortRenderer {
    pub fn new(
        cohort: &CohortData,
        hpo: Arc<FullCsrOntology>
    ) -> Result<Self, String> {
        let acronym = match &cohort.cohort_acronym {
            Some(acronym) => acronym.to_string(),
            None => "n/a".to_string(),
        };
        // This should actually always work, but if not, just create an empty map
        let top_level_map = match crate::hpo::get_hpo_terms_by_toplevel(cohort.clone(), hpo.clone()){
            Ok(tlmap) => tlmap,
            Err(e) => {
                eprint!("{}", e.to_string());
                HashMap::<String, Vec<HpoTermDuplet>>::new()
            }
        };
        let mut top_level_list: Vec<TopLevelHpoRenderer> = Vec::new();
        for top in top_level_map {
            let top_level = Self::get_top_level_section(&top.0, top.1, &cohort)
                .map_err(|e|e.to_string())?;
            top_level_list.push(top_level);
        }

        
    
        Ok(Self {  
            acronym: acronym,
            hpo_version: cohort.hpo_version.to_string(),
            phetools_schema_version: cohort.phetools_schema_version.to_string(),
            cohort_type: cohort.cohort_type.to_string(),
            n_phenopackets: cohort.rows.len(),
            n_distinct_hpo_terms: cohort.hpo_headers.len(),
            disease_list: cohort.disease_list.clone(),
            top_level_list
        })

    }


    fn get_top_level_section(
        top_level: &str,
        duplets: Vec<HpoTermDuplet>,
        cohort: &CohortData
    ) -> Result<TopLevelHpoRenderer, String> {
        let duplet_set: HashSet<HpoTermDuplet> = duplets.clone().into_iter().collect();
        let mut rows: Vec<Vec<RenderCell>> = Vec::new();
        for row in &cohort.rows {
            let mut data_row: Vec<RenderCell> = Vec::new();
            let pmid = row.individual_data.pmid.clone();
            let indi_id = row.individual_data.individual_id.clone();
            data_row.push(RenderCell::new_pmid(pmid));
            data_row.push(RenderCell::new_individual(indi_id));
            for (duplet, cell) in cohort.hpo_headers.iter().zip(&row.hpo_data) {
                if duplet_set.contains(duplet) {
                    match &cell {
                        crate::dto::hpo_term_dto::CellValue::Observed => data_row.push(RenderCell::observed()),
                        crate::dto::hpo_term_dto::CellValue::Excluded => data_row.push(RenderCell::excluded()),
                        crate::dto::hpo_term_dto::CellValue::Na => data_row.push(RenderCell::na()),
                        crate::dto::hpo_term_dto::CellValue::OnsetAge(onset) => data_row.push(RenderCell::onset(onset.to_string())),
                        crate::dto::hpo_term_dto::CellValue::Modifier(_) => data_row.push(RenderCell::observed()),
                    }
                }
            }
            rows.push(data_row);
        }
        let top = TopLevelHpoRenderer{
            top_level_name: top_level.to_string(),
            hpo_header: duplets.clone(),
            rows,
        };
        Ok(top)
    }


}