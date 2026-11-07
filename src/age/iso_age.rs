use std::{fmt::format, sync::LazyLock};
use regex::Regex;

use crate::{age::hpo_age, dto::hpo_term_dto::HpoTermDuplet};




static ISO8601_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^P(?:(\d+)Y)?(?:(\d+)M)?(?:(\d+)D)?$").unwrap()
});

pub struct Iso8601Age{}

impl Iso8601Age {
    /// Check whether the string value is a valid ISO match. It is efficient to check the length of the string
    /// so that a bare "P" is not matched; then the regex is correct.
    pub fn is_valid(cell_value: &str) -> bool {
        cell_value.len() > 1 && ISO8601_RE.is_match(cell_value)
    }


     pub fn get_hpo_onset_term_from_iso8601(iso_string: &str) -> Result<String, String> {
        if iso_string.len() < 3 {
            return Err(format!("Malformed iso age string: {}", iso_string));
        }

        let captures = ISO8601_RE
            .captures(iso_string)
            .ok_or_else(|| format!("Could not parse ISO8601 string: {}", iso_string))?;

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
            return Err(format!("Could not determine onset label from: {}", iso_string));
        };
        Ok(label.to_string())
    }



    pub fn get_duplet(cell_value: &str) -> Result<HpoTermDuplet, String> {
        let hpo_label = Self::get_hpo_onset_term_from_iso8601(cell_value)?;
        return hpo_age::HpoTermAge::get_duplet(&hpo_label);
    }

}



#[cfg(test)]
mod tests {
    use super::*;
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
        let result= Iso8601Age::get_duplet(age_string);
        assert!(result.is_ok());
        let duplet: HpoTermDuplet  = result.unwrap();
        assert_eq!(onset_term, duplet.hpo_label());
    }


    
}
