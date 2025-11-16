use std::collections::HashMap;

use crate::{age, dto::{cohort_dto::CohortData, hpo_term_dto::HpoTermDuplet}, hpoa::counted_hpo_term::CountedHpoTerm};


/// structure to get counts of HPO Onset terms per PMID.
/// automatically transforms IsoAge and GestationalAge strings into HPO onset terms.
pub struct HpoaOnsetCalculator {
 
}

impl HpoaOnsetCalculator {
    


    /// Extract a list of CountedHpoTerm objects for the ages of onset for each PMID.
    pub fn pmid_to_onset_freq_d(
        cohort_dto: &CohortData
    ) 
    -> Result<Vec<CountedHpoTerm>, String> {
        let mut counted_term_list: Vec<CountedHpoTerm> = Vec::new();
        let mut pmid_to_onset_string_d : HashMap<String, Vec<HpoTermDuplet>> = HashMap::new();
        for row in &cohort_dto.rows {
            let pmid = row.individual_data.pmid.clone();
            let onset = row.individual_data.age_of_onset.clone();
            if onset != "na" {
                if ! age::is_valid_age_string(&onset) {
                    return Err(format!("Invalid age string '{}' for '{}'", onset, row.individual_data.individual_id));
                }
                let onset_term = age::get_onset_term(&onset)?;
                let onset_list = pmid_to_onset_string_d.entry(pmid).or_insert(Vec::new());
                onset_list.push(onset_term);
            }
        }
        // When we get here, we have a list of strings for each PMID. For the HPOA output, we want to represent them as
        // HPO onset terms with the correct frequencies.
        for (pmid, onset_list) in pmid_to_onset_string_d {
            let n_onset_observations = onset_list.len();
             let mut counts_map: HashMap<HpoTermDuplet, usize> = HashMap::new();
            for hpo_duplet in onset_list {
                *counts_map.entry(hpo_duplet).or_insert(0) += 1;
            }
            for (onset_term, counts) in counts_map.iter() {
                let counted_hpo = CountedHpoTerm::from(onset_term.clone(), *counts, n_onset_observations, &pmid);
                counted_term_list.push(counted_hpo);
            }
        }
        Ok(counted_term_list)
    }




 

   
   

}
