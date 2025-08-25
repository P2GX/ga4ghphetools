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


/*
static FORBIDDEN_CHARS: Lazy<HashSet<char>> = Lazy::new(|| {
    ['/', '\\', '(', ')'].iter().copied().collect()
});
 */


use crate::age::{gestational_age::GestationalAgeValidator, hpo_age::HpoTermAge, iso_age::Iso8601Age};

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

