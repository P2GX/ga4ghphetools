use std::collections::HashMap;

use phenopackets::schema::v2::Phenopacket;

use crate::{dto::cohort_dto::{CohortData, CohortType}, repo::{disease_qc::DiseaseQc, qc_report::QcReport}};




pub struct CohortQc {
    cohort_name: String,
    disease_qc_list: Vec<DiseaseQc>,
    non_mendelian_cohorts: Vec<CohortData>,
    unexpected_files: Vec<String>
}


impl CohortQc {
    pub fn new (
        cohort_name: &str, 
        cohort_list: Vec<CohortData>,
        phenopackets: Vec<Phenopacket>,
        unexpected_files: Vec<String>) -> Result<Self, String> {
        let mut non_mendelian_cohorts: Vec<CohortData> = Vec::new();
        let mut disease_to_ppkt_d: HashMap<String, DiseaseQc> = HashMap::new();
        for cohort in cohort_list {
            if cohort.cohort_type != CohortType::Mendelian {
                non_mendelian_cohorts.push(cohort);
            } else {
                // by design, Mendelian can only have one disease
                let ddata = &cohort.disease_list[0];
                let dqc = DiseaseQc::new(ddata, &cohort);
                disease_to_ppkt_d.insert(ddata.disease_id.clone(), dqc);
            }
        }
        for ppkt in phenopackets {
            let disease_id = Self::get_disease_id(&ppkt)?;
            match disease_to_ppkt_d.get_mut(&disease_id) {
                Some(dqc) => {
                    dqc.add_ppkt(ppkt);
                },
                None => {
                    for (k,v) in disease_to_ppkt_d.iter() {
                        println!("Could not find disease for id {}", disease_id);
                    }
                }
            }
        }
        let disease_qc_list: Vec<DiseaseQc> = disease_to_ppkt_d.into_values().collect();

        Ok(Self {
            cohort_name: cohort_name.to_string(),
            disease_qc_list,
            non_mendelian_cohorts,
            unexpected_files
        })
    }


    pub fn get_errors(&self) -> Vec<QcReport> {
        let mut errs: Vec<QcReport> = self
            .unexpected_files
            .iter()
            .map(|file| QcReport::unexpected_file(&self.cohort_name, file))
            .collect();

        errs.extend(self.check_moi());

        errs.extend(
            self.disease_qc_list
                .iter()
                .filter_map(|dqc| dqc.check_all_rows_output_as_ppkt()),
        );

        errs
    }

   
    fn get_disease_id(ppkt: &Phenopacket) -> Result<String, String> {
        if ppkt.diseases.len() != 1 {
            return Err(format!("Unexpected disease count {}", ppkt.diseases.len()))
        }
        match &ppkt.diseases[0].term {
            Some(ot) => Ok(ot.id.clone()),
            None => Err(format!("No ontology term for disease {:?}", ppkt.diseases[0])),
        }
    }

    

    pub fn ppkt_count(&self) -> usize {
        return self.disease_qc_list.iter().map(|dqc|dqc.phenopacket_count()).sum();
    }

    pub fn check_moi(&self) -> Vec<QcReport> {
        self.disease_qc_list.iter()
            .flat_map(|dqc|dqc.check_moi())
            .collect()
    }

    



}