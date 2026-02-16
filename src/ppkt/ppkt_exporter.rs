//! Module to export GA4GH Phenopackets from the information in the template.


use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use ontolius::{Identified, TermId};
use ontolius::ontology::csr::FullCsrOntology;
use ontolius::ontology::{MetadataAware, OntologyTerms};
use phenopacket_tools::builders::time_elements::time_element_from_str;
use phenopackets::schema::v2::core::{KaryotypicSex, OntologyClass};
use phenopackets::schema::v2::core::vital_status::Status;
use phenopackets::schema::v2::core::{Disease, ExternalReference, Individual, MetaData, PhenotypicFeature, Sex, VitalStatus};
use phenopackets::schema::v2::Phenopacket;
use regex::Regex;
use serde_json::Value;
use crate::dto::cohort_dto::{CohortData, DiseaseData, RowData};
use crate::dto::hpo_term_dto::HpoTermDuplet;
use crate::ppkt::ppkt_variant_exporter::PpktVariantExporter;
use phenopacket_tools;
use phenopacket_tools::builders::builder::Builder;


const DEFAULT_HGNC_VERSION: &str =  "06/01/25";
const DEFAULT_OMIM_VERSION: &str =  "06/01/25";
const DEFAULT_GENO_VERSION: &str =  "2025-07-25";


/// Structure to export phenopackets from a CohortData object.
pub struct PpktExporter {
    /// Reference to the Ontolius Human Phenotype Ontology Full CSR object
    hpo: Arc<FullCsrOntology>,
    geno_version: String,
    omim_version: String,
    hgnc_version: String,
    orcid_id: String,
    cohort_dto: CohortData,
    disease_id_map: HashMap<String, DiseaseData>,
}

impl PpktExporter {


    pub fn new( 
        hpo: Arc<FullCsrOntology>,
        creator_orcid: &str,
        cohort: CohortData
    ) -> Self {
        Self::from_versions(
            hpo,
            DEFAULT_GENO_VERSION,
            DEFAULT_OMIM_VERSION,
            DEFAULT_HGNC_VERSION,
            creator_orcid,
            cohort)
    }

    pub fn from_versions(
        hpo: Arc<FullCsrOntology>,
        geno_version: &str,
        omim_version: &str, 
        hgnc_version: &str ,
        creator_orcid: &str,
        cohort: CohortData
    ) -> Self {
        let mut disease_map = HashMap::new();
        for d in &cohort.disease_list {
            disease_map.insert(d.disease_id.clone(), d.clone());
        }
        Self{ 
            hpo, 
            geno_version: geno_version.to_string(),
            omim_version: omim_version.to_string(), 
            hgnc_version: hgnc_version.to_string(),
            orcid_id: creator_orcid.to_string(),
            cohort_dto: cohort,
            disease_id_map: disease_map,
        }
    }


    /// Create a GA4GH Individual message
    pub fn extract_individual(&self, ppkt_row: &RowData) -> Result<Individual, String> {
        let individual_dto = &ppkt_row.individual_data;
        let mut idvl = Individual{ 
            id: individual_dto.individual_id.clone(), 
            alternate_ids: vec![], 
            date_of_birth: None, 
            time_at_last_encounter: None, 
            vital_status: None, 
            sex: Sex::UnknownSex.into(), 
            karyotypic_sex: KaryotypicSex::UnknownKaryotype.into(), 
            gender: None, 
            taxonomy: None };
        match individual_dto.sex.as_ref() {
            "M" => idvl.sex = Sex::Male.into(),
            "F" => idvl.sex = Sex::Female.into(),
            "O" => idvl.sex = Sex::OtherSex.into(),
            "U" => idvl.sex = Sex::UnknownSex.into(),
            _ => { return Err(format!("Did not recognize sex string '{}' for '{}' ({})", idvl.sex, idvl.id, ppkt_row.individual_data.pmid)); }
        };
        let last_enc = &individual_dto.age_at_last_encounter;
        if last_enc != "na" {
            let age = time_element_from_str(last_enc)
                .map_err(|e| format!("malformed time_element for last encounter '{}':{} for {}",last_enc, e.to_string(), idvl.id))?;
            idvl.time_at_last_encounter = Some(age);
        }
        if individual_dto.deceased == "yes" {
            idvl.vital_status = Some(VitalStatus{ 
                status: Status::Deceased.into(), 
                time_of_death: None, 
                cause_of_death: None, 
                survival_time_in_days: 0 
            });
        } 
        Ok(idvl)

    }

    pub fn hpo_version(&self) -> &str {
        &self.hpo.version()
    } 

    pub fn geno_version(&self) -> &str {
        &self.geno_version
    } 

    pub fn omim_version(&self) -> &str {
        &self.omim_version
    } 

    pub fn hgnc_version(&self) -> &str {
        &self.hgnc_version
    } 

    /// Create GA4GH MetaData object from version numbers using functions from phenopacket_tools
    pub fn get_meta_data(&self, row_dto: &RowData) -> Result<MetaData, String> {
        let created_by = self.orcid_id.clone();
        let mut meta_data = Builder::meta_data_now(created_by);
        let hpo = phenopacket_tools::builders::resources::Resources::hpo_version(self.hpo_version());
        let geno = phenopacket_tools::builders::resources::Resources::geno_version(self.geno_version());
        let omim = phenopacket_tools::builders::resources::Resources::omim_version(self.omim_version());
        let hgnc = phenopacket_tools::builders::resources::Resources::hgnc_version(&self.hgnc_version());
        let indvl_dto = row_dto.individual_data.individual_id.clone();
        let ext_res = ExternalReference{ 
            id: row_dto.individual_data.pmid.clone(), 
            reference: String::default(), 
            description: row_dto.individual_data.title.clone()
        };
        meta_data.resources.push(hpo);
        meta_data.resources.push(geno);
        meta_data.resources.push(omim);
        meta_data.resources.push(hgnc);
        meta_data.external_references.push(ext_res);
        Ok(meta_data)
    }


    /// Generate the phenopacket identifier from the PMID and the individual identifier
    pub fn get_phenopacket_id(&self, ppkt_row: &RowData) -> String {
        let individual_dto = &ppkt_row.individual_data;
        let pmid = ppkt_row.individual_data.pmid.replace(":", "_");
        let individual_id = individual_dto.individual_id.replace(" ", "_");
        let ppkt_id = format!("{}_{}", pmid, individual_id);
        let ppkt_id = ppkt_id.replace("__", "_");
        // Replace any non-ASCII characters with _, but remove trailing "_" if it exists.
        let mut sanitized: String = ppkt_id.chars()
            .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
            .clone().collect();
         // Collapse multiple underscores, if any
        let re = Regex::new(r"_+").unwrap();
        sanitized = re.replace_all(&sanitized, "_").to_string();
        if let Some(stripped) = sanitized.strip_suffix('_') {
            sanitized = stripped.to_string();
        }
        sanitized
    }

    pub fn get_disease_list(&self, ppkt_row: &RowData) -> Result<Vec<Disease>, String> {
        let disease_id_list = &ppkt_row.disease_id_list;
        if disease_id_list.is_empty() {
            return Err("No disease data found".to_string());
        }
        let has_multiple_dx = disease_id_list.len() > 1;
        let mut disease_list: Vec<Disease> = Vec::new();
        for dx_id in disease_id_list {
            let d_data = self.disease_id_map.get(dx_id)
                .ok_or_else(|| format!("Disease with id {} not found", dx_id))?;
            let dx_clz = OntologyClass { 
                id:d_data.disease_id.clone(), 
                label: d_data.disease_label.clone()
            };
            let mut disease = Disease{ 
                term: Some(dx_clz), 
                excluded: false, 
                onset: None, 
                resolution: None, 
                disease_stage: vec![], 
                clinical_tnm_finding: vec![], 
                primary_site: None, 
                laterality: None 
            };
            // If we have multiple diseases, we cannot automatically say when the disease onset was (which disease has the earliest onset)
            if ! has_multiple_dx {
                let idl_dto = ppkt_row.individual_data.individual_id.clone();
                let onset = &ppkt_row.individual_data.age_of_onset;
                if onset != "na" {
                    let age = time_element_from_str(onset)
                        .map_err(|e| format!("malformed time_element for onset '{}': {}", onset, e.to_string()))?;
                    disease.onset = Some(age);
                };
            }
            disease_list.push(disease);
        }
        Ok(disease_list)
    }

    fn allele_not_contained(allele: &str) -> String {
        format!("'{allele}' must be validated before exporting to Phenopacket Schema")
    }


    
    fn get_ontology_class(&self, term: &HpoTermDuplet) -> Result<OntologyClass, String> {
        let hpo_id = term.hpo_id();
        let hpo_label = term.hpo_label();
        let hpo_term_id = TermId::from_str(hpo_id).map_err(|e| e.to_string())?;
        let hpo_term = match self.hpo.term_by_id(&hpo_term_id) {
            Some(term) => term.clone(),
            None => {
                return Err(format!("Could not find HPO term for {hpo_id}"));
            }
        };
        if hpo_term.identifier() != &hpo_term_id {
            return Err(format!("{} is not the primary id ({}) for {}",
                hpo_term_id, hpo_term.identifier(), hpo_label));
        }
        let hpo_term = Builder::ontology_class(term.hpo_id(), term.hpo_label())
                .map_err(|e| format!("termid_parse_error '{:?}'", term))?;
        Ok(hpo_term)
    }

    pub fn get_phenopacket_features(&self, ppkt_row: &RowData) -> Result<Vec<PhenotypicFeature>, String> {
        let hpo_term_list = &self.cohort_dto.hpo_headers;
        let hpo_data = &ppkt_row.hpo_data;
        if hpo_data.len() != hpo_term_list.len() {
            return Err(format!("Length of HPO headers ({}) does not match length of HPO values {}",
            hpo_term_list.len(), hpo_data.len()));
        }
        let mut ppkt_feature_list: Vec<PhenotypicFeature> = Vec::with_capacity(hpo_data.len());
        for (term, cell_contents) in hpo_term_list.iter().zip(hpo_data.iter()) {
            if ! cell_contents.is_ascertained() {
                continue;
            }
            let hpo_term = self.get_ontology_class(term)?;
            let mut pf = PhenotypicFeature{ 
                description: String::default(), 
                r#type: Some(hpo_term), 
                excluded: cell_contents.is_excluded(), 
                severity: None, 
                modifiers: vec![], 
                onset: None,
                resolution: None, 
                evidence: vec![]
            };
            if cell_contents.has_onset() {
                let ost = time_element_from_str(&cell_contents.to_string())
                    .map_err(|e| format!("malformed time_element for cell '{}': {}", cell_contents, e.to_string()))?;
                pf.onset = Some(ost);
            }
            ppkt_feature_list.push(pf);
        }
        Ok(ppkt_feature_list)
    }


 fn extract_phenopacket_from_row(
        &self, 
        ppkt_row_dto: &RowData, 
    ) -> Result<Phenopacket, String> {
        let individual = self.extract_individual(ppkt_row_dto)?;
        let is_male =  &ppkt_row_dto.individual_data.sex == "M";
        
        let ppkt_var_exporter = PpktVariantExporter::new(is_male,&self.cohort_dto);
        let interpretation_list = ppkt_var_exporter.get_interpretation_list(ppkt_row_dto)?;

        let ppkt = Phenopacket{ 
            id: self.get_phenopacket_id(ppkt_row_dto), 
            subject:  Some(self.extract_individual(ppkt_row_dto)?), 
            phenotypic_features: self.get_phenopacket_features(ppkt_row_dto)?, 
            measurements: vec![], 
            biosamples: vec![], 
            interpretations: interpretation_list, 
            diseases: self.get_disease_list(ppkt_row_dto)?, 
            medical_actions: vec![], 
            files: vec![], 
            meta_data: Some(self.get_meta_data(ppkt_row_dto)?) 
        };
    
        Ok(ppkt)


    }

/// The serde JSON serialization outputs certain fields that have concrete default values. For instance, karyotypic_sex is an integer enumeration,
/// and the first value (zero) stands for UNKNOWN_KARYOTYPE. Even though we did not actually enter this value into out Phenopacket, the serialization
/// routine outputs this default value, which essentially just clutters the output and does not provide useful information. Another default value is
/// survival_time_in_days of zero - this would appear if we list the subject as deceased even though we do not provide survival time information. In the latter
/// case, this is incorrect. Therefore, we manually strip these two values in the output.
/// Remove default-but-unset fields from a Phenopacket JSON without touching `subject` itself.
/// Note that we use the preserve_order option for serde_json; otherwise, this step
/// is likely to rearrange order of top-level elements.
/// - Drops `subject.karyotypic_sex` if it's "UNKNOWN_KARYOTYPE" or 0
/// - Optionally drops `survival_time_in_days` if it's 0, wherever you expect it (subject or nested)
pub fn strip_phenopacket_defaults(root: &mut Value) {
    // Top-level `subject`
    if let Value::Object(root_map) = root {
        if let Some(Value::Object(subject)) = root_map.get_mut("subject") {
            // Remove karyotypic_sex if it's the unknown/default
            let drop_karyotype = match subject.get("karyotypicSex") {
                Some(Value::String(s)) if s == "UNKNOWN_KARYOTYPE" => true,
                Some(Value::Number(n)) if n.as_i64() == Some(0) => true,
                _ => false,
            };
            if drop_karyotype {
                subject.remove("karyotypicSex");
            }

            if let Some(Value::Object(vs)) = subject.get_mut("vitalStatus") {
                if let Some(Value::Number(n)) = vs.get("survivalTimeInDays") {
                    if n.as_i64() == Some(0) {
                        vs.remove("survivalTimeInDays");
                    }
                }
            }
        }
    }
}

    pub fn get_all_phenopackets(&self) -> Result<Vec<Phenopacket>, String> {
        let mut ppkt_list: Vec<Phenopacket> = Vec::new();
        for row in &self.cohort_dto.rows {
           let ppkt = self.extract_phenopacket_from_row(row)?;
           ppkt_list.push(ppkt);
        }

        Ok(ppkt_list)
    }


}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use serde_json::json;

    /// Remove the redundant field while leaving all else intact
    #[test]
    fn test_strip_removes_unknown_karyotypic_sex_string() {
        let mut packet = json!({
            "subject": {
                "id": "patient1",
                "sex": "MALE",
                "karyotypicSex": "UNKNOWN_KARYOTYPE"
            }
        });
        PpktExporter::strip_phenopacket_defaults(&mut packet);
        // karyotypic_sex should be gone
        assert!(!packet["subject"].get("karyotypicSex").is_some());
        // id and sex should remain
        assert_eq!(packet["subject"]["id"], "patient1");
        assert_eq!(packet["subject"]["sex"], "MALE");
    }

    /// This is falsely added to the export with some vital status because
    /// the default value of an integer is zero (which leads to an incorrect phenopacket)
    /// Here we show we remove this entry without changing the rest
    #[test]
    fn test_strip_removes_survival_time_in_days_zero() {
        let mut packet = json!({
            "subject": {
                "id": "patient3",
                "sex": "UNKNOWN_SEX",
                "vitalStatus": {
                    "status": "DECEASED",
                    "survivalTimeInDays": 0
                }
            }
        });

        PpktExporter::strip_phenopacket_defaults(&mut packet);
        //println!("{}", packet);

        assert!(! packet["subject"]["vitalStatus"].get("survivalTimeInDays").is_some());
        assert_eq!(packet["subject"]["vitalStatus"]["status"], "DECEASED");
    }


  #[test]
    fn test_strip_removes_unknown_karyotypic_sex_string2() {
        let mut packet = json!({
            "subject": {
                "id": "PMID_29198722_p_Arg913Ter_Affected_Individual_1",
                "sex": "MALE",
                "karyotypicSex":"UNKNOWN_KARYOTYPE",
                "vitalStatus": {
                    "status": "DECEASED",
                    "survivalTimeInDays": 0
                }
            }
        });
         PpktExporter::strip_phenopacket_defaults(&mut packet);
        assert!(!packet["subject"].get("karyotypicSex").is_some());
}




    #[test]
    fn test_strip_does_not_remove_valid_values() {
        let mut packet = json!({
            "subject": {
                "id": "patient4",
                "sex": "MALE",
                "karyotypicSex": "XY",
                "vitalStatus": {
                    "status": "DECEASED",
                    "survivalTimeInDays": 365
                }
            }
        });

        PpktExporter::strip_phenopacket_defaults(&mut packet);

        // Nothing should be removed
        assert_eq!(packet["subject"]["karyotypicSex"], "XY");
        assert_eq!(packet["subject"]["vitalStatus"]["survivalTimeInDays"], 365);
    }

    #[rstest]
    fn test_strip_removes_2_invalid_values() {
        let mut packet = json!({
            "subject": {
                "id": "patient4",
                "sex": "MALE",
                "karyotypicSex": "UNKNOWN_KARYOTYPE",
                "vitalStatus": {
                    "status": "DECEASED",
                    "survivalTimeInDays": 0
                }
            }
        });

        PpktExporter::strip_phenopacket_defaults(&mut packet);
        assert!(!packet["subject"].get("karyotypicSex").is_some());
        assert!(!packet["subject"]["vitalStatus"].get("survivalTimeInDays").is_some());
    }

    /// This test is actually making sure that function from phenopacket_tools is doing what we expect it to
    /// i.e., it is a sanity check
    #[rstest]
    #[case("Antenatal onset", true)]
    #[case("Antenatl onset", false)]
    #[case("P43Y2D", true)]
    #[case("P43Y2", false)]
    #[case("G34w2d", true)]
    #[case("G34w7d", false)]
    #[case("G34w", true)]
    fn test_age_strings(
        #[case] onset_string: &str,
        #[case] is_valid: bool
    ) {
        let result = time_element_from_str(onset_string);
        if is_valid {
            assert!(result.is_ok());
        } else {
            assert!(result.is_err())
        }

    }


}
