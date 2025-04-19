
use std::collections::HashSet;

use polars::series::implementations;
use regex::Regex;
use lazy_static::lazy_static;

use crate::error::{self, Error, Result};

impl Error {
    fn hgnc_error<T>(val: &str) -> Self
    {
        Error::AlleleError { msg: format!("{}", val) }
    }

    fn empty_allele() -> Self {
        Error::AlleleError{ msg: format!("HGVS cannot be empty")}
    }
}

lazy_static! {
    pub static ref HGVS_RE: Regex = Regex::new(r"c.[\d_]+(.*)").unwrap(); // Capture everything after digits or underscores
    pub static ref SUBSTITUTION_RE: Regex = Regex::new(r"([ACGT]+)([>]{1}[ACGT]+)$").unwrap();
    pub static ref INSERTION_RE: Regex = Regex::new(r"ins[ACGT]+$").unwrap();
    pub static ref DELINS_RE: Regex = Regex::new(r"^c\.(\d+_\d+)delins[A-Za-z0-9]+$").unwrap();
    pub static ref ALLOWED_STRUCTURAL_PREFIX: HashSet<String> =  {
        let mut set = HashSet::new();
        set.insert("DEL".to_string());
        set.insert("DUP".to_string());
        set.insert("INV".to_string());
        set.insert("INS".to_string());
        set.insert("TRANSL".to_string());
        set
    };

}

pub fn check_valid_hgvs(value: &str) -> bool {
    // if we get here, there was a non-empty string that starts with "c."
    if let Some(captures) = HGVS_RE.captures(value) {
        if let Some(matched_substr) = captures.get(1) {
            // we now have either G>T, del, insT (etc), or delinsT (etc)
            let remaining_hgvs = matched_substr.as_str();
            if SUBSTITUTION_RE.is_match(remaining_hgvs) {
                return true;
            } else if INSERTION_RE.is_match(remaining_hgvs) {
                return true;
            } else if remaining_hgvs == "del" {
                return true;
            } else if DELINS_RE.is_match(remaining_hgvs) {
                return true;
            }
            return false;
        }
    }
    return false;
}


pub fn check_valid_structural(value: &str) -> bool {
    let parts: Vec<&str> = value.split(':').collect();
    let prefix = parts[0];
    let suffix = parts[1..].join(":"); // in case the original string contains ":"
    let structural_var = suffix.trim();
    return  ALLOWED_STRUCTURAL_PREFIX.contains(prefix)
}