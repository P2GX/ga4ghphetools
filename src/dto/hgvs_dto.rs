

use serde::{Deserialize, Serialize};



/// A Data Transfer Object for information about an HGVS Variant that we want to validate.
///
/// These are "Small" variants, such as single nucleotide variants, that are represented with Human Genome Variation Society (HGVS) nomenclature, e.g., c. 123G>T
/// The validated field represents whether we have validated the variant with Variant Validator.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct HgvsDto {
    pub hgvs: String,
    pub transcript: String,
    pub validated: bool
}



impl HgvsDto {
    pub fn new(
        hgvs: impl Into<String>,
        transcript: impl Into<String>,
        validated: bool
    ) -> Self {
        Self { 
            hgvs: hgvs.into(), 
            transcript: transcript.into(), 
            validated: validated
        }
    }
}