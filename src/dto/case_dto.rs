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
}


