

use std::collections::HashMap;

use crate::{dto::cohort_dto::{DiseaseData, ModeOfInheritance}, hpoa::counted_hpo_term::CountedHpoTerm};
use once_cell::sync::Lazy;


/// Valid Mode of inheritance terms that can be used for outputting HPOA files
pub static VALID_MODES_OF_INHERITANCE: Lazy<HashMap<String, String>> = Lazy::new(|| {
    let mut moi_map: HashMap<String, String> = HashMap::new();
    let moi_terms = [
        ("HP:0000006", "Autosomal dominant inheritance"), 
        ("HP:0000007", "Autosomal recessive inheritance"),
        ("HP:0001417", "X-linked inheritance"),
        ("HP:0001423", "X-linked dominant inheritance"),
        ("HP:0001419", "X-linked recessive inheritance"),
        ("HP:0001427", "Mitochondrial inheritance"),
        ("HP:0010984", "Digenic inheritance"),
        ("HP:0001450", "Y-linked inheritance"),
        ("HP:0034340", "Pseudoautosomal dominant inheritance"),
        ("HP:0034341", "Pseudoautosomal recessive inheritance"),
        ];
    for tpl in moi_terms {
        moi_map.insert(tpl.0.to_string(), tpl.1.to_string());
    }
    moi_map
});






/// The default frequency is the empty string
/// In the HPOA context, this is taken to mean 100%
/// Here this is used for the mode of inheritance rows, which do 
/// not have frequencies in the HPOA format.
const DEFAULT_FREQ: &str = "";

pub struct HpoaTableRow {
    disease_id: String,
    disease_name:String,
    phenotype_id: String,
    phenotype_name: String,
    onset_id: String,
    onset_name: String,
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
        disease: &DiseaseData, 
        term_id: &str,
        term_label: &str, 
        freq_string: &str,
        pmid: &str,
        biocurator: &str) -> Result<Self, String> {
        Ok(Self { 
            disease_id: disease.disease_id.to_string(), 
            disease_name: disease.disease_label.to_string(), 
            phenotype_id: term_id.to_string(), 
            phenotype_name: term_label.to_string(), 
            onset_id: "".to_string(), 
            onset_name: "".to_string(), 
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


    pub fn from_counted_term(disease: &DiseaseData, cterm: CountedHpoTerm, biocurator: &str) -> Result<Self, String> {
        HpoaTableRow::new(disease, cterm.hpo_id(), cterm.hpo_label(), &cterm.freq_string(), cterm.pmid(), biocurator)
    }


    pub fn header_fields() -> Vec<String> {
        vec![
            "#diseaseID".to_string(),
            "diseaseName".to_string(),
            "phenotypeID".to_string(),
            "phenotypeName".to_string(),
            "onsetID".to_string(),
            "onsetName".to_string(),
            "frequency".to_string(),
            "sex".to_string(),
            "negation".to_string(),
            "modifier".to_string(),
            "description".to_string(),
            "publication".to_string(),
            "evidence".to_string(),
            "biocuration".to_string()
            ]
    }

    pub fn row(&self) -> Vec<String> {
        let fields = vec![
            self.disease_id.clone(),
            self.disease_name.clone(),
            self.phenotype_id.clone(),
            self.phenotype_name.clone(),
            self.onset_id.clone(),
            self.onset_name.clone(),
            self.frequency.clone(),
            self.sex.clone(),
            self.negation.clone(),
            self.modifier.clone(),
            self.description.clone(),
            self.publication.clone(),
            self.evidence.clone(),
            self.biocuration.clone(),
        ];
        fields
    }

    pub fn from_moi(
        disease: &DiseaseData,
        moi: &ModeOfInheritance,
        biocurator: &str
    ) -> Result<Self, String> {
        if VALID_MODES_OF_INHERITANCE
            .get(&moi.hpo_id)
            .map(|s| s == &moi.hpo_label)
            .unwrap_or(false)
        {
             HpoaTableRow::new(disease, &moi.hpo_id, &moi.hpo_label, DEFAULT_FREQ, &moi.citation, biocurator)
        } else {
            Err(format!("Invalid mode of inheritance data: '{}' / '{}' ", moi.hpo_label, moi.hpo_id))
        }
    }

}


#[cfg(test)]
mod tests {
    use super::*;
    use rstest::{fixture, rstest};

    #[fixture]
    fn disease_data() -> DiseaseData {
        DiseaseData { 
            disease_id: "OMIM:607616".to_string(), 
            disease_label: "Niemann-Pick disease, type B".to_string(), 
            mode_of_inheritance_list: vec![], 
            gene_transcript_list: vec![]
        }
    }

    #[rstest]
    #[case("Y-linked inheritance", "HP:0001450")]
    #[case("Autosomal recessive inheritance", "HP:0000007")]
    #[case("Pseudoautosomal recessive inheritance", "HP:0034341")]
    fn test_valid_moi(#[case] label: &str, #[case] hpo_id: &str, disease_data: DiseaseData) {
        let biocurator = "0000-0000-0000-0001";
        let pmid = "PMID:123".to_string();
        let moi = ModeOfInheritance { 
            hpo_id: hpo_id.to_string(), 
            hpo_label: label.to_string(), 
            citation: pmid 
        };
        let result = HpoaTableRow::from_moi(&disease_data, &moi, biocurator);
        assert!(result.is_ok());
        let hpoa_row = result.unwrap();
        assert_eq!(hpo_id, hpoa_row.phenotype_id);
        assert_eq!(label, hpoa_row.phenotype_name);
        assert_eq!(disease_data.disease_id, hpoa_row.disease_id);
        assert_eq!(disease_data.disease_label, hpoa_row.disease_name);
        assert_eq!(biocurator, hpoa_row.biocuration);

    }

    /// Test we get an error if the user provides an invalid HPO id or label
    #[rstest]
    #[case("Y-linked", "HP:0001450")]
    #[case("Autosomal recessive", "HP:0000007")]
    #[case("Autosomal recessive inheritance", "HP:1234567")]
    fn tes_invalid_moi(#[case] label: &str, #[case] hpo_id: &str, disease_data: DiseaseData) {
        let biocurator = "0000-0000-0000-0001";
        let pmid = "PMID:123".to_string();
        let moi = ModeOfInheritance { 
            hpo_id: hpo_id.to_string(), 
            hpo_label: label.to_string(), 
            citation: pmid 
        };
        let result = HpoaTableRow::from_moi(&disease_data, &moi, biocurator);
        assert!(result.is_err());
    }


}
