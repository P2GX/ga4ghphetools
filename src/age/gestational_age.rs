//!  Gestational age is a measure of the age of a pregnancy usually taken from the beginning of the woman's last menstrual period (LMP).
//! 
//!  Gestational age is reported in weeks and days, e.g.,  22 weeks and 3 days. This may be reported as 22 3/7, but there is no single standard
//! In our templates, we would report this gestational age as G22w3d 

use std::collections::HashMap;

use once_cell::sync::Lazy;
use regex::Regex;

use crate::{age::hpo_age::ONSET_TERM_DICT, dto::hpo_term_dto::HpoTermDuplet};

/// gestational age represented as G37d2d, G32w, G11w3d, etc.
pub static GESTATIONAL_AGE_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^G(\d+)w(?:([0-6])d)?$").unwrap()
});


pub static PRENATAL_ONSET_TERM_DICT: Lazy<HashMap<String, HpoTermDuplet>> = Lazy::new(|| {
    let wanted = ["Antenatal onset", "Embryonal onset","Fetal onset", "Late first trimester onset", "Second trimester onset","Third trimester onset"];
    ONSET_TERM_DICT
        .iter()
        .filter(|(k, _)| wanted.contains(&k.as_str()))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect()
});


pub struct GestationalAgeValidator {}

impl GestationalAgeValidator {
    pub fn is_valid(cell_value: &str) -> bool {
        GESTATIONAL_AGE_RE.is_match(cell_value)
    }

    pub fn is_valid_gestational_term(cell_value: &str) -> bool {
        PRENATAL_ONSET_TERM_DICT.contains_key(cell_value)
    }

     /// Derive an HPO onset term from a Gestational Age string such as G32w3d
    pub fn get_duplet(gestational_age: &str) -> Result<HpoTermDuplet, String> {
        let captures = GESTATIONAL_AGE_RE
            .captures(gestational_age)
            .ok_or_else(|| format!("Could not parse Gestational Age string: '{}'", gestational_age))?;

        let weeks: usize = captures
            .get(1)
            .map_or(Ok(0), |m| m.as_str().parse())
            .map_err(|_| "Invalid weeks format")?;

        let term_option = if weeks >= 28 {
            //   ("HP:0034197", "Third trimester onset")];
            PRENATAL_ONSET_TERM_DICT.get("Third trimester onset")
        } else if weeks >= 14 {
            //  ("HP:0034198", "Second trimester onset")
            PRENATAL_ONSET_TERM_DICT.get("Second trimester onset")
        } else if weeks >= 11 {
            // Late first trimester onset HP:0034199
             PRENATAL_ONSET_TERM_DICT.get("Late first trimester onset")
        } else {
            // Embryonal onset HP:0011460
            PRENATAL_ONSET_TERM_DICT.get("Embryonal onset")
        };
        println!("LEN IS {}", PRENATAL_ONSET_TERM_DICT.len());
        match term_option {
            Some(prenatal_term) => Ok(prenatal_term.clone()),
            None => Err(format!("Could not find HPO Onset term for '{}'", gestational_age))
        }
    }


}




#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

 


    #[rstest]
    #[case("Third trimester onset", "G37w1d")]
    #[case("Third trimester onset", "G37w")]
    #[case("Second trimester onset", "G22w6d")]
    #[case("Late first trimester onset", "G12w6d")]
    #[case("Embryonal onset", "G9w")]
    fn test_gestationalage(#[case] label: &str, #[case] age_string: &str) {
        let result= GestationalAgeValidator::get_duplet(age_string);
        assert!(result.is_ok());
        let onset_term: HpoTermDuplet  = result.unwrap();
        assert_eq!(onset_term.hpo_label(), label);
    }

    #[rstest]
    #[case("Third trimester onset", true)]
    #[case("Third trimesteronset", false)]
    #[case("Second trimester onset", true)]
    #[case("Late first trimester onset", true)]
    #[case("Embryonal onset", true)]
    #[case("Fetal onset", true)]
    #[case("Antenatal onset", true)]
    fn test_valid_gestational_term(
        #[case] label: &str,
        #[case] is_valid: bool
    ) {
        let v = GestationalAgeValidator::is_valid_gestational_term(label);
        assert_eq!(is_valid, v);
    }


    
}

// endregion: --- Tests