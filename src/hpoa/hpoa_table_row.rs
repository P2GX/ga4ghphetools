use ontolius::{term::{simple::SimpleMinimalTerm, MinimalTerm}, Identified};

use crate::dto::template_dto::DiseaseDto;



pub struct HpoaTableRow {
    diseaseID: String,
    diseaseName:String,
    phenotypeID: String,
    phenotypeName: String,
    onsetID: String,
    onsetName: String,
    frequency: String,
    sex: String,
    negation : String,
    modifier: String,
    description: String,
    publication: String,
    evidence: String,
    biocuration: String
}

impl HpoaTableRow {
    pub fn new(
        disease: &DiseaseDto, 
        term_id: &str,
        term_label: &str, 
        freq_string: &str,
        pmid: &str,
        biocurator: &str) -> Result<Self, String> {
        Ok(Self { 
            diseaseID: disease.disease_id.to_string(), 
            diseaseName: disease.disease_label.to_string(), 
            phenotypeID: term_id.to_string(), 
            phenotypeName: term_label.to_string(), 
            onsetID: "".to_string(), 
            onsetName: "".to_string(), 
            frequency: freq_string.to_string(), 
            sex: "".to_string(), 
            negation: "".to_string(), 
            modifier: "".to_string(), 
            description: "".to_string(), 
            publication: pmid.to_string(), 
            evidence: "PCS".to_string(), 
            biocuration: biocurator.to_string() 
        })
    }
}

