use std::fmt::format;

use crate::rpyphetools_traits::TableCell;

use regex::Regex;


#[derive(Debug, PartialEq)]
enum AlleleType {
    SmallHgvs,
    ChromosomalDeletion,
    ChromosomalInsertion,
    ChromosomalInversion,
    ChromosomalTranslocation,
    NotAvailable
}


pub struct Allele {
    allele: String,
    allele_type: AlleleType,
}

impl TableCell for Allele {
    fn value(&self) -> String {
        self.allele.clone()
    }
}

fn is_valid_hgvs(value: &str) -> Result<(), String> {
    if value.is_empty() {
        return Err("HGVS is empty".to_string());
    } else if ! value.trim().starts_with("c") {
        return Err(format!("HGVS does not start with c: '{}", value));
    } else if value.chars().any(|c| c.is_whitespace()) {
        return Err(format!("HGVS contains stray whitespace: '{}'", value));
    } else if ! value.starts_with("c.") {
        return Err(format!("HGVS expression did not start with c.: '{}'", value));
    } 
    // if we get here, there was a non-empty string that starts with "c."
    let re = Regex::new(r"c.[\d_]+(.*)").unwrap(); // Capture everything after digits or underscores
    let subsitution = Regex::new(r"([ACGT]+)([>]{1}[ACGT]+)$").unwrap();
    let insertion_re = Regex::new(r"ins[ACGT]+$").unwrap();
    let delins_re = Regex::new(r"^c\.(\d+_\d+)delins[A-Za-z0-9]+$").unwrap();

    if let Some(captures) = re.captures(value) {
        if let Some(matched_substr) = captures.get(1) {
            // we now have either G>T, del, insT (etc), or delinsT (etc)
            let remaining_hgvs = matched_substr.as_str();
            if subsitution.is_match(remaining_hgvs) {
                return Ok(());
            } else if insertion_re.is_match(remaining_hgvs) {
                return Ok(());
            } else if remaining_hgvs == "del" {
                return Ok(());
            } else if delins_re.is_match(remaining_hgvs) {
                return Ok(());
            }
            return Err(format!("Could not id {}", remaining_hgvs));
        }
    }

    return Ok(());
}


impl Allele {

    pub fn new(value: &str) -> Result<Self, String> {
        if value != value.trim() {
            return Err(format!("Could not parse allele: HGVS contains stray whitespace: '{}'", value));
        }
        if value == "na" {
            return Ok(Allele{allele: "na".to_string(), allele_type: AlleleType::NotAvailable});
        }
        let result = is_valid_hgvs(value);
        match result {
            Ok(_) => Ok(Allele {allele: value.to_string(), allele_type:AlleleType::SmallHgvs}),
            Err(err) => {   
                if ! value.contains(":") {
                    return Err(format!("Could not parse allele: {}", err));
                } else if value.starts_with("c") {
                    return Err(err);
                }
                let parts: Vec<&str> = value.split(':').collect();
                let prefix = parts[0];
                let suffix = parts[1..].join(":"); // in case the original string contains ":"
                let structural_var = suffix.trim();
                return match prefix {
                    "DEL" => Ok(Allele{allele: structural_var.to_string(), allele_type:AlleleType::ChromosomalDeletion}),
                    "INS"  => Ok(Allele{allele: structural_var.to_string(), allele_type:AlleleType::ChromosomalInsertion}),
                    "INV"  => Ok(Allele{allele: structural_var.to_string(), allele_type:AlleleType::ChromosomalInversion}),
                    "TRANS" => Ok(Allele{allele: structural_var.to_string(), allele_type:AlleleType::ChromosomalTranslocation}),
                    _ => Err(format!("Unrecognized non-HGVS prefix: '{}' for {}", prefix, value)),
                };
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_hgvs() {
        let test_cases = vec![
        ("c.123_124insT","c.123_124insT"),  // Insertion
        ("c.123A>G","c.123A>G"),  // Substitution
        ("c.34del", "c.34del"),   // Deletion
        ("c.100G>A", "c.100G>A"), // Another substitution
        ("c.200_201del", "c.200_201del"), // Deletion with range
        ("c123A>G","Could not parse allele: HGVS expression did not start with c.: 'c123A>G'"),
        ("c.123A>G ","Could not parse allele: HGVS contains stray whitespace: 'c.123A>G '"),  
        (" c.123A>G","Could not parse allele: HGVS contains stray whitespace: ' c.123A>G'"),  
        ("c.123A > G","Could not parse allele: HGVS contains stray whitespace: 'c.123A > G'"),  
        ("c.123_124delinsT", "c.123_124delinsT")
        ];
        for test in test_cases {
            let allele = Allele::new(test.0);
            match allele {
                Ok(a) => assert_eq!(test.1, a.value()),
                Err(err) => assert_eq!(test.1, err)
            }
        }
    }

    #[test]
    fn test_non_hgvs() {
        let tests = vec![
          //  ("DEL: telomeric 11q deletion", "telomeric 11q deletion"),
           // ("DEL: deletion of exon 4", "deletion of exon 4"),
            ("DELETION: deletion of exon 4", "Unrecognized non-HGVS prefix: 'DELETION' for DELETION: deletion of exon 4"),
            ("na", "na")];
        for test in tests {
            let allele = Allele::new(test.0);
            match allele {
                Ok(a) => assert_eq!(test.1, a.value()),
                Err(err) => assert_eq!(test.1, err)
            }
        }
    }

    #[test]
    fn test_na_allele() {
        let allele = Allele::new("na");
        assert!(allele.is_ok());
        let allele = allele.unwrap();
        assert_eq!("na", allele.value());
        assert_eq!(AlleleType::NotAvailable, allele.allele_type);
    }


}