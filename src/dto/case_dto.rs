use serde::{de, Deserialize, Serialize};

/// All information about a case (phenopacket) except for the HPO terms, which will be 
/// transmitted in a separate step
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CaseDto {
    /// PubMed identifier
    pub pmid: String,
    /// title of the corresponding article
    pub title: String,
    /// identifier of the individual within the article
    pub individual_id: String,
    /// optional comment
    pub comment: String,
    /// First pathogenic allele
    pub allele_1: String,
    /// Second pathogenic allele (or "na" if there is only one)
    pub allele_2: String,
    /// Optional comment about the alleles
    pub variant_comment: String,
    /// Age string representing onset of the disease
    pub age_of_onset: String,
    /// Age string representing age when patient last seen in a medical encounter
    pub age_at_last_encounter: String,
    /// yes/no/na 
    pub deceased: String,
    /// M:F:O:U (male, female, other, unknown)
    pub sex: String
}

impl CaseDto {
    pub fn new(
         pmid: impl Into<String>,
         title: impl Into<String>,
         individual_id: impl Into<String>,
         comment: impl Into<String>,
         allele_1: impl Into<String>,
         allele_2: impl Into<String>,
         variant_comment: impl Into<String>,
         age_of_onset: impl Into<String>,
         age_at_last_encounter: impl Into<String>,
         deceased: impl Into<String>,
         sex: impl Into<String>,
    ) -> Self
     {
        Self {
            pmid: pmid.into(),
            title: title.into(),
            individual_id: individual_id.into(),
            comment: comment.into(),
            allele_1: allele_1.into(),
            allele_2: allele_2.into(),
            variant_comment: variant_comment.into(),
            age_of_onset: age_of_onset.into(),
            age_at_last_encounter: age_at_last_encounter.into(),
            deceased: deceased.into(),
            sex: sex.into()
        }
    }

    pub fn pmid(&self) -> &str {
        &self.pmid
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn individual_id(&self) -> &str {
        &self.individual_id
    }

    pub fn comment(&self) -> &str {
        &self.comment
    }

    pub fn allele_1(&self) -> &str {
        &self.allele_1
    }

    pub fn allele_2(&self) -> &str {
        &self.allele_2
    }

    pub fn variant_comment(&self) -> &str {
        &self.variant_comment
    }

    pub fn age_of_onset(&self) -> &str {
        &self.age_of_onset
    }

    pub fn age_at_last_encounter(&self) -> &str {
        &self.age_at_last_encounter
    }

    pub fn deceased(&self) -> &str {
        &self.deceased
    }

    pub fn sex(&self) -> &str {
        &self.sex
    }

    /// return the first four columns needed for a phenopacket row
    pub fn individual_values(&self) -> Vec<String> {
        vec![self.pmid.to_string(), self.title.to_string(), self.individual_id.to_string(), self.comment.to_string()]
    }

     /// return the last seven columns of the constant part needed for a phenopacket row
    pub fn variant_values(&self) -> Vec<String> {
        vec![self.allele_1.to_string(), self.allele_2.to_string(), self.variant_comment.to_string(),
            self.age_of_onset.to_string(), self.age_at_last_encounter.to_string(), self. deceased.to_string(), self.sex.to_string()]
    }
    
}


