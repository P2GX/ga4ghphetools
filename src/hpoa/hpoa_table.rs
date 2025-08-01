use std::{collections::{HashMap, HashSet}};

use chrono::Local;

use crate::{dto::template_dto::{DiseaseDto, TemplateDto}, hpoa::{hpoa_table_row::HpoaTableRow, pmid_counter::PmidCounter}};





pub struct HpoaTable {
    hpoa_row_list: Vec<HpoaTableRow>
}

impl HpoaTable {

    pub fn new(cohort: TemplateDto, biocurator: &str) -> Result<Self, String>{
        let todays_date = Local::now().format("%Y-%m-%d").to_string();
        if ! cohort.is_mendelian() {
            return Err(format!("Can only export Mendelian HPOA table, but this cohort is {:?}", 
                cohort.cohort_type));
        }
        let hpo_header = cohort.hpo_headers;
        let mut pmid_map: HashMap<String, PmidCounter> = HashMap::new();
        let mut disease_set: HashSet<DiseaseDto> = HashSet::new();
        let mut hpoa_rows = Vec::new();
        for row in &cohort.rows {
            if row.disease_dto_list.len() != 1 {
                // should never happen
                return Err("Can only export Mendelian (one disease) HPOA file".to_string());
            }
            let disease_dto = row.disease_dto_list[0].clone();
            disease_set.insert(disease_dto);
            let pmid = &row.individual_dto.pmid;
            let counter = pmid_map
                .entry(pmid.clone())
                .or_insert(PmidCounter::new(pmid));
            // Iterate across HPO terms and add to counter 
            if hpo_header.len() != row.hpo_data.len() {
                return Err(format!("Length mismatch: hpo_header has {}, hpo_data has {}", hpo_header.len(), row.hpo_data.len()));
            }
            for (hpo_item, data_item) in hpo_header.iter().zip(row.hpo_data.iter()) {
                let hpo_duplet = hpo_item.to_hpo_duplet();
                let hpo_id = hpo_duplet.hpo_id();
                let label = hpo_duplet.hpo_label();
                if data_item.value == "na" {
                    continue;
                }
                if data_item.value == "observed" {
                    counter.observed(hpo_id);
                } else if data_item.value == "excluded" {
                    counter.excluded(hpo_id);
                } else {
                    return Err(format!("Unknown HPO cell contents '{}' for HPO '{}'", data_item.value, hpo_id));
                }
            }
        }
        if disease_set.len() != 1 {
            return Err(format!("Expected exactly one disease, found {}", disease_set.len()));
        }
        let disease_dto = disease_set.into_iter().next().unwrap();
        for (pmid, counter) in &pmid_map {
            for hpo_item in &hpo_header {
                let hpo_duplet = hpo_item.to_hpo_duplet();
                if counter.contains(hpo_duplet.hpo_id()) {
                    let freq = counter.get_freq(hpo_duplet.hpo_id())?;
                    let row = HpoaTableRow::new(
                        &disease_dto, 
                        hpo_duplet.hpo_id(), 
                        hpo_duplet.hpo_label(),
                        &freq,
                        &pmid, 
                        biocurator)?;
                    hpoa_rows.push(row);
                }
            }
        }

        Ok(Self{
            hpoa_row_list: hpoa_rows,
        })

    }
}