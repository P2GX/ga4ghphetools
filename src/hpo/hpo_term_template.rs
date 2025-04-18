use crate::error::{self, Error, Result};
use crate::hpo::onset::Onset;

#[derive(Clone, Debug)]
pub enum HpoTermStatus {
    Observed,
    Excluded,
    NotAvailable,
    Mild,
    Moderate,
    Severe,
    Onset(Onset),
}

#[derive(Debug)]
pub struct HpoTemplate {
    hpo_id: String,
    label: String,
    status: HpoTermStatus,
}

impl HpoTemplate {
    pub fn new(id: &str, label: &str, status: HpoTermStatus) -> Self {
        HpoTemplate {
            hpo_id: id.to_string(),
            label: label.to_string(),
            status: status.clone(),
        }
    }
}

#[derive(Debug)]
pub struct HpoTemplateFactory {
    hpo_id: String,
    label: String,
}

impl HpoTemplateFactory {
    /// take ownership of the string in the HeaderDuplet
    pub fn new(header1: &str, header2: &str) -> Self {
        HpoTemplateFactory {
            hpo_id: header1.to_string(),
            label: header2.to_string(),
        }
    }

    fn with_status(&self, status: HpoTermStatus) -> Result<HpoTemplate> {
        Ok(HpoTemplate {
            hpo_id: self.hpo_id.clone(),
            label: self.label.clone(),
            status: status.clone(),
        })
    }

    pub fn from_cell_value(&self, val: &str) -> Result<HpoTemplate> {
        match val {
            "observed" => self.with_status(HpoTermStatus::Observed),
            "excluded" => self.with_status(HpoTermStatus::Excluded),
            "na" => self.with_status(HpoTermStatus::NotAvailable),
            "Mild" => self.with_status(HpoTermStatus::Mild),
            "Moderate" => self.with_status(HpoTermStatus::Moderate),
            "Severe" => self.with_status(HpoTermStatus::Severe),
            other => {
                let ons = Onset::new(other);
                self.with_status(HpoTermStatus::Onset(ons))
            }
        }
    }
}
