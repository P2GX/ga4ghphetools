//! CohortData
//! 
//! This file contains definitions of structure with data about a cohort. They are used as Data Transfer Objects between the front and back end and the CohortData is used to serialize the data
//! about the entire cohort.

use std::collections::HashMap;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use crate::dto::hgvs_variant::HgvsVariant;
use crate::dto::hpo_term_dto::CellValue;
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


#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiseaseData {
    pub disease_id: String,
    pub disease_label: String,
}

impl DiseaseData {
    pub fn new(disease_id: &str, disease_label: &str) -> Self {
        Self { 
            disease_id: disease_id.to_string(), 
            disease_label: disease_label.to_string() 
        }
    }
}




 

/// A gene and its trasncript of reference
/// We use this to act as a seed when we create a new row (phenopacket)
/// as part of a DiseaseGeneBundleDto
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneTranscriptData {
    pub hgnc_id: String,
    pub gene_symbol: String,
    pub transcript: String,
}

/// Genes and Diseases of reference for a cohort 
/// We use this to act as a seed when we create a new row (phenopacket) 
/// It can be used for Mendelian, Melded, Digenic
/// Mendelian: disease_dto_list and gene_variant_dto_list must both be of length 1
/// Melded: both of length two
/// Digenic: disease_dto of length 1, gene_variant_dto of length 2
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiseaseGeneData {
    pub disease_dto_list: Vec<DiseaseData>,
    pub gene_transcript_dto_list: Vec<GeneTranscriptData>,
}


#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RowData {
    pub individualData: IndividualData,
    pub disease_data_list: Vec<DiseaseData>,
    //pub gene_var_dto_list: Vec<GeneVariantDto>,
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
        Ok(Self { individualData: ppkt_row.get_individual_dto(), 
            disease_data_list: ppkt_row.get_disease_dto_list(), 
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



/// This is the representation of the cohort (source of truth)
/// There is a corresponding typescript DTO in the front-end
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CohortData {
    /// Mendelian, Melded, or Digenic
    pub cohort_type: CohortType,
    /// The diseases and genes in focus for the current cohort
    pub disease_gene_data: DiseaseGeneData,
    /// The HPO terms used to annotate the cohort
    pub hpo_headers: Vec<HpoTermDuplet>,
    /// The phenopackets (rows) in the current cohort
    pub rows: Vec<RowData>,
    /// Validated HGVS variants.
    pub hgvs_variants: HashMap<String, HgvsVariant>,
    /// Validated structural (symbolic) variants
    pub structural_variants: HashMap<String, StructuralVariant>,
    /// Version of the DTO JSON
    pub dto_version: String,
    /// Acronym that we will use for storing the template (GENE_ACRONYM_individuals.json)
    pub cohort_acronym: Option<String>,
}

/// Version of the Cohort JSON schema
const COHORT_DTO_VERSION: &str = "0.2";

impl CohortData {
    /// Initialize a new TemplateDto for Mendelian cohorts. 
    /// Lists for validated variants are generated that should be filled using
    /// VariantValidator (for HGVS) and StructuralVariantValidator (for structural variants).
    /// This function is only used for ingesting (legacy) Excel files, since we are migrating
    /// to using the JSON representation of the TemplateDto as the serialization format.
    pub fn mendelian(
            dg_dto: DiseaseGeneData,
            hpo_headers: Vec<HpoTermDuplet>, 
            rows: Vec<RowData>) -> Self {
        Self::mendelian_with_variants(dg_dto, hpo_headers, rows, HashMap::new(), HashMap::new())
    }

    pub fn mendelian_with_variants(
            dg_dto: DiseaseGeneData,
            hpo_headers: Vec<HpoTermDuplet>, 
            rows: Vec<RowData>,
            hgvs_variants: HashMap<String, HgvsVariant>,
            structural_variants: HashMap<String, StructuralVariant>
        ) -> Self {
        Self { 
            cohort_type: CohortType::Mendelian, 
            disease_gene_data: dg_dto,
            hpo_headers, 
            rows,
            hgvs_variants,
            structural_variants,
            dto_version: COHORT_DTO_VERSION.to_string(),
            cohort_acronym: None
        }
    }

    pub fn template_type(&self) -> CohortType {
        self.cohort_type
    }

    pub fn is_mendelian(&self) -> bool {
        self.template_type() == CohortType::Mendelian
    }


    pub fn get_disease_dto_list(&self) -> std::result::Result<Vec<DiseaseData>, String> {
        if ! self.is_mendelian() {
            return Err("Not implemented except for Mendelian".to_string());
        }
        let first_disease = self.rows
            .first()
            .ok_or_else(|| "No rows provided".to_string())?
            .disease_data_list
            .get(0)
            .ok_or_else(|| "First row has no disease".to_string())?
            .clone();

        for (i, row) in self.rows.iter().enumerate() {
            if row.disease_data_list.len() != 1 {
                return Err(format!("Row {} does not have exactly one disease", i));
            }
            if row.disease_data_list[0] != first_disease {
                return Err(format!("Row {} has a different disease", i));
            }
        }

    Ok(vec![first_disease])
}
    
}