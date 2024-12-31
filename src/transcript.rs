use crate::individual_template::TableCell;


/// We use this function to check that transcripts end with a version
fn ends_with_period_and_number(s: &str) -> bool {
    if let Some((before_last, last)) = s.rsplit_once('.') {
        return !before_last.is_empty() && last.chars().all(|c| c.is_ascii_digit());
    }
    false
}


pub struct Transcript {
    value: String,
}

impl TableCell for Transcript {
    fn value(&self) -> String {
        self.value.clone()
    }
}

impl Transcript {
    pub fn new(val: &str) -> Result<Self, String> {
        if val.starts_with("ENST") || val.starts_with("NM_") {
            let valid_version = ends_with_period_and_number(val);
            if valid_version {
                return Ok(Transcript{value: val.to_string()});
            } else {
                return Err(format!("Invalid version: {}", val));
            }
        } 
        Err(format!("Transcript starts with unrecognized prefix: '{}'", val))
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_transcripts() {
        let tests = vec![
            ("NM_006139.4", "NM_006139.4"),
            ("NM_006139", "Invalid version: NM_006139"),
            ("NM006139.4", "Transcript starts with unrecognized prefix: 'NM006139.4'"),
            ("ENST00000316623.10", "ENST00000316623.10")
        ];
        for test in tests {
            match Transcript::new(test.0) {
                Ok(id) => assert_eq!(test.1, id.value()),
                Err(err) => assert_eq!(test.1, err)
            };
        }
    }


}