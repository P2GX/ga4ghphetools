use std::collections::HashMap;

use ontolius::term::simple::SimpleMinimalTerm;
use regex::Regex;

use crate::{dto::cohort_dto::CohortData, age::age_util, hpoa::counted_hpo_term::CountedHpoTerm};



pub struct HpoaOnsetCalculator {
    /// Key: a PMID, value: A list of HPO onset terms
    onset_to_count_d: HashMap<String, CountedHpoTerm>,
}

impl HpoaOnsetCalculator {
    pub fn new() -> Self {
        Self { 
            onset_to_count_d: HashMap::new(), 
        }
    } 

    pub fn add_onset(&mut self, onset_str: &str) -> Result<(), String> {
        let term = age_util::is_valid_age_string(onset_str);
        if let Ok(onset_term) = Self::get_hpo_onset_term_from_iso8601(onset_str) {

        }

        Ok(())
    }

    pub fn add_onset_term(&mut self, pmid: &str, onset_term: SimpleMinimalTerm) {
        // Insert a new Vec if the key does not exist yet
        let counted_term = self
            .onset_to_count_d
            .entry(pmid.to_string())
            .or_insert(CountedHpoTerm::from_simple_term(onset_term));
        // modify in place
        counted_term.increment_observed();
    }

    pub fn ingest_dto(&mut self, dto: &CohortData) -> Result<(), String>  {
        for row in &dto.rows {
            let pmid = row.individual_dto.pmid.clone();
            if row.individual_dto.age_of_onset != "na" {
                let onset = row.individual_dto.age_of_onset.clone();
                match age_util::ONSET_TERM_DICT.get(&onset) {
                    Some(term) => self.add_onset_term(&pmid, term.clone()),
                    None => {
                        let term = Self::get_term_from_age_string(&onset)?;
                        self.add_onset_term(&pmid, term.clone());
                    }
                }            
            }
        }
        Ok(())
    }

    /// Get a SimpleMinimalTerm representing age from an age string (iso8601 or gestationalclear)
    pub fn get_term_from_age_string(onset: &str) -> Result<SimpleMinimalTerm, String> {
        let label = if onset.starts_with("P") {
            Self::get_hpo_onset_term_from_iso8601(onset)
        } else {
            return Err("NEED TO REFACTOR FOR GA".to_string());
            //Self::get_hpo_onset_term_from_gestational_age(onset)
        }?; 
        match age_util::ONSET_TERM_DICT.get(&label) {
            Some(term) => Ok(term.clone()),
            None => Err(format!("Could not find onset term for '{label}'")),
        }
    }


    pub fn get_hpo_onset_term_from_iso8601(isostring: &str) -> Result<String, String> {
        let iso8601_regex = Regex::new(r"^P(?:(\d+)Y)?(?:(\d+)M)?(?:(\d+)D)?$")
            .map_err(|e| format!("Invalid regex: {}", e))?;

        let captures = iso8601_regex
            .captures(isostring)
            .ok_or_else(|| format!("Could not parse ISO8601 string: {}", isostring))?;

        let years: usize = captures
            .get(1)
            .map_or(Ok(0), |m| m.as_str().parse())
            .map_err(|_| "Invalid year format")?;

        let months: usize = captures
            .get(2)
            .map_or(Ok(0), |m| m.as_str().parse())
            .map_err(|_| "Invalid month format")?;

        let days: usize = captures
            .get(3)
            .map_or(Ok(0), |m| m.as_str().parse())
            .map_err(|_| "Invalid day format")?;

        let label = if years >= 60 {
            "Late onset"
        } else if years >= 40 {
            "Middle age onset"
        } else if years >= 16 {
            "Young adult onset"
        } else if years >= 5 {
            "Juvenile onset"
        } else if years >= 1 {
            "Childhood onset"
        } else if months >= 1 {
            "Infantile onset"
        } else if days >= 1 {
            "Neonatal onset"
        } else if days == 0 {
            "Congenital onset"
        } else {
            return Err(format!("Could not determine onset label from: {}", isostring));
        };
        Ok(label.to_string())
    }


   

}

// region:    --- Tests

#[cfg(test)]
mod tests {
    use super::*;
    use ontolius::term::MinimalTerm;
    use rstest::rstest;

    #[rstest]
    #[case("Middle age onset", "P40Y")]
    #[case("Middle age onset", "P40Y3M22D")]
    #[case("Late onset", "P70Y")]
    #[case("Neonatal onset", "P1D")]
    #[case("Infantile onset", "P2M1D")]
    #[case("Childhood onset", "P3Y2M1D")]
    #[case("Juvenile onset", "P13Y2M1D")]
    #[case("Young adult onset", "P23Y2M1D")]
    fn test_isoage(#[case] onset_term: &str, #[case] age_string: &str) {
        let result= HpoaOnsetCalculator::get_term_from_age_string(age_string);
        assert!(result.is_ok());
        let smt: SimpleMinimalTerm  = result.unwrap();
        let label = smt.name();
        assert_eq!(onset_term, label);
    }


    
}

// endregion: --- Tests