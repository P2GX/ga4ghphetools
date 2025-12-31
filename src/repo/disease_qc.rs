use std::collections::HashSet;

use phenopackets::schema::v2::{Phenopacket, core::genomic_interpretation::Call};

use crate::dto::cohort_dto::{DiseaseData};



#[derive(Clone, Debug)]
pub struct DiseaseQc {
    disease_data: DiseaseData,
    ppkt_list: Vec<Phenopacket>
}


impl DiseaseQc {
    pub fn new(disease_data: &DiseaseData) -> Self {
        Self { 
            disease_data: disease_data.clone(), 
            ppkt_list: Vec::new() 
        }
    }

    pub fn add_ppkt(&mut self, ppkt: Phenopacket) {
        self.ppkt_list.push(ppkt);
    }


    pub fn phenopacket_count(&self) -> usize {
        return self.ppkt_list.len();
    }

    pub fn check_moi(&self) -> Option<String> {
        let mut allowable_allele_counts: HashSet<usize> = HashSet::new();
        for moi in &self.disease_data.mode_of_inheritance_list {
            if moi.is_autosomal_dominant() {
                allowable_allele_counts.insert(1);
            } else if moi.is_autosomal_recessive() {
                allowable_allele_counts.insert(2);
            } else if moi.is_x_chromosomal() {
                allowable_allele_counts.insert(1);
            } else {
                return Some(format!("Did not recognize MOI: {:?}", moi));
            }
        }
        for ppkt in &self.ppkt_list {
            let ac = Self::get_allele_count(ppkt);
            if ! allowable_allele_counts.contains(&ac) {
                return Some(format!("{}: Expected counts of {:?} but got {} for {}.", ppkt.id,allowable_allele_counts, ac, self.disease_data_display()))
            }
        }
        None
    }

    fn disease_data_display(&self) -> String {
       format!("{} ({}/{})",  
       self.disease_data.disease_label, 
       self.disease_data.disease_id,
       self.disease_data.gene_transcript_list[0].gene_symbol)
    }


    fn get_allele_count(ppkt: &Phenopacket) -> usize {
        let mut ac = 0 as usize;
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
                                            "hemozygous" =>  1,
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
}