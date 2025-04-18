use std::collections::HashSet;

use crate::error::{self, Error, Result};
use crate::rphetools_traits::TableCell;

impl Error {
    fn malformed_label(label: &str) -> Self {
        Error::MalformedLabel {
            label: label.to_string(),
        }
    }

    fn malformed_disease_label(label: &str) -> Self {
        Error::MalformedDiseaseLabel {
            label: label.to_string(),
        }
    }
}

/// A valid label does not begin with or end with a white space
/// Valid labels also may not contain /,\, (,  ), or perdiod (".").
fn check_white_space(value: &str) -> Result<()> {
    if value.chars().last().map_or(false, |c| c.is_whitespace()) {
        return Err(Error::trailing_ws(value));
    } else if value.chars().next().map_or(false, |c| c.is_whitespace()) {
        return Err(Error::leading_ws(value));
    } else {
        Ok(())
    }
}

fn check_forbidden_chars(value: &str) -> Result<()> {
    let forbidden_chars: HashSet<char> = ['/', '\\', '(', ')', '.'].iter().copied().collect();
    match value.chars().find(|&c| forbidden_chars.contains(&c)) {
        Some(fc) => Err(Error::forbidden_character(fc, value)),
        None => Ok(()),
    }
}

#[derive(Debug)]

pub struct SimpleLabel {
    label: String,
}

impl TableCell for SimpleLabel {
    fn value(&self) -> String {
        self.label.clone()
    }
}

impl SimpleLabel {
    pub fn individual_id(value: &str) -> Result<Self> {
        if value.is_empty() {
            return Err(Error::EmptyLabel);
        }
        check_forbidden_chars(value)?;
        check_white_space(value)?;
        return Ok(SimpleLabel {
            label: value.to_string(),
        });
    }

    pub fn disease_label(value: &str) -> Result<Self> {
        if value.is_empty() {
            return Err(Error::EmptyLabel);
        }
        check_forbidden_chars(value)?;
        check_white_space(value)?;
        return Ok(SimpleLabel {
            label: value.to_string(),
        });
    }

    pub fn gene_symbol(value: &str) -> Result<Self> {
        let valid_curie = check_white_space(value);
        if valid_curie.is_err() {
            return Err(Error::malformed_label(value));
        } else {
            return Ok(SimpleLabel {
                label: value.to_string(),
            });
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_individual_id_ctor() {
        let tests = vec![
            ("proband", "proband"),
            ("individual II:3", "individual II:3"),
            (
                "patient (II:2)",
                "Forbidden character '(' found in label 'patient (II:2)'",
            ),
            (
                "individual II/3",
                "Forbidden character '/' found in label 'individual II/3'",
            ),
            (
                "individual II\\3",
                "Forbidden character '\\' found in label 'individual II\\3'",
            ),
            ("", "Empty label"),
        ];
        for test in tests {
            let individual_id = SimpleLabel::individual_id(test.0);
            match individual_id {
                Ok(id) => assert_eq!(test.1, id.value()),
                Err(err) => assert_eq!(test.1, err.to_string()),
            }
        }
    }

    #[test]
    fn test_disease_label_ctor() {
        let tests = vec![
            ("Marfan syndrome", "Marfan syndrome"),
            (
                "Marfan syndrome(",
                "Forbidden character '(' found in label 'Marfan syndrome('",
            ),
            (
                "Marfan/syndrome",
                "Forbidden character '/' found in label 'Marfan/syndrome'",
            ),
            (
                "Marfan syndrome ",
                "Trailing whitespace in 'Marfan syndrome '",
            ),
            ("", "Empty label"),
        ];
        for test in tests {
            let disease_label = SimpleLabel::disease_label(test.0);
            match disease_label {
                Ok(id) => assert_eq!(test.1, id.value()),
                Err(err) => assert_eq!(test.1, err.to_string()),
            }
        }
    }
}
