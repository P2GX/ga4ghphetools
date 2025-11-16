use std::collections::HashMap;

use crate::{dto::{cohort_dto::CohortData, hpo_term_dto::HpoTermData}, hpoa::counted_hpo_term::CountedHpoTerm};




pub struct HpoTermCounter {
}


impl HpoTermCounter {

    /// Extract a list of CountedHpoTerm objects for the ages of onset for each PMID.
    pub fn pmid_term_count_list(
        cohort_dto: &CohortData
    ) 
    -> Result<Vec<CountedHpoTerm>, String> {
        let mut counted_term_list: Vec<CountedHpoTerm> = Vec::new();
        let mut pmid_to_term_data_list_d : HashMap<String, Vec<HpoTermData>> = HashMap::new();
        for row in &cohort_dto.rows {
            let pmid = row.individual_data.pmid.clone();
            let term_data_list = pmid_to_term_data_list_d
                .entry(pmid)
                .or_insert(Vec::new());
            for (value, duplet) in row.hpo_data.iter().zip(cohort_dto.hpo_headers.clone()) {
                let hpo_data = HpoTermData{
                    term_duplet: duplet.clone(),
                    entry: value.clone(),
                };
                term_data_list.push(hpo_data);
            }
        }
        // When we get here, we have a list of strings for each PMID. For the HPOA output, we want to represent them as
        // HPO onset terms with the correct frequencies.
        for (pmid, term_data_vec) in &pmid_to_term_data_list_d {
            let mut counted_term_map: HashMap<String, CountedHpoTerm> = HashMap::new();
            for term_data in term_data_vec {
                if term_data.is_not_ascertained() {
                    continue; // Some of our upstream data has "na" values for a specific HPO term. There is no use adding these non-values to the HPOA (because they would yield 0/0)
                }
                let cterm = counted_term_map.entry(term_data.term_id().to_string()).or_insert(CountedHpoTerm::new(term_data.term_id(), term_data.label(), &pmid));
                cterm.increment_value(&term_data.entry);
            }
            // we are now done with the counted terms for the current PMID
            counted_term_list.extend(counted_term_map.values().cloned());
        }
        Ok(counted_term_list)
    }

}