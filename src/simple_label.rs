use std::collections::HashSet;

use crate::rphetools_traits::TableCell;
use crate::error::{self, Error, Result};



impl Error {

    fn forbidden_char(val: String) -> Self {
        Error::ForbiddenLabelChar { msg: val }
    }

    fn malformed_label(label: &str) -> Self {
        Error::MalformedLabel { label: label.to_string() }
    }

    fn malformed_disease_label(label: &str) -> Self {
        Error::MalformedDiseaseLabel { label: label.to_string() }
    }

}


/// A valid label does not begin with or end with a white space
/// Valid labels also may not contain /,\, (,  ), or perdiod (".").
fn check_valid_label(value: &str) -> Result<bool> {
    let forbidden_chars: HashSet<char> = ['/', '\\', '(', ')', '.'].iter().copied().collect();
    if value.is_empty() {
        return Err(Error::EmptyLabel);
    } else if let Some(forbidden) = value.chars().find(|&c| forbidden_chars.contains(&c)) {
        return Err(Error::forbidden_char(format!("Forbidden character '{}' found: '{}'", forbidden, value)));
    } else if value.chars().last().map_or(false, |c| c.is_whitespace()) {
        return Err(Error::malformed_label(value));
    } else if value.chars().next().map_or(false, |c| c.is_whitespace()) {
        return Err(Error::malformed_label(value));
    } else {
        Ok(true)
    }
}



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
        let valid_curie = check_valid_label(value);
        if valid_curie.is_err() {
            return Err(Error::malformed_label(value));
        }  else {
            return Ok(SimpleLabel { label: value.to_string(), });
        }
    }

    pub fn disease_label(value: &str) -> Result<Self> {
        let valid_curie = check_valid_label(value);
        if valid_curie.is_err() {
            return Err(Error::malformed_disease_label(value));
        }  else {
            return Ok(SimpleLabel { label: value.to_string(), });
        }
    }

    pub fn gene_symbol(value: &str) -> Result<Self> {
        let valid_curie = check_valid_label(value);
        if valid_curie.is_err() {
            return Err(Error::malformed_label(value));
        }  else {
            return Ok(SimpleLabel { label: value.to_string(), });
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
            ("patient (II:2)", "Invalid individual_id: Forbidden character '(' found: 'patient (II:2)'"),
            ("individual II/3", "Invalid individual_id: Forbidden character '/' found: 'individual II/3'"),
            ("individual II\\3", "Invalid individual_id: Forbidden character '\\' found: 'individual II\\3'"),
            ("", "Invalid individual_id: is empty")

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
            ("Marfan syndrome(", "Invalid disease label: Forbidden character '(' found: 'Marfan syndrome('"),
            ("Marfan/syndrome", "Invalid disease label: Forbidden character '/' found: 'Marfan/syndrome'"),
            ("Marfan syndrome ", "Invalid disease label: 'Marfan syndrome ' ends with whitespace"),
            ("", "Invalid disease label: is empty")

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