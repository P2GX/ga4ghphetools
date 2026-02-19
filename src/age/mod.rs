//! Tools for handling age and time-related values in GA4GH Phenopackets.
//!
//! This module provides:
//! - Validation of ISO-8601 duration strings
//! - Mapping of HPO onset terms to age categories
//! - Support for gestational age representations
//! - Utilities for parsing and comparing age values
//!
//! Example:
//! ```
//! use ga4ghphetools::age::is_valid_age_string;
//!
//! assert!(is_valid_age_string("P3Y6M4D")); // ISO 8601
//! assert!(is_valid_age_string("Congenital onset")); // HPO onset
//! assert!(is_valid_age_string("G20w1d")); // gestational age
//! assert!(is_valid_age_string("na")); // allowed special case
//! ```


pub mod gestational_age;
pub mod hpo_age;
pub mod iso_age;
mod age_etl;


use phenopacket_tools::builders::time_elements::ISO8601_RE;

use crate::{age::{gestational_age::{GESTATIONAL_AGE_RE, GestationalAgeValidator}, hpo_age::HpoTermAge, iso_age::Iso8601Age}, dto::hpo_term_dto::HpoTermDuplet};

pub fn is_valid_age_string(cell_value: &str) -> bool {
    // empty not allowed
    if cell_value.is_empty() {
        return false;
    }
    // but na is OK
    if cell_value == "na" {
        return true;
    }
    // check for match to HPO Onset terms (incuding gestational)
    if HpoTermAge::is_valid(cell_value) {
        return true;
    }
    // check for match to ISO (601)
    if Iso8601Age::is_valid(cell_value) {
        return true;
    }

    if GestationalAgeValidator::is_valid(cell_value) {
        return true;
    }

    false
}


/// Get an HPO Onset term from one of the valid age string values.
/// Note that we assume that we are only getting candidate ages here, not "na", "observed", "expected"
pub fn get_onset_term(cell_value: &str) -> Result<HpoTermDuplet, String> {
   if HpoTermAge::is_valid(cell_value) {
        return HpoTermAge::get_duplet(cell_value);
    } else if  Iso8601Age::is_valid(cell_value) {
        return Iso8601Age::get_duplet(cell_value);
    } else if GestationalAgeValidator::is_valid(cell_value) {
        return GestationalAgeValidator::get_duplet(cell_value);
    } else {
        return Err(format!("Malformed age string '{}'", cell_value))
    }
}


/// Processes a raw age string into a standardized clinical format.
///
/// This is a "waterfall" parser that attempts to resolve the input in the following order:
/// 1. **Symbolic Mapping**: Checks for terms like "neonate" or "birth".
/// 2. **Gestational Age**: Preserves strings matching clinical patterns like `G20w2d`.
/// 3. **ISO8601**: Preserved if the input is already a valid duration (e.g., `P25Y`).
/// 4. **Natural Language**: Parses human-readable strings (e.g., "5y 6m") into ISO8601.
///
/// # Arguments
///
/// * `age_string` - A string slice representing the user input.
///
/// # Returns
///
/// * `Option<String>` - The standardized string if valid, or `None` if the input couldn't be parsed.
///
/// # Examples
///
/// ```
/// use ga4ghphetools::age::convert_raw_age_string_to_validated_age_string;
/// let result = convert_raw_age_string_to_validated_age_string("neonate");
/// assert_eq!(result, Some("Neonatal onset".to_string()));
///
/// let result = convert_raw_age_string_to_validated_age_string("5y6m");
/// assert_eq!(result, Some("P5Y6M".to_string()));
/// ```
pub fn convert_raw_age_string_to_validated_age_string(age_string: &str) -> Option<String> {
    age_etl::map_age_string_to_symbolic(age_string)
        .or_else(|| {
            GESTATIONAL_AGE_RE.is_match(age_string)
                .then(|| age_string.to_string())
        })
        .or_else(|| {
            ISO8601_RE.is_match(age_string)
                .then(|| age_string.to_string())
        })
        // Finally, try the human YMD -> ISO parser
        .or_else(|| age_etl::map_ymd_to_iso(age_string))
}

