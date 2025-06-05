

use std::process::id;

use phenopacket_tools::builders::time_elements::time_element_from_str;
use phenopackets::schema::v1::core::KaryotypicSex;
use phenopackets::schema::v2::core::vital_status::Status;
use phenopackets::schema::v2::core::{Disease, ExternalReference, Individual, MetaData, PhenotypicFeature, Sex, TimeElement, VitalStatus};
use phenopackets::schema::v2::Phenopacket;
use prost_types::value;
use crate::error::{self, Error, Result};
use phenopacket_tools;
use super::ppkt_row::{self, PpktRow};
use phenopacket_tools::builders::builder::Builder;

pub struct PpktExporter {

}

impl Error {
    pub fn malformed_time_element(msg: impl Into<String>) -> Self {
        Error::AgeParseError { msg: msg.into() }
    }

    pub fn malformed_ppkt_disease(ppkt_row: &PpktRow) -> Self {
        let disease_id = match ppkt_row.disease_id() {
            Ok(id) => id,
            Err(_) => "?".to_string()
        };
        let disease_label = match ppkt_row.disease_label() {
            Ok(id) => id,
            Err(_) => "?".to_string()
        };
        let msg = format!("Malformed PpktRow disease: {} ({})", disease_label, disease_id);
        Error::TemplateError { msg }
    }
}


impl PpktExporter {


    /// Create a GA4GH Individual message
    pub fn extract_individual(&self, ppkt_row: &PpktRow) -> Result<Individual> {
        let mut idvl = Individual{ 
            id: ppkt_row.individual_id()?, 
            alternate_ids: vec![], 
            date_of_birth: None, 
            time_at_last_encounter: None, 
            vital_status: None, 
            sex: Sex::UnknownSex.into(), 
            karyotypic_sex: KaryotypicSex::UnknownKaryotype.into(), 
            gender: None, 
            taxonomy: None };
        match ppkt_row.sex()?.as_ref() {
            "M" => idvl.sex = Sex::Male.into(),
            "F" => idvl.sex = Sex::Female.into(),
            "O" => idvl.sex = Sex::OtherSex.into(),
            "U" => idvl.sex = Sex::UnknownSex.into(),
            _ => { return Err(Error::TemplateError { msg: format!("Did not recognize sex string '{}'", ppkt_row.sex()?) });
            }
        };
        let age_last_encounter = ppkt_row.age_at_last_encounter()?;
        if age_last_encounter != "na" {
            let age = time_element_from_str(&age_last_encounter)
                .map_err(|e| Error::malformed_time_element(e.to_string()))?;
            idvl.time_at_last_encounter = Some(age);
        }
        let deceased = ppkt_row.deceased()?;
        if deceased == "yes" {
            idvl.vital_status = Some(VitalStatus{ 
                status: Status::Deceased.into(), 
                time_of_death: None, 
                cause_of_death: None, 
                survival_time_in_days: 0 
            });
        } 
        Ok(idvl)

    }

    /// TODO possibly the PpktExporter has state (created, etc, also dynamically get the time string)
    pub fn get_meta_data(&self, ppkt_row: &PpktRow) -> Result<MetaData> {

        let created_by = "Earnest B. Biocurator";
        let mut meta_data = Builder::meta_data_now(created_by);
        let hpo = phenopacket_tools::builders::resources::Resources::hpo_version("v2025-03-03");
        let geno = phenopacket_tools::builders::resources::Resources::geno_version("2023-10-08");
        let pmid = ppkt_row.pmid()?;
        let title = ppkt_row.title()?;
        let ext_res = ExternalReference{ 
            id: pmid, 
            reference: String::default(), 
            description: title 
        };
        meta_data.resources.push(hpo);
        meta_data.resources.push(geno);
        meta_data.external_references.push(ext_res);
        Ok(meta_data)
    }


    /// Generate the phenopacket identifier from the PMID and the individual identifier
    /// TODO - improve
    pub fn get_phenopacket_id(&self, ppkt_row: &PpktRow) -> Result<String> {
        let pmid = ppkt_row.pmid()?.replace(":", "_");
        let individual_id = ppkt_row.individual_id()?.replace(" ", "_");
        let ppkt_id = format!("{}_{}", pmid, individual_id);
        let ppkt_id = ppkt_id.replace("__", "_");
        /* TODO remove trailing "_" 
        if ppkt_id.ends_with("_") {
            ppkt_id = ppkt_id.
        }*/
        // TODO don't just filter, replace with "_"
        let ppkt_id = ppkt_id.chars().into_iter()
            .filter(|c| char::is_alphanumeric(*c))
            .clone().collect();
        Ok(ppkt_id)
    }

    /// TODO extend for multiple diseases
    pub fn get_disease(&self, ppkt_row: &PpktRow) -> Result<Disease> {
        let dx_id = Builder::ontology_class(ppkt_row.disease_id()?, ppkt_row.disease_label()?)
            .map_err(|e| Error::DiseaseIdError{msg:format!("malformed disease id")})?;
        let mut disease = Disease{ 
            term: Some(dx_id), 
            excluded: false, 
            onset: None, 
            resolution: None, 
            disease_stage: vec![], 
            clinical_tnm_finding: vec![], 
            primary_site: None, 
            laterality: None 
        };
        let onset = ppkt_row.age_of_onset()?;
        if onset != "na" {
            let age = time_element_from_str(&onset)
                .map_err(|e| Error::malformed_time_element(e.to_string()))?;
            disease.onset = Some(age);
        };
        Ok(disease)
    }

    


    pub fn get_phenopacket_features(&self, ppkt_row: &PpktRow) -> Result<Vec<PhenotypicFeature>> {
        let dto_list = ppkt_row.get_hpo_term_dto_list()?;
        let mut ppkt_feature_list: Vec<PhenotypicFeature> = Vec::with_capacity(dto_list.len());
        for dto in dto_list {
            if dto.is_not_ascertained() {
                continue;
            }
            let hpo_term = Builder::ontology_class(dto.term_id(), dto.label())
                .map_err(|e| Error::termid_parse_error(dto.term_id()))?;
            let pf = PhenotypicFeature{ 
                description: String::default(), 
                r#type: Some(hpo_term), 
                excluded: dto.is_excluded(), 
                severity: None, 
                modifiers: vec![], 
                onset: todo!(),
                resolution: None, 
                evidence: vec![]
            };
            if dto.has_onset() {
                let value = dto.onset()?;
                let ost = time_element_from_str(&value)
                    .map_err(|e| Error::malformed_time_element(value))?;
                pf.onset = Some(ost);
            }
            ppkt_feature_list.push(pf);
        }
        Ok(ppkt_feature_list)
    }


    pub fn export_phenopacket(&self, ppkt_row: PpktRow) -> Result<Phenopacket> {
        let ppkt = Phenopacket{ 
            id: self.get_phenopacket_id(&ppkt_row)?, 
            subject:  Some(self.extract_individual(&ppkt_row)?), 
            phenotypic_features: self.get_phenopacket_features(&ppkt_row)?, 
            measurements: vec![], 
            biosamples: vec![], 
            interpretations: todo!(), 
            diseases: vec![self.get_disease(&ppkt_row)?], 
            medical_actions: vec![], 
            files: vec![], 
            meta_data: Some(self.get_meta_data(&ppkt_row)?) 
        };
    
        Ok(ppkt)
    }


}