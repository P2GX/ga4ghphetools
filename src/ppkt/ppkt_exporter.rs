//! Module to export GA4GH Phenopackets from the information in the template.


use std::collections::HashMap;

use std::sync::Arc;
use ontolius::ontology::csr::FullCsrOntology;

use phenopackets::schema::v2::core::{ExternalReference, MetaData};
use phenopackets::schema::v2::Phenopacket;
use crate::dto::cohort_dto::{CohortData, DiseaseData, RowData};
use crate::ppkt::ppkt_builder::{DEFAULT_GENO_VERSION, DEFAULT_HGNC_VERSION, DEFAULT_OMIM_VERSION, DEFAULT_SO_VERSION, PhenopacketBuilder};
use crate::ppkt::ppkt_variant_exporter::PpktVariantExporter;
use phenopacket_tools;
use phenopacket_tools::builders::builder::Builder;






/// Structure to export phenopackets from a CohortData object.
pub struct PpktExporter {
    /// Reference to the Ontolius Human Phenotype Ontology Full CSR object
    hpo: Arc<FullCsrOntology>,
    geno_version: String,
    omim_version: String,
    hgnc_version: String,
    so_version: String,
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
            DEFAULT_SO_VERSION,
            creator_orcid,
            cohort)
    }

    pub fn from_versions(
        hpo: Arc<FullCsrOntology>,
        geno_version: &str,
        omim_version: &str, 
        hgnc_version: &str ,
        so_version: &str,
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
            so_version: so_version.to_string(),
            orcid_id: creator_orcid.to_string(),
            cohort_dto: cohort,
            disease_id_map: disease_map,
        }
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

    fn so_version(&self) -> &str {
        &self.so_version
    }

    fn has_sequence_ontology(&self, ppkt_row: &RowData) -> bool {
        for allele in ppkt_row.allele_count_map.keys() {
            if self.cohort_dto.structural_variants.contains_key(allele) {
                return true;
            }
        }
        false
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
        if self.has_sequence_ontology(row_dto) {
            // We only need Sequence Ontology (SO) for structural variants (SV)
            // If we do not have an SV, then SO would be redundant
            let so = phenopacket_tools::builders::resources::Resources::so_version(self.so_version());
            meta_data.resources.push(so);
        }
        meta_data.external_references.push(ext_res);
        Ok(meta_data)
    }


    fn allele_not_contained(allele: &str) -> String {
        format!("'{allele}' must be validated before exporting to Phenopacket Schema")
    }
   
    fn extract_phenopacket_from_row(
        &self, 
        ppkt_row_dto: &RowData, 
    ) -> Result<Phenopacket, String> {
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



    pub fn get_all_phenopackets(&self) -> Result<Vec<Phenopacket>, String> {
        let mut ppkt_list: Vec<Phenopacket> = Vec::new();
        for row in &self.cohort_dto.rows {
           let ppkt = self.extract_phenopacket_from_row(row)?;
           ppkt_list.push(ppkt);
        }

        Ok(ppkt_list)
    }


}

impl PhenopacketBuilder for PpktExporter {
    fn hpo(&self) -> &Arc<FullCsrOntology> { &self.hpo }
    fn cohort(&self) -> &CohortData { &self.cohort_dto }
    fn disease_id_map(&self) -> &HashMap<String, DiseaseData> { &self.disease_id_map }
    fn orcid(&self) -> &str { &self.orcid_id }
    fn geno_version(&self) -> &str { &self.geno_version }
    fn omim_version(&self) -> &str { &self.omim_version }
    fn hgnc_version(&self) -> &str { &self.hgnc_version }
    fn so_version(&self) -> &str { &self.so_version }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use std::path::PathBuf;  
    use serde_json::json;
    use crate::{ppkt::json_cleanup::strip_phenopacket_defaults, test_utils::fixtures::hpo};

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
        strip_phenopacket_defaults(&mut packet);
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

        strip_phenopacket_defaults(&mut packet);
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
        strip_phenopacket_defaults(&mut packet);
        assert!(!packet["subject"].get("karyotypicSex").is_some());
    }

    #[test]
    fn test_do_not_add_unknown_sex() {
        let mut packet = json!({
            "subject": {
                "id": "PMID_29198722_p_Arg913Ter_Affected_Individual_1",
            }
        });
        strip_phenopacket_defaults(&mut packet);
        assert!(!packet["subject"].get("sex").is_some());
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

        strip_phenopacket_defaults(&mut packet);

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

        strip_phenopacket_defaults(&mut packet);
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
        use phenopacket_tools::builders::time_elements::time_element_from_str;

        let result = time_element_from_str(onset_string);
        if is_valid {
            assert!(result.is_ok());
        } else {
            assert!(result.is_err())
        }

    }


     #[rstest]
     #[ignore = "local file"]
    fn test_export_ppkt(hpo: Arc<FullCsrOntology>) {
        let input_file = "/Users/robin/GIT/mgd-ppkt/cohorts/MYH7_CMH1_PRKAG2_CMH6_individuals.json";
        let cohort = crate::factory::load_json_cohort(input_file).expect("Could not load Cohort JSON file");
        let orcid = "0000-0000-0000-0000".to_string();
        let output_dir = "/Users/robin/TMP";
        let path = PathBuf::from(output_dir);
        let overwrite = true;
        let result = crate::ppkt::write_phenopackets(cohort, path, orcid, hpo.clone(), overwrite);
        assert!(result.is_ok());
        let n_processed = result.unwrap();
        assert_eq!(1, n_processed); 
    }


}
