

use crate::dto::hpo_term_dto::HpoTermDuplet;



pub struct CountedHpoTerm {
    hpo_id: String,
    hpo_label: String,
    numerator: u32,
    denominator: u32,
}

impl CountedHpoTerm {
    pub fn new(id: &str, label: &str) -> Self {
        Self { 
            hpo_id: id.to_string(), 
            hpo_label: label.to_string(), 
            numerator: 0, 
            denominator: 0,
        }
    }

    pub fn new_with_counts(id: &str, label: &str, num: u32, denom: u32) -> Self {
        Self { 
            hpo_id: id.to_string(), 
            hpo_label: label.to_string(), 
            numerator: num, 
            denominator: denom,
        }
    }

    pub fn from_simple_term(onset_term: HpoTermDuplet) -> Self {
        let hpo_id = onset_term.hpo_id().to_string();
        let label = onset_term.hpo_label().to_string();
        Self::new(&hpo_id, &label)
    }

    pub fn increment_observed(&mut self) {
        self.numerator += 1;
        self.denominator += 1;
    }
}