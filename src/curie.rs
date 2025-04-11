use crate::rphetools_traits::TableCell;

use crate::error::{self, Error, Result};

/// A valid curie must have a non-empty prefix and a non-empty numeric suffic
/// white-space is not allowed.
fn check_valid_curie(s: &str) -> Result<bool> {
    if s.is_empty() {
        return Err(Error::CurieError {
            msg: "Empty CURIE".to_string(),
        });
    } else if let Some(pos) = s.find(':') {
        if s.chars().any(|c| c.is_whitespace()) {
            return Err(Error::CurieError {
                msg: format!("Contains stray whitespace: '{}'", s),
            });
        } else if s.matches(':').count() != 1 {
            return Err(Error::CurieError {
                msg: format!("Invalid CURIE with more than one colon: '{}", s),
            });
        } else if pos == 0 {
            return Err(Error::CurieError {
                msg: format!("Invalid CURIE with no prefix: '{}'", s),
            });
        } else if pos == s.len() - 1 {
            return Err(Error::CurieError {
                msg: format!("Invalid CURIE with no suffix: '{}'", s),
            });
        } else if let Some((_prefix, suffix)) = s.split_once(':') {
            if !suffix.chars().all(char::is_numeric) {
                return Err(Error::CurieError {
                    msg: format!("Invalid CURIE with non-digit characters in suffix: '{}'", s),
                });
            }
        }
    } else {
        return Err(Error::CurieError {
            msg: format!("Invalid CURIE with no colon: '{}'", s),
        });
    }
    Ok(true)
}

/// We use the CURIE struct to represent PMIDs, disease identifiers, and HGNC identifiers
/// We use separate creator objects to ensure that the prefix is correct
#[derive(Clone, Debug)]
pub struct Curie {
    curie_value: String,
}

impl TableCell for Curie {
    fn value(&self) -> String {
        self.curie_value.clone()
    }
}

impl Curie {
    pub fn new_pmid(value: &str) -> Result<Self> {
        check_valid_curie(value)?;
        if !value.starts_with("PMID") {
            return Err(Error::CurieError {
                msg: format!("Invalid PubMed prefix: '{}'", value),
            });
        }
        return Ok(Curie {
            curie_value: value.to_string(),
        });
    }

    pub fn new_disease_id(value: &str) -> Result<Self> {
        let valid_curie = check_valid_curie(value);
        if valid_curie.is_err() {
            return Err(Error::DiseaseIdError {
                msg: format!("Invalid disease identifier: {}", valid_curie.err().unwrap()),
            });
        } else if !(value.starts_with("OMIM") || value.starts_with("MONDO")) {
            return Err(Error::DiseaseIdError {
                msg: format!("Disease id has invalid prefix: '{}'", value),
            });
        } else {
            return Ok(Curie {
                curie_value: value.to_string(),
            });
        }
    }

    pub fn new_hgnc_id(value: &str) -> Result<Self> {
        let valid_curie = check_valid_curie(value);
        if valid_curie.is_err() {
            return Err(Error::HgncError {
                msg: format!("Invalid HGNC identifier: {}", valid_curie.err().unwrap()),
            });
        } else if !value.starts_with("HGNC") {
            return Err(Error::HgncError {
                msg: format!("HNGC id has invalid prefix: '{}'", value),
            });
        } else {
            return Ok(Curie {
                curie_value: value.to_string(),
            });
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_pmid_ctor() {
        let tests = vec![
            ("PMID:12345", "PMID:12345"),
            ("PMID: 12345", "Contains stray whitespace: 'PMID: 12345'"),
            ("PMID:12345 ", "Contains stray whitespace: 'PMID:12345 '"),
            (" PMID:12345", "Contains stray whitespace: ' PMID:12345'"),
            ("PMD:12345", "Invalid PubMed prefix: 'PMD:12345'"),
            ("PMID12345", "Invalid CURIE with no colon: 'PMID12345'"),
            (
                "PMID:12a45",
                "Invalid CURIE with non-digit characters in suffix: 'PMID:12a45'",
            ),
            ("", "Empty CURIE"),
        ];
        for test in tests {
            let pmid = Curie::new_pmid(test.0);
            match pmid {
                Ok(pmid) => assert_eq!(test.1, pmid.value()),
                Err(err) => assert_eq!(test.1, err.to_string()),
            }
        }
    }

    #[test]
    fn test_disease_id() {
        let tests = vec![
            ("OMIM:154700", "OMIM:154700"),
            (
                "OMIM154700",
                "Invalid disease identifier: Invalid CURIE with no colon: 'OMIM154700'",
            ),
            (
                "OMIM: 154700",
                "Invalid disease identifier: Contains stray whitespace: 'OMIM: 154700'",
            ),
            (
                "OMIM:154700 ",
                "Invalid disease identifier: Contains stray whitespace: 'OMIM:154700 '",
            ),
            (
                " OMIM:154700",
                "Invalid disease identifier: Contains stray whitespace: ' OMIM:154700'",
            ),
            (
                " OMIM:154700 ",
                "Invalid disease identifier: Contains stray whitespace: ' OMIM:154700 '",
            ),
            (
                "OMIM:",
                "Invalid disease identifier: Invalid CURIE with no suffix: 'OMIM:'",
            ),
            (
                ":154700",
                "Invalid disease identifier: Invalid CURIE with no prefix: ':154700'",
            ),
            ("OMM:154700", "Disease id has invalid prefix: 'OMM:154700'"),
            ("MONDO:0007947", "MONDO:0007947"),
            (
                "MOND:0007947",
                "Disease id has invalid prefix: 'MOND:0007947'",
            ),
        ];
        for test in tests {
            let disease_id = Curie::new_disease_id(test.0);
            match disease_id {
                Ok(disease_id) => assert_eq!(test.1, disease_id.value()),
                Err(err) => assert_eq!(test.1, err.to_string()),
            }
        }
    }

    #[test]
    fn test_hgnc_id() {
        let tests = vec![("HGNC:3603", "HGNC:3603")];

        for test in tests {
            let hgnc_id = Curie::new_hgnc_id(test.0);
            match hgnc_id {
                Ok(hgnc) => assert_eq!(test.1, hgnc.value()),
                Err(err) => assert_eq!(test.1, err.to_string()),
            }
        }
    }
}
