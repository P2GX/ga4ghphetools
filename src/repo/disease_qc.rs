use std::collections::HashSet;

use phenopackets::schema::v2::{Phenopacket, core::genomic_interpretation::Call};

use crate::{dto::cohort_dto::{CohortData, DiseaseData}, repo::qc_report::QcReport};



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

    pub fn add_ppkt(&mut self, ppkt: Phenopacket) {
        self.ppkt_list.push(ppkt);
    }


    pub fn phenopacket_count(&self) -> usize {
        return self.ppkt_list.len();
    }

    pub fn check_moi(&self) -> Vec<QcReport> {
        let mut errs: Vec<QcReport> = Vec::new();
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
                let qc = QcReport::moi_mismatch(&self.disease_data_display(), &ppkt.id, &allowable_allele_counts, ac);
                errs.push(qc);
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

    pub fn check_all_rows_output_as_ppkt(&self) -> Option<QcReport> {
        let n_nrows = self.cohort.rows.len();
        let n_phenopackets = self.phenopacket_count();
        if n_nrows == n_phenopackets {
            None
        } else {
            Some(QcReport::count_mismatch(&self.disease_data_display(), n_nrows, n_phenopackets))
        }
    }

    pub fn check_no_hpo(&self) -> Vec<QcReport> {
        let mut errs: Vec<QcReport> = Vec::new();
        for ppkt in &self.ppkt_list {
            let n_hpo = ppkt.phenotypic_features.iter().filter(|p| ! p.excluded )
            .count();
            if n_hpo == 0 {
                errs.push(QcReport::no_hpo(&self.disease_data_display(), &ppkt.id));
            }
        }
        errs
    }
    
    pub fn check_acronym(&self) -> Vec<QcReport> {
       let mut errs: Vec<QcReport> = Vec::new();
       let acronym = self.cohort.acronym();
       let acronym = self.cohort.acronym();
       if ! acronym.contains("_") {
            errs.push(QcReport::malformed_acronym(acronym));
       }
       errs
    }
}


#[cfg(test)]
mod test {
    use super::*;
    use rstest::rstest;
    use rstest::fixture;
  
    // : 'RUNX2_CLCD1' -
    #[fixture]
    fn disease_qc_valid_acronym() -> DiseaseQc  {
        let ddata = DiseaseData::new("OMIM:119600", "Cleidocranial dysplasia");
        let fake_ppkt_list: Vec<Phenopacket> = Vec::new();
        let mut cohort = CohortData::mendelian(ddata.clone(), vec![], vec![], "fake version");
        cohort.cohort_acronym = Some("RUNX2_CLCD1".to_string());
        DiseaseQc::new(&ddata, &cohort)
    }

    #[fixture]
    fn disease_qc_invalid_acronym() -> DiseaseQc  {
        let ddata = DiseaseData::new("OMIM:621280", "Retinitis pigmentosa 100");
        let fake_ppkt_list: Vec<Phenopacket> = Vec::new();
        let mut cohort = CohortData::mendelian(ddata.clone(), vec![], vec![], "fake version");
        cohort.cohort_acronym = Some("RP100".to_string());
        DiseaseQc::new(&ddata, &cohort)
    }

    #[rstest]
    fn check_valid_acronym(disease_qc_valid_acronym: DiseaseQc) {
        let qc_report_list = disease_qc_valid_acronym.check_acronym();
        assert!(qc_report_list.is_empty());
    }

     #[rstest]
    fn check_invalid_acronym(disease_qc_invalid_acronym: DiseaseQc) {
        let qc_report_list = disease_qc_invalid_acronym.check_acronym();
        assert_eq!(1, qc_report_list.len());
    }




}