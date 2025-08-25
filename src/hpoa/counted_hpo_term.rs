

use crate::dto::hpo_term_dto::HpoTermDuplet;



pub struct CountedHpoTerm {
    hpo_id: String,
    hpo_label: String,
    numerator: usize,
    denominator: usize,
    pmid: String
}

impl CountedHpoTerm {
    pub fn new(id: &str, label: &str, pmid: &str) -> Self {
        Self::new_with_counts(id, label, 0, 0, pmid)
    }

    pub fn new_with_counts(id: &str, label: &str, num: usize, denom: usize, pmid: &str) -> Self {
        Self { 
            hpo_id: id.to_string(), 
            hpo_label: label.to_string(), 
            numerator: num, 
            denominator: denom,
            pmid: pmid.to_string()
        }
    }

    pub fn from(onset_term: HpoTermDuplet, num: usize, denom: usize, pmid: &str) -> Self {
        let hpo_id = onset_term.hpo_id().to_string();
        let label = onset_term.hpo_label().to_string();
        Self::new_with_counts(&hpo_id, &label, num, denom, pmid)
    }

    pub fn increment_observed(&mut self) {
        self.numerator += 1;
        self.denominator += 1;
    }

    #[inline]
    pub fn hpo_id(&self) -> &str {
        &self.hpo_id
    }

    #[inline]
    pub fn hpo_label(&self) -> &str {
        &self.hpo_label
    }

    #[inline]
    pub fn numerator(&self) -> usize {
        self.numerator
    }

    #[inline]
    pub fn denominator(&self) -> usize {
        self.denominator
    }

    #[inline]
    pub fn pmid(&self) -> &str {
        &self.pmid
    }

    pub fn freq_string(&self) -> String {
        format!("{}/{}", self.numerator, self.denominator)
    }
}