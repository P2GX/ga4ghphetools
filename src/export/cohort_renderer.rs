use std::{collections::{HashMap, HashSet}, sync::Arc};

use ontolius::ontology::csr::FullCsrOntology;
use serde::{Deserialize,Serialize};

use crate::dto::{cohort_dto::{CohortData, DiseaseData, RowData}, hpo_term_dto::HpoTermDuplet};


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
        let pm_number: &str = pmid.strip_prefix("PMID:")
            .map(|x| x.trim()) // remove extra whitespace
            .filter(|x| x.chars().all(|c| c.is_ascii_digit())) // ensure it's all digits
            .unwrap_or(&pmid);

    
        Self { cell_contents: pm_number.to_string(), cell_type: RenderCellType::Pmid }
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


#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CohortRow {
    pub individual_id: String,
    pub pmid: String,
    pub title: String,
    pub values: Vec<RenderCell>
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IndividualRow {
    pub individual_id: String,
    pub pmid: String,
    pub title: String,
    pub onset_age: String,
    pub last_encounter: String,
    pub sex: String,
    pub deceased: String,
    pub alleles: Vec<String>,
}

/// Data class for rendering HTML
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TopLevelHpoRenderer {
    pub top_level_name: String,
    pub anchor: String,
    pub hpo_header: Vec<HpoTermDuplet>,
    pub n_hpo: usize,
    pub rows: Vec<CohortRow>
}


impl TopLevelHpoRenderer  {
    pub fn new(
        top_level_name: &str,
        hpo_header: &[HpoTermDuplet],
        rows: Vec<CohortRow>) -> Self {
        Self { 
            top_level_name: top_level_name.to_string(), 
            anchor: Self::make_anchor_id(top_level_name),
            hpo_header: hpo_header.to_vec(), 
            n_hpo: hpo_header.len(),
            rows: rows 
        }
    }

    fn make_anchor_id(name: &str) -> String {
        name.replace(' ', "_")
            .replace(['/', ',', '(', ')'], "")
            .to_lowercase()
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
    pub individuals: Vec<IndividualRow>
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
                eprint!("{}", e);
                HashMap::<String, Vec<HpoTermDuplet>>::new()
            }
        };
        let mut top_level_list: Vec<TopLevelHpoRenderer> = Vec::new();
        for top in top_level_map {
            let top_level = Self::get_top_level_section(&top.0, top.1, cohort)
                .map_err(|e|e.to_string())?;
            top_level_list.push(top_level);
        }
        let individuals = Self::get_individuals(cohort);
        
    
        Ok(Self {  
            acronym: acronym,
            hpo_version: cohort.hpo_version.to_string(),
            phetools_schema_version: cohort.phetools_schema_version.to_string(),
            cohort_type: cohort.cohort_type.to_string(),
            n_phenopackets: cohort.rows.len(),
            n_distinct_hpo_terms: cohort.hpo_headers.len(),
            disease_list: cohort.disease_list.clone(),
            top_level_list,
            individuals
        })
    }


    fn get_individuals(cohort: &CohortData) -> Vec<IndividualRow> {
        let mut individuals = Vec::new();
        for row in &cohort.rows {
            let pmid = row.individual_data.pmid.clone();
            let pmid = pmid.strip_prefix("PMID:")
                .map(|x| x.trim()) // remove extra whitespace
                .filter(|x| x.chars().all(|c| c.is_ascii_digit())) // ensure it's all digits
                .unwrap_or(&pmid);
            let title = row.individual_data.title.clone();
            let onset_age = row.individual_data.age_of_onset.clone();
            let last_encounter = row.individual_data.age_at_last_encounter.clone();
            let deceased = row.individual_data.deceased.clone();
            let sex = row.individual_data.sex.clone();
            let indi_id = row.individual_data.individual_id.clone();
            let alleles = Self::get_alleles(row, cohort);
            let indi = IndividualRow{
                individual_id: indi_id,
                pmid: pmid.to_string(),
                title,
                onset_age,
                last_encounter,
                sex,
                deceased,
                alleles,
            };
            individuals.push(indi);
        }
        individuals

    }


    fn get_alleles(row: &RowData, cohort: &CohortData) -> Vec<String> {
        row.allele_count_map
            .iter()
            .filter_map(|(allele, &count)| {
                if let Some(hgvs) = cohort.hgvs_variants.get(allele) {
                    let transcript = hgvs.transcript();
                    let symbol = hgvs.symbol();
                    let hgvs_p = match hgvs.p_hgvs() {
                        Some(phgvs) => phgvs,
                        None => "n/a".to_string()
                    };
                    let allele_string = format!(
                        "{}({}):{}; {}: {}",
                        transcript, symbol, hgvs.hgvs(), hgvs_p, count
                    );
                    Some(allele_string)
                } else if let Some(sv) = cohort.structural_variants.get(allele) {
                    let svtext = sv.label();
                    let symbol = sv.gene_symbol();
                    let svtype = sv.get_sequence_ontology_term().label;
                    let allele_string = format!(
                        "{} ({}), {}: {}",
                        svtext, symbol, svtype, count
                    );
                    Some(allele_string)
                } else {
                    None
                }
            })
        .collect()
    }



    fn get_top_level_section(
        top_level: &str,
        duplets: Vec<HpoTermDuplet>,
        cohort: &CohortData
    ) -> Result<TopLevelHpoRenderer, String> {
        let duplet_set: HashSet<HpoTermDuplet> = duplets.clone().into_iter().collect();
        let mut rows: Vec<CohortRow> = Vec::new();
        for row in &cohort.rows {
            let mut data_row: Vec<RenderCell> = Vec::new();
            let pmid = row.individual_data.pmid.clone();
            let pmid = pmid.strip_prefix("PMID:")
                .map(|x| x.trim()) // remove extra whitespace
                .filter(|x| x.chars().all(|c| c.is_ascii_digit())) // ensure it's all digits
                .unwrap_or(&pmid);
            let title = row.individual_data.title.clone();
            let indi_id = row.individual_data.individual_id.clone();
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
            let row = CohortRow{
                individual_id: indi_id,
                pmid: pmid.to_string(),
                title,
                values: data_row,
            };
            rows.push(row);
        }
        let top = TopLevelHpoRenderer{
            top_level_name: top_level.to_string(),
            anchor: TopLevelHpoRenderer::make_anchor_id(top_level),
            hpo_header: duplets.clone(),
            n_hpo: duplets.len(),
            rows,
        };
        Ok(top)
    }


}