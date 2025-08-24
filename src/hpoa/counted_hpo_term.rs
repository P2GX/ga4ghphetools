use ontolius::{common::hpo, term::{simple::SimpleMinimalTerm, MinimalTerm}, Identified};



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

    pub fn from_simple_term(onset_term: SimpleMinimalTerm) -> Self {
        let hpo_id = onset_term.identifier().to_string();
        let label = onset_term.name().to_string();
        Self::new(&hpo_id, &label)
    }

    pub fn increment_observed(&mut self) {
        self.numerator += 1;
        self.denominator += 1;
    }
}