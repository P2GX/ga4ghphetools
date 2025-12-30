use crate::repo::{cohort_qc::CohortQc};




pub struct RepoQc {
    cohort_qc_list: Vec<CohortQc>
}


impl RepoQc {
    pub fn new(cohort_qc_list: Vec<CohortQc>) -> Self {
        Self {
            cohort_qc_list
        }
    }

    pub fn phenopacket_count(&self) -> usize {
        let mut c = 0 as usize;
        for cohort in &self.cohort_qc_list {
            c += cohort.ppkt_count();
        }
        return c;
    }

    pub fn check_moi(&self) {
        println!("Checking agreement of MOI and allele counts");
        let mut misfit = 0 as usize;
        for cohort in &self.cohort_qc_list {
            misfit += cohort.check_moi();
        }
         println!("Found misalignment in {} cohorts", misfit);
    }

}