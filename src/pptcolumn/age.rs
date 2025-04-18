use regex::Regex;
use std::collections::HashMap;

use crate::error::{self, Error, Result};
use crate::rphetools_traits::TableCell;

impl Error {
    fn age_parse_error<T>(val: T) -> Self
    where
        T: Into<String>,
    {
        Error::AgeParseError { msg: val.into() }
    }
}

/// We offer a simple HPO implementation that checks validity of individual Term identifiers and labels
/// We may also implement a version that keeps track of the Ontology object to perform other checks in the future TODO
pub trait AgeTrait {
    // Define methods that types implementing the trait must provide
    fn age_string(&self) -> String;
}

#[derive(Clone, Debug)]
pub struct GestationalAge {
    weeks: u32,
    days: u32,
    age_string: String,
}

impl AgeTrait for GestationalAge {
    fn age_string(&self) -> String {
        self.age_string.clone()
    }
}

impl GestationalAge {
    pub fn new<S: Into<String>>(w: u32, d: u32, agestring: S) -> Self {
        GestationalAge {
            weeks: w,
            days: d,
            age_string: agestring.into(),
        }
    }
    pub fn weeks(&self) -> u32 {
        self.weeks
    }

    pub fn days(&self) -> u32 {
        self.days
    }
}

#[derive(Clone, Debug)]
pub struct Iso8601Age {
    years: u32,
    months: u32,
    days: u32,
    age_string: String,
}

impl Iso8601Age {
    pub fn new<S: Into<String>>(years: u32, months: u32, days: u32, agestring: S) -> Self {
        Iso8601Age {
            years: years,
            months: months,
            days: days,
            age_string: agestring.into(),
        }
    }
    pub fn years(&self) -> u32 {
        self.years
    }

    pub fn months(&self) -> u32 {
        self.months
    }

    pub fn days(&self) -> u32 {
        self.days
    }
}
impl AgeTrait for Iso8601Age {
    fn age_string(&self) -> String {
        self.age_string.clone()
    }
}

#[derive(Debug, Clone)]
pub struct HpoTermAge {
    term_id: String,
    label: String,
}

impl HpoTermAge {
    pub fn new<T: Into<String>, U: Into<String>>(tid: T, lab: U) -> Self {
        HpoTermAge {
            term_id: tid.into(),
            label: lab.into(),
        }
    }

    pub fn term_id(&self) -> String {
        self.term_id.clone()
    }

    pub fn label(&self) -> String {
        self.label.clone()
    }
}

impl AgeTrait for HpoTermAge {
    fn age_string(&self) -> String {
        return self.label.clone();
    }
}

/// It is valid to put "na" in the Age column. We represent this is NaAge

#[derive(Clone, Debug)]
pub struct NaAge;

impl AgeTrait for NaAge {
    fn age_string(&self) -> String {
        return "na".to_string();
    }
}
impl AgeTrait for Age {
    fn age_string(&self) -> String {
        match self {
            Age::Gestational(ga) => ga.age_string(),
            Age::HpoTerm(ht) => ht.age_string(),
            Age::Iso8601(iso) => iso.age_string(),
            Age::NaAge(na) => na.age_string(),
        }
    }
}

pub trait AgeToolTrait {
    fn parse(&self, cell_value: &str) -> Result<Age>;
}

#[derive(Clone, Debug)]
pub enum Age {
    Gestational(GestationalAge),
    HpoTerm(HpoTermAge),
    Iso8601(Iso8601Age),
    NaAge(NaAge),
}

impl TableCell for Age {
    fn value(&self) -> String {
        match &self {
            Age::Gestational(ga) => ga.age_string(),
            Age::HpoTerm(h) => h.age_string(),
            Age::Iso8601(iso) => iso.age_string(),
            Age::NaAge(na) => na.age_string(),
        }
    }
}

pub struct AgeTool {
    hpo_label_to_age_term_d: HashMap<String, String>,
}

impl AgeTool {
    pub fn new() -> Self {
        AgeTool {
            hpo_label_to_age_term_d: Self::create_age_term_d(),
        }
    }

    /// Create a dictionary with all HPO Age of onset terms
    fn create_age_term_d() -> HashMap<String, String> {
        let mut age_term_d: HashMap<String, String> = HashMap::new();
        let onset_tuples = [
            ("HP:0003584", "Late onset"),
            ("HP:0003596", "Middle age onset"),
            ("HP:0011462", "Young adult onset"),
            ("HP:0025710", "Late young adult onset"),
            ("HP:0025709", "Intermediate young adult onset"),
            ("HP:0025708", "Early young adult onset"),
            ("HP:0003581", "Adult onset"),
            ("HP:0003621", "Juvenile onset"),
            ("HP:0011463", "Childhood onset"),
            ("HP:0003593", "Infantile onset"),
            ("HP:0003623", "Neonatal onset"),
            ("HP:0003577", "Congenital onset"),
            ("HP:0030674", "Antenatal onset"),
            ("HP:0011460", "Embryonal onset"),
            ("HP:0011461", "Fetal onset"),
            ("HP:0034199", "Late first trimester onset"),
            ("HP:0034198", "Second trimester onset"),
            ("HP:0034197", "Third trimester onset"),
        ];
        for tup in onset_tuples {
            age_term_d.insert(tup.1.to_string(), tup.0.to_string());
        }
        return age_term_d;
    }
}

impl AgeToolTrait for AgeTool {
    fn parse(&self, cell_value: &str) -> Result<Age> {
        if cell_value.starts_with("P") {
            let iso8601_re = Regex::new(r"^P(?:(\d+)Y)?(?:(\d+)M)?(?:(\d+)D)?$").unwrap();
            if let Some(captures) = iso8601_re.captures(cell_value) {
                let mut years: u32 = 0;
                let mut months: u32 = 0;
                let mut days: u32 = 0;
                if let Some(yearmatch) = captures.get(1) {
                    if let Some(y) = yearmatch.as_str().parse::<u32>().ok() {
                        years = y;
                    }
                }
                if let Some(monthmatch) = captures.get(2) {
                    if let Some(m) = monthmatch.as_str().parse::<u32>().ok() {
                        months = m;
                    }
                }
                if let Some(daymatch) = captures.get(3) {
                    if let Some(d) = daymatch.as_str().parse::<u32>().ok() {
                        days = d;
                    }
                }
                let iso_age = Iso8601Age::new(years, months, days, cell_value);
                return Ok(Age::Iso8601(iso_age));
            } else {
                return Err(Error::age_parse_error(format!(
                    "Input string '{}' starts with P but was not valid ISO8601 period",
                    cell_value
                )));
            }
        } else if self.hpo_label_to_age_term_d.contains_key(cell_value) {
            let onset_id = self
                .hpo_label_to_age_term_d
                .get(cell_value)
                .expect("Could not retrieve SimpleTerm for onset");
            let hponset = HpoTermAge::new(onset_id, cell_value);
            return Ok(Age::HpoTerm(hponset));
        } else if cell_value.contains("+") {
            let gestational_re = Regex::new(r"(\d+)\+([0-6])").unwrap();
            if let Some(captures) = gestational_re.captures(cell_value) {
                let mut weeks: u32 = 0;
                let mut days: u32 = 0;
                if let Some(weekmatch) = captures.get(1) {
                    if let Some(w) = weekmatch.as_str().parse::<u32>().ok() {
                        weeks = w;
                    }
                }
                if let Some(daymatch) = captures.get(2) {
                    if let Some(d) = daymatch.as_str().parse::<u32>().ok() {
                        days = d;
                    }
                }
                let ga = GestationalAge::new(weeks, days, cell_value);
                return Ok(Age::Gestational(ga));
            } else {
                return Err(Error::age_parse_error(format!(
                    "Could not parse '{}' as gestational age",
                    cell_value
                )));
            }
        }
        return Err(Error::age_parse_error(format!(
            "Could not parse '{}' as Age",
            cell_value
        )));
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_hpo_term_age() {
        let tests = vec![
            ("HP:0003596", "Middle age onset"),
            ("HP:0011462", "Young adult onset"),
            ("HP:0025710", "Late young adult onset"),
            ("HP:0025709", "Intermediate young adult onset"),
            ("HP:0025708", "Early young adult onset"),
            ("HP:0003581", "Adult onset"),
            ("HP:0003621", "Juvenile onset"),
            ("HP:0011463", "Childhood onset"),
            ("HP:0003593", "Infantile onset"),
            ("HP:0003623", "Neonatal onset"),
            ("HP:0003577", "Congenital onset"),
            ("HP:0030674", "Antenatal onset"),
            ("HP:0011460", "Embryonal onset"),
            ("HP:0011461", "Fetal onset"),
            ("HP:0034199", "Late first trimester onset"),
            ("HP:0034198", "Second trimester onset"),
            ("HP:0034197", "Third trimester onset"),
        ];
        let parser = AgeTool::new();
        for test in tests {
            let result = parser.parse(test.1);
            match result {
                Ok(age) => match age {
                    Age::Gestational(_) => {
                        assert!(false, "Not expecting Gestational Age here");
                    }
                    Age::HpoTerm(hpo_term_age) => {
                        assert_eq!(test.1, hpo_term_age.age_string());
                    }
                    Age::Iso8601(_) => {
                        assert!(false, "Not expecting Iso8601 Age here");
                    }
                    Age::NaAge(_) => {
                        assert!(false, "Not expecting na age here");
                    }
                },
                Err(e) => {
                    assert!(false, "Not expecting error in this test: {}", e);
                }
            }
        }
    }

    #[test]
    fn test_hpo_term_age_malformed() {
        let tests = vec![
            (
                "HP:0003596",
                "Middle age onst",
                "Could not parse 'Middle age onst' as Age",
            ),
            (
                "HP:0003623",
                "Neontal onset",
                "Could not parse 'Neontal onset' as Age",
            ),
        ];
        let parser = AgeTool::new();
        for test in tests {
            let result = parser.parse(test.1);
            assert!(result.is_err());
            assert_eq!(test.2, result.err().unwrap().to_string());
        }
    }

    #[test]
    fn test_iso_age() {
        let tests: Vec<(&str, Box<u32>, Box<u32>, Box<u32>)> = vec![
            ("P3Y2M", Box::new(3), Box::new(2), Box::new(0)),
            ("P3Y7M", Box::new(3), Box::new(7), Box::new(0)),
            ("P3Y7M21D", Box::new(3), Box::new(7), Box::new(21)),
            ("P33Y", Box::new(33), Box::new(0), Box::new(0)),
            ("P3D", Box::new(0), Box::new(0), Box::new(3)),
        ];
        for test in tests {
            let parser = AgeTool::new();
            let result = parser.parse(test.0);
            match result {
                Ok(age) => match age {
                    Age::Gestational(_) => {
                        assert!(false, "Not expecting gestational age here")
                    }
                    Age::HpoTerm(_) => {
                        assert!(false, "Not expecting HpoTerm age here")
                    }
                    Age::Iso8601(iso8601_age) => {
                        assert_eq!(*test.1, iso8601_age.years());
                        assert_eq!(*test.2, iso8601_age.months());
                        assert_eq!(*test.3, iso8601_age.days());
                    }
                    Age::NaAge(_) => {
                        assert!(false, "Not expecting na age here")
                    }
                },
                Err(e) => {
                    assert!(false, "Not expecting error in this test: {}", e);
                }
            }
        }
    }

    #[test]
    fn test_iso_age_malformed() {
        let tests: Vec<(&str, &str)> = vec![
            (
                "P3Y2",
                "Input string 'P3Y2' starts with P but was not valid ISO8601 period",
            ),
            (
                "P3YM",
                "Input string 'P3YM' starts with P but was not valid ISO8601 period",
            ),
            ("3YM2", "Could not parse '3YM2' as Age"),
        ];
        for test in tests {
            let parser = AgeTool::new();
            let result = parser.parse(test.0);
            match result {
                Ok(age) => match age {
                    Age::Gestational(_) => {
                        assert!(false, "Not expecting Gestational Age here")
                    }
                    Age::HpoTerm(_) => {
                        assert!(false, "Not expecting HpoTerm Age here")
                    }
                    Age::Iso8601(_) => {
                        assert!(false, "Not expecting Iso8601 Age here")
                    }
                    Age::NaAge(_) => {
                        assert!(false, "Not expecting na age here")
                    }
                },
                Err(e) => {
                    assert_eq!(test.1, e.to_string());
                }
            }
        }
    }

    #[test]
    fn test_gestational_age() {
        let tests: Vec<(&str, Box<u32>, Box<u32>)> = vec![
            ("37+2", Box::new(37), Box::new(2)),
            ("24+0", Box::new(24), Box::new(0)),
        ];
        for test in tests {
            let parser = AgeTool::new();
            let result = parser.parse(test.0);
            match result {
                Ok(age) => match age {
                    Age::Gestational(gestational_age) => {
                        assert_eq!(*test.1, gestational_age.weeks());
                        assert_eq!(*test.2, gestational_age.days());
                    }
                    Age::HpoTerm(_) => {
                        assert!(false, "Not expecting HpoTerm Age here")
                    }
                    Age::Iso8601(_) => {
                        assert!(false, "Not expecting Iso8601 Age here")
                    }
                    Age::NaAge(_) => {
                        assert!(false, "Not expecting na age here")
                    }
                },
                Err(e) => {
                    assert!(false, "Not expecting error in this test: {}", e);
                }
            }
        }
    }
}
