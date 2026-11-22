use std::collections::HashSet;

use phenopackets::schema::v2::{Phenopacket, core::genomic_interpretation::Call};
use crate::{dto::cohort_data::{CohortData, DiseaseData}, error::cohort_error::CohortError};



#[derive(Clone, Debug)]
pub struct DiseaseQc {
    disease_data: DiseaseData,
    ppkt_list: Vec<Phenopacket>,
    cohort: CohortData,
}


impl DiseaseQc {
    pub fn new(disease_data: &DiseaseData, cohort: &CohortData) -> Self {
        Self { 
            disease_data: disease_data.clone(), 
            ppkt_list: Vec::new(),
            cohort: cohort.clone()
        }
    }

    pub fn from_ppkt_list(
        disease_data: &DiseaseData, 
        cohort: &CohortData,
        ppkt_list: &[&Phenopacket]
    ) -> Self {
        Self {
            disease_data: disease_data.clone(), 
            ppkt_list: ppkt_list.iter().map(|p| (*p).clone()).collect(),
            cohort: cohort.clone()
        }
    }

    pub fn add_ppkt(&mut self, ppkt: Phenopacket) {
        self.ppkt_list.push(ppkt);
    }


    pub fn phenopacket_count(&self) -> usize {
        return self.ppkt_list.len();
    }

    pub fn check_moi(&self) -> Vec<CohortError> {
        let mut errs: Vec<CohortError> = Vec::new();
        let mut allowable_allele_counts: HashSet<usize> = HashSet::new();
        for moi in &self.disease_data.mode_of_inheritance_list {
            if moi.is_autosomal_dominant() {
                allowable_allele_counts.insert(1);
            } else if moi.is_autosomal_recessive() {
                allowable_allele_counts.insert(2);
            } else if moi.is_x_chromosomal() {
                allowable_allele_counts.insert(1);
            } else if moi.is_pseudoautosomal_dominant() {
                allowable_allele_counts.insert(1);
            }  else if moi.is_pseudoautosomal_recessive() {
                allowable_allele_counts.insert(2);
            } else {
                eprintln!("Did not recognize MOI: {:?}", moi);
            }
        }
        for ppkt in &self.ppkt_list {
            let ac = Self::get_allele_count(ppkt);
            if ! allowable_allele_counts.contains(&ac) {
                let err = CohortError::moi_mismatch(&ppkt.id, &allowable_allele_counts, ac);
                errs.push(err);
            }
        }
        errs
    }

    fn disease_data_display(&self) -> String {
       format!("{} ({}/{})",  
       self.disease_data.disease_label, 
       self.disease_data.disease_id,
       self.disease_data.gene_transcript_list[0].gene_symbol)
    }


    fn get_allele_count(ppkt: &Phenopacket) -> usize {
        let mut ac = 0_usize;
        for interpret in &ppkt.interpretations {
            match &interpret.diagnosis {
                Some(dx) => {
                    for gi in &dx.genomic_interpretations {
                        if let Some(Call::VariantInterpretation(ref vi)) = gi.call {
                               if let Some(vd) =  &vi.variation_descriptor {
                                    if let Some(oclz) =  &vd.allelic_state {
                                        let count = match oclz.label.as_str() {
                                            "homozygous" => 2,
                                            "heterozygous" => 1,
                                            "hemizygous" =>  1,
                                            _ => 0,
                                        };
                                        ac += count;
                                    }
                               }
                            }
                    }
                },
                None => {println!("Could not find dx")},
            }
        }

        ac
    }

    pub fn check_all_rows_output_as_ppkt(&self) -> Option<CohortError> {
        let n_nrows = self.cohort.rows.len();
        let n_phenopackets = self.phenopacket_count();
        if n_nrows == n_phenopackets {
            None
        } else {
            Some(CohortError::count_mismatch( n_nrows, n_phenopackets))
        }
    }

    pub fn check_no_hpo(&self) -> Vec<CohortError> {
        let mut errs: Vec<CohortError> = Vec::new();
        for ppkt in &self.ppkt_list {
            let n_hpo = ppkt.phenotypic_features.iter().filter(|p| ! p.excluded )
            .count();
            if n_hpo == 0 {
                errs.push(CohortError::no_observed_hpo_annots(&ppkt.id));
            }
        }
        errs
    }
    
    pub fn check_acronym(&self) -> Option<CohortError> {
        let acronym = self.cohort.acronym();
        if is_valid_acronym(&acronym) { None } 
        else { Some(CohortError::malformed_acronym(&acronym)) }
    }
}

/// A cohort acronym has the form `GENE_DISEASEACRONYM`, e.g. `ARHGEF15_BSVD5`.
pub fn is_valid_acronym(acronym: &str) -> bool {
    match acronym.split_once('_') {
        Some((gene, disease)) => {
            !gene.is_empty()
                && !disease.is_empty()
                && !acronym.chars().any(char::is_whitespace)
        }
        None => false,
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rstest::rstest;
   
    #[rstest]
    fn check_valid_acronym() {
       assert!(is_valid_acronym("ARHGEF15_BSVD5"));
    }

     #[rstest]
    fn check_invalid_acronym() {
        assert!(!is_valid_acronym("RP100"));
        assert!(!is_valid_acronym("RP100_"));
        assert!(!is_valid_acronym("RP100_ BSVD5"));
        assert!(!is_valid_acronym("_BSVD5"));
    }

}