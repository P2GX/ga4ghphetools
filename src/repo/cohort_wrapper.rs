//! CohortWrapper
//! Provides all of the information that we need to extract a cohort and its corresponding phenopackets
//! Note that a CohortDir may contain multiple cohorts but only has one phenopacket directory
//! We need to disemcombobulate this prior to processing, and it is convenient to do so with this struct

use std::{collections::HashMap, path::{Path, PathBuf}};
use phenopackets::schema::v2::Phenopacket;

use crate::{cohort_qc::cohort_dir::CohortDir, dto::cohort_dto::CohortData, error::PheToolsError, ppkt, repo::ppkt_wrapper::PpktWrapper};



#[derive(Clone, Debug)]
pub(crate) struct CohortWrapper {
    disease_id: String,
    cohort: CohortData,
    cohort_path: PathBuf,
    ppkt_map: HashMap<PathBuf, Phenopacket>,
}


impl CohortWrapper {
    pub fn new(disease_id: String,
        cohort: CohortData,
        cohort_path: PathBuf,
        ppkt_wrappers: &[PpktWrapper]) -> Self {
            let mut ppkt_map: HashMap<PathBuf, Phenopacket> = HashMap::new();
            for pw in ppkt_wrappers {
                if pw.disease_id == disease_id {
                    ppkt_map.insert(pw.path.clone(), pw.ppkt.clone());
                }
            }
            Self { 
                disease_id, 
                cohort, 
                cohort_path, 
                ppkt_map 
            }
        }
    
    pub fn disease_id(&self) -> &str {
        &self.disease_id
    }

    pub fn cohort_data(&self) -> &CohortData {
        &self.cohort
    }

    pub fn cohort_path(&self) -> &Path {
        &self.cohort_path
    }

    pub fn ppkt_map(&self) -> &HashMap<PathBuf, Phenopacket> {
        &self.ppkt_map
    }

     pub fn get_cohort_wrapper_list(individual_file_list: &[PathBuf]) -> Result<Vec<CohortWrapper>, PheToolsError> {
        let mut  ppkt_w_list = Vec::new();
        for path in individual_file_list.iter() {
            let ppkt = match ppkt::load_phenopacket(path) {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("Warning: Failed to load phenopacket at {:?}: {}", path, e);
                    continue; // Skip this file and move to the next
                }
            };

            let ppkt_disease_id = match ppkt::get_disease_id(&ppkt) {
                Ok(id) => id,
                Err(e) => {
                    eprintln!("Warning: Failed to get disease ID for {:?}: {}", path, e);
                    continue;
                }
            };
            let ppkt_wrap = PpktWrapper::new(path, ppkt_disease_id, ppkt);
            ppkt_w_list.push(ppkt_wrap);
        }
        let mut cohort_wrappers: Vec<CohortWrapper> = Vec::new();
        for pth in individual_file_list {
            let cohort = CohortDir::read_cohort(pth)?;
            let disease_id = cohort.get_mendelian_disease_id()?;
            let filtered_ppkt: Vec<PpktWrapper> = ppkt_w_list.iter()
                .filter(|w| w.disease_id == disease_id)
                .cloned()
                .collect();
            let wrapper = CohortWrapper::new(disease_id, cohort, pth.clone(), &filtered_ppkt);
            cohort_wrappers.push(wrapper);
        }
        Ok(cohort_wrappers)

    }

}