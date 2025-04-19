use std::{cell, collections::HashSet};

use lazy_static::lazy_static;
use regex::Regex;



lazy_static! {
    pub static ref ALLOWED_AGE_LABELS: HashSet<String> =  {
        let mut set = HashSet::new();
        set.insert("Late onset".to_string());
        set.insert("Middle age onset".to_string());
        set.insert("Young adult onset".to_string());
        set.insert( "Late young adult onset".to_string());
        set.insert("Intermediate young adult onset".to_string());
        set.insert("Early young adult onset".to_string());
        set.insert("Adult onset".to_string());
        set.insert("Juvenile onset".to_string());
        set.insert("Childhood onset".to_string());
        set.insert("Infantile onset".to_string());
        set.insert("Neonatal onset".to_string());
        set.insert("Congenital onset".to_string());
        set.insert("Antenatal onset".to_string());
        set.insert("Embryonal onset".to_string());
        set.insert("Fetal onset".to_string());
        set.insert("Late first trimester onset".to_string());
        set.insert("Second trimester onset".to_string());
        set.insert("Third trimester onset".to_string());
        set
    };

    pub static ref  ISO8601_RE: Regex = Regex::new(r"^P(?:(\d+)Y)?(?:(\d+)M)?(?:(\d+)D)?$").unwrap();
    pub static ref GESTATIONAL_AGE_RE: Regex = Regex::new(r"G\d+w[0-6]d").unwrap();

}



pub fn is_valid_age_string(cell_value: &str) -> bool {
    // empty not allowed
    if cell_value.is_empty() {
        return false;
    }
    // but na is OK
    if cell_value == "na" {
        return true;
    }
    // check for match to HPO Onset terms
    if ALLOWED_AGE_LABELS.contains(cell_value) {
        return true;
    }
    // check for match to ISO (601)
    if ISO8601_RE.is_match(cell_value) {
        return true;
    }

    if GESTATIONAL_AGE_RE.is_match(cell_value) {
        return true;
    }

    false
}