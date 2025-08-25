//!  Gestational age is a measure of the age of a pregnancy usually taken from the beginning of the woman's last menstrual period (LMP).
//! 
//!  Gestational age is reported in weeks and days, e.g.,  22 weeks and 3 days. This may be reported as 22 3/7, but there is no single standard
//! In our templates, we would report this gestational age as G22w3d 

use once_cell::sync::Lazy;
use regex::Regex;

/// gestational age represented as G37d2d, G32w, G11w3d, etc.
pub static GESTATIONAL_AGE_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^G(\d+)w(?:([0-6])d)?$").unwrap()
});


pub struct GestationalAgeValidator {}

impl GestationalAgeValidator {
    pub fn is_valid(cell_value: &str) -> bool {
        GESTATIONAL_AGE_RE.is_match(cell_value)
    }

     /// Derive an HPO onset term from a Gestational Age string such as G32w3d
    pub fn get_hpo_onset_term_from_gestational_age(gestational_age: &str) -> Result<String, String> {
        let captures = GESTATIONAL_AGE_RE
            .captures(gestational_age)
            .ok_or_else(|| format!("Could not parse Gestational Age string: '{}'", gestational_age))?;

        let weeks: usize = captures
            .get(1)
            .map_or(Ok(0), |m| m.as_str().parse())
            .map_err(|_| "Invalid weeks format")?;

        let label = if weeks >= 28 {
            "Third trimester onset"
        } else if weeks >= 14 {
            "Second trimester onset"
        } else if weeks >= 11 {
            "Late first trimester onset"
        } else {
            "Embryonal onset"
        };
        Ok(label.to_string())
    }


}




#[cfg(test)]
mod tests {
    use super::*;
    use ontolius::term::MinimalTerm;
    use rstest::rstest;

 


    #[rstest]
    #[case("Third trimester onset", "G37w1d")]
    #[case("Third trimester onset", "G37w")]
    #[case("Second trimester onset", "G22w6d")]
    #[case("Late first trimester onset", "G12w6d")]
    #[case("Embryonal onset", "G9w")]
    fn test_gestationalage(#[case] onset_term: &str, #[case] age_string: &str) {
        let result= GestationalAgeValidator::get_hpo_onset_term_from_gestational_age(age_string);
        assert!(result.is_ok());
        let label: String  = result.unwrap();
        assert_eq!(onset_term, label);
    }


    
}

// endregion: --- Tests