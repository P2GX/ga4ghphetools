//! CohortData
//! 
//! This file contains definitions of structure with data about a cohort. They are used as Data Transfer Objects between the front and back end and the CohortData is used to serialize the data
//! about the entire cohort.

use std::collections::HashMap;
use std::str::FromStr;

use chrono::Local;
use serde::{Deserialize, Serialize};
use crate::dto::hgvs_variant::HgvsVariant;
use crate::dto::hpo_term_dto::CellValue;
use crate::dto::intergenic_variant::IntergenicHgvsVariant;
use crate::dto::structural_variant::StructuralVariant;
use crate::dto::hpo_term_dto::HpoTermDuplet;
use crate::ppkt::ppkt_row::PpktRow;




#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IndividualData {
    pub pmid: String,
    pub title: String,
    pub individual_id: String,
    pub comment: String,
    pub age_of_onset: String,
    pub age_at_last_encounter: String,
    pub deceased: String,
    pub sex: String
}

impl IndividualData {
    pub fn new(
        pmid: &str,
        title: &str,
        individual_id: &str,
        comment: &str,
        age_of_onset: &str,
        age_at_last_encounter: &str,
        deceased: &str,
        sex: &str,) -> Self{
            Self { 
                pmid: pmid.to_string(), 
                title: title.to_string(), 
                individual_id: individual_id.to_string(), 
                comment: comment.to_string(),
                age_of_onset: age_of_onset.to_string(),
                age_at_last_encounter: age_at_last_encounter.to_string(),
                deceased: deceased.to_string(),
                sex: sex.to_string(),
            }
    }
}

/// This structure contains information about the 
/// variants found in an individual for one specific gene.
/// The full information about the variants needed to create phenopackets is stored in the
/// HashMaps 
/// /* 
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneVariantData {
    pub hgnc_id: String,
    pub gene_symbol: String,
    pub transcript: String,
    pub allele1: String,
    pub allele2: String,
    pub variant_comment: String,
}

/// This struct is used to organize data for the import for Legacy Excel files, and also to transmit data about
/// variants from the front end to the backend. It is not used for serialization.
impl GeneVariantData {
    pub fn new(hgnc_id: &str,
                gene_symbol: &str,
                transcript: &str,
                allele1: &str,
                allele2: &str,
                variant_comment: &str) -> Self {
        Self { 
            hgnc_id: hgnc_id.to_string(),
            gene_symbol: gene_symbol.to_string(), 
            transcript: transcript.to_string(), 
            allele1: allele1.to_string(), 
            allele2: allele2.to_string(), 
            variant_comment: variant_comment.to_string() 
        }
    }

    pub fn get_key_allele1(&self) -> String {
        format!("{}_{}_{}", self.allele1, self.gene_symbol, self.transcript)
    }

    pub fn get_key_allele2(&self) -> String {
        if self.allele2.is_empty() || self.allele2 == "na" {
            "na".to_string()
        } else {
            format!("{}_{}_{}", self.allele2, self.gene_symbol, self.transcript)
        }
    }

    pub fn allele1_is_hgvs(&self) -> bool {
        self.allele1.starts_with("c.") || self.allele1.starts_with("n.")
    }

    pub fn allele2_is_hgvs(&self) -> bool {
        self.allele2.starts_with("c.") || self.allele2.starts_with("n.")
    }

    pub fn allele1_is_present(&self) -> bool {
        self.allele1 != "na"
    }

    pub fn allele1_is_sv(&self) -> bool {
        self.allele1_is_present() && ! self.allele1_is_hgvs()
    }

    pub fn allele2_is_present(&self) -> bool {
        self.allele2 != "na"
    }

    pub fn allele2_is_sv(&self) -> bool {
        self.allele2_is_present() && ! self.allele2_is_hgvs()
    }

}

/// A structure for representing the mode of inheritance (MOI) of 
/// a disease. Knowing the MOI can help to Q/C a cohort - e.g., to 
/// flag autosomal recessive cases with just one pathogenic allele
/// Also, it allows us to use the MOI data to output HPOA annotation
/// for the mode of inheritance
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModeOfInheritance {
    /// Human Phenotype Ontology identifier such as HP:0000006
    pub hpo_id: String,
    /// Human Phenotype Ontology term label such as Autosomal dominant inheritance 
    pub hpo_label: String,
    /// PMID or other citation in CURIE form to support the assertion
    pub citation: String
}

impl ModeOfInheritance {
    pub fn is_autosomal_dominant(&self) -> bool {
        self.hpo_id == "HP:0000006"
    }

    pub fn is_autosomal_recessive(&self) -> bool {
        self.hpo_id == "HP:0000007"
    }

    pub fn is_x_chromosomal(&self) -> bool {
        self.hpo_id == "HP:0001417" || self.hpo_id == "HP:0001423" || self.hpo_id == "HP:0001419"
    }
    /// HP:0034340 Pseudoautosomal dominant inheritance
    pub fn is_pseudoautosomal_dominant(&self) -> bool {
        self.hpo_id == "HP:0034340"
    }

    pub fn is_pseudoautosomal_recessive(&self) -> bool {
         self.hpo_id == "HP:0034341"
    }

}


 

/// A gene and its trasncript of reference
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneTranscriptData {
    pub hgnc_id: String,
    pub gene_symbol: String,
    pub transcript: String,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiseaseData {
    pub disease_id: String,
    pub disease_label: String,
    pub mode_of_inheritance_list: Vec<ModeOfInheritance>,
    pub gene_transcript_list: Vec<GeneTranscriptData>
}

impl DiseaseData {
    pub fn new(disease_id: &str, disease_label: &str) -> Self {
        Self { 
            disease_id: disease_id.to_string(), 
            disease_label: disease_label.to_string(),
            mode_of_inheritance_list: vec![],
            gene_transcript_list: vec![],
        }
    }
}


#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RowData {
    pub individual_data: IndividualData,
    pub disease_id_list: Vec<String>,
    pub allele_count_map: HashMap<String, usize>,
    pub hpo_data: Vec<CellValue>
}

impl RowData {
    pub fn from_ppkt_row(ppkt_row: &PpktRow, allele_key_list: Vec<String>) -> Result<Self, String> {
        let mut allele_count_map: HashMap<String, usize> = HashMap::new();
        for allele in allele_key_list {
            *allele_count_map.entry(allele).or_insert(0) += 1;
        };
        let hpo_list = ppkt_row.get_hpo_value_list()?;
        Ok(Self { 
            individual_data: ppkt_row.get_individual_dto(), 
            disease_id_list: ppkt_row.get_disease_id_list(), 
            allele_count_map, 
            hpo_data: hpo_list,
        })
    }
}



#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum CohortType {
    Mendelian,
    Melded,
    Digenic
}

impl FromStr for CohortType {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, String> {
        match s.to_ascii_lowercase().as_str() {
            "mendelian" => Ok(CohortType::Mendelian),
            "melded" => Ok(CohortType::Melded),
            "digenic" => Ok(CohortType::Digenic),
            _ => Err(format!("Unrecognized template type {s}")),
        }
    }
}


impl std::fmt::Display for CohortType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            CohortType::Mendelian => "mendelian",
            CohortType::Melded => "melded",
            CohortType::Digenic => "digenic",
        };
        write!(f, "{}", s)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CurationEvent {
    /// ORCID identifier of the curator
    pub orcid: String,
    /// Date of curation in YYYY-MM-DD format
    pub date: String,
}

impl CurationEvent {
    pub fn new(orcid: &str) -> Self {
        Self { 
            orcid: orcid.to_string(), 
            date: Local::now().format("%Y-%m-%d").to_string()
        }
    }
}



/// This is the representation of the cohort (source of truth)
/// There is a corresponding typescript DTO in the front-end
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CohortData {
    /// Mendelian, Melded, or Digenic
    pub cohort_type: CohortType,
    /// The diseases and genes in focus for the current cohort
    pub disease_list: Vec<DiseaseData>,
    /// The HPO terms used to annotate the cohort
    pub hpo_headers: Vec<HpoTermDuplet>,
    /// The phenopackets (rows) in the current cohort
    pub rows: Vec<RowData>,
    /// Validated HGVS variants.
    pub hgvs_variants: HashMap<String, HgvsVariant>,
    /// Validated structural (symbolic) variants
    pub structural_variants: HashMap<String, StructuralVariant>,
    /// Validated intergenic variants
    #[serde(default)]
    pub intergenic_variants: HashMap<String, IntergenicHgvsVariant>,
    /// Version of this DTO JSON
    pub phetools_schema_version: String,
    /// Version of HPO used to create the current version of this cohort
    pub hpo_version: String,
    /// Acronym that we will use for storing the template (GENE_ACRONYM_individuals.json)
    pub cohort_acronym: Option<String>,
    /// History of biocuration events in chronological order
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub curation_history: Vec<CurationEvent>,
}

/// Version of the Cohort JSON schema
const PHETOOLS_SCHEMA_VERSION: &str = "0.3";

impl CohortData {
    /// Initialize a new CohortData object for Mendelian cohorts. 
    /// Lists for validated variants are generated that should be filled using
    /// VariantValidator (for HGVS) and StructuralVariantValidator (for structural variants).
    /// This function is only used for ingesting (legacy) Excel files, since we are migrating
    /// to using the JSON representation of the CohortData as the serialization format.
    pub fn mendelian(
            dg_data: DiseaseData,
            hpo_headers: Vec<HpoTermDuplet>, 
            rows: Vec<RowData>,
            hpo_version: &str) -> Self {
        Self::mendelian_with_variants(dg_data, hpo_headers, rows, hpo_version, HashMap::new(), HashMap::new())
    }

    /// Initialize a new CohortData object for a Melded Phenotype case.
    /// All fields will be empty expect the DiseaseData
    pub fn melded(
        disease_list: Vec<DiseaseData>,
        hpo_version: &str
    ) -> Self {
        Self {
            cohort_type: CohortType::Melded,
            disease_list: disease_list,
            hpo_headers: vec![],
            rows: vec![],
            hgvs_variants: HashMap::new(),
            structural_variants: HashMap::new(),
            intergenic_variants: HashMap::new(),
            phetools_schema_version: PHETOOLS_SCHEMA_VERSION.to_string(),
            hpo_version: hpo_version.to_string(),
            cohort_acronym: None,
            curation_history: vec![],
        }
    }

    /// We will mark the existing (Excel legacy) curation events using the date of publication of the 
    /// Phenopacket Store article
    fn legacy_curation() -> CurationEvent {
        CurationEvent { 
            orcid: "0000-0002-0736-9199".to_string(), 
            date: "2025-01-09".to_string(),
        }
    }

    /// Legacy function for the old Excel files.
    /// These files do not have intergenic variants, thus we just create an empty HashMap
    /// There are less than 10 files that still need work to transform TODO -- after that DELETE this function.
    pub fn mendelian_with_variants(
            dg_data: DiseaseData,
            hpo_headers: Vec<HpoTermDuplet>, 
            rows: Vec<RowData>,
            hpo_version: &str,
            hgvs_variants: HashMap<String, HgvsVariant>,
            structural_variants: HashMap<String, StructuralVariant>
        ) -> Self {
        Self { 
            cohort_type: CohortType::Mendelian, 
            disease_list: vec![dg_data],
            hpo_headers, 
            rows,
            hgvs_variants,
            structural_variants,
            intergenic_variants: HashMap::new(),
            phetools_schema_version: PHETOOLS_SCHEMA_VERSION.to_string(),
            hpo_version: hpo_version.to_string(),
            cohort_acronym: None,
            curation_history: vec![Self::legacy_curation()],
        }
    }

    pub fn template_type(&self) -> CohortType {
        self.cohort_type
    }

    pub fn is_mendelian(&self) -> bool {
        self.template_type() == CohortType::Mendelian
    }

     pub fn is_melded(&self) -> bool {
        self.template_type() == CohortType::Melded
    }

    pub fn get_phetools_schema_version() -> String {
        PHETOOLS_SCHEMA_VERSION.to_string()
    }


    pub fn get_disease_dto_list(&self) -> std::result::Result<Vec<DiseaseData>, String> {
        if ! self.is_mendelian() {
            return Err("Not implemented except for Mendelian".to_string());
        }
        Ok(self.disease_list.clone())
    }

    pub fn phenopackets_schema_version() -> String {
        return PHETOOLS_SCHEMA_VERSION.to_string()
    }
    
}