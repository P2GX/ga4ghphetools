use core::convert::From;
use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use crate::dto::hgvs_variant::HgvsVariant;
use crate::dto::structural_variant::{self, StructuralVariant, SvType};
use crate::header::duplet_item::DupletItem;
use crate::header::hpo_term_duplet::HpoTermDuplet;
use crate::ppkt::ppkt_row::PpktRow;
use crate::template::cohort_dto_builder::CohortType;



#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IndividualDto {
    pub pmid: String,
    pub title: String,
    pub individual_id: String,
    pub comment: String,
    pub age_of_onset: String,
    pub age_at_last_encounter: String,
    pub deceased: String,
    pub sex: String
}

impl IndividualDto {
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
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneVariantDto {
    pub hgnc_id: String,
    pub gene_symbol: String,
    pub transcript: String,
    pub allele1: String,
    pub allele2: String,
    pub variant_comment: String,
}


impl GeneVariantDto {
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
pub struct DiseaseDto {
    pub disease_id: String,
    pub disease_label: String,
}

impl DiseaseDto {
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
pub struct GeneTranscriptDto {
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
pub struct DiseaseGeneDto {
    pub template_type: CohortType,
    /// Acronym that we will use for storing the template (GENE_ACRONYM_individuals.json)
    pub cohort_acronym: String,
    pub disease_dto_list: Vec<DiseaseDto>,
    pub gene_transcript_dto_list: Vec<GeneTranscriptDto>,
}





/// For Melded Phenotypes, there are two diseases, and two gene/variant bundles.
/// Their order does not matter in the GA4GH phenopacket. By convention, we will 
/// enforce that they have the same order.
/// For digenic, there is one disease and there are two gene/variant bundles.
/// For Mendelian, there is one disease and one gene/variant bundle.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaseBundleDto {
    pub diseases: Vec<DiseaseDto>, // 1 or 2 depending on template
    pub gene_vars: Vec<GeneVariantDto>, // 1 or 2 depending on template
}




#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CellDto {
    pub value: String
}

impl CellDto {
    pub fn new(val: impl Into<String>) -> Self {
        Self { value: val.into() }
    }
}


#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RowDto {
    pub individual_dto: IndividualDto,
    pub disease_dto_list: Vec<DiseaseDto>,
    //pub gene_var_dto_list: Vec<GeneVariantDto>,
    pub allele_count_map: HashMap<String, usize>,
    pub hpo_data: Vec<CellDto>
}

impl RowDto {
    pub fn from_ppkt_row(ppkt_row: &PpktRow, allele_key_list: Vec<String>) -> Self {
        let mut allele_count_map: HashMap<String, usize> = HashMap::new();
        for allele in allele_key_list {
            *allele_count_map.entry(allele).or_insert(0) += 1;
        };
        Self { individual_dto: ppkt_row.get_individual_dto(), 
            disease_dto_list: ppkt_row.get_disease_dto_list(), 
            allele_count_map, 
            hpo_data: ppkt_row.get_hpo_value_list()
        }
    }
}

/// The tabular serialization format phetools has the first two rows act as header.
/// For display, we will combine the two rows into one. For the HPO rows, we
/// we add a link.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HeaderDupletDto {
    pub h1: String,
    pub h2: String,
}

impl HeaderDupletDto {
    pub fn new(row1: &str, row2: &str) -> Self {
        Self { h1: row1.into(), h2: row2.into() }
    }

    pub fn from_duplet_item(duplet: &DupletItem) -> Self {
        Self::new(duplet.row1(), duplet.row2())
    }

    pub fn to_hpo_duplet(&self) -> HpoTermDuplet {
        HpoTermDuplet::new(self.h1.clone(), self.h2.clone())
    }
}
/// convert from DupletItem using into()
impl From<DupletItem> for HeaderDupletDto {
    fn from(duplet: DupletItem) -> Self {
        Self {
            h1: duplet.row1.clone(),
            h2: duplet.row2.clone()
        }
    }
}

/// This is the representation of the cohort (source of truth)
/// There is a corresponding typescript DTO in the front-end
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CohortDto {
    /// Mendelian, Melded, or Digenic
    pub cohort_type: CohortType,
    /// The diseases and genes in focus for the current cohort
    pub disease_gene_dto: DiseaseGeneDto,
    /// The HPO terms used to annotate the cohort
    pub hpo_headers: Vec<HeaderDupletDto>,
    /// The phenopackets (rows) in the current cohort
    pub rows: Vec<RowDto>,
    pub hgvs_variants: HashMap<String, HgvsVariant>,
    pub structural_variants: HashMap<String, StructuralVariant>,
}

impl CohortDto {
    /// Initialize a new TemplateDto for Mendelian cohorts. 
    /// Lists for validated variants are generated that should be filled using
    /// VariantValidator (for HGVS) and StructuralVariantValidator (for structural variants).
    /// This function is only used for ingesting (legacy) Excel files, since we are migrating
    /// to using the JSON representation of the TemplateDto as the serialization format.
    /// TODO: remove this function once we have migrated all cohorts to the JSON format.
    pub fn mendelian(
            dg_dto: DiseaseGeneDto,
            hpo_headers: Vec<HeaderDupletDto>, 
            rows: Vec<RowDto>) -> Self {
        Self { 
            cohort_type: CohortType::Mendelian, 
            disease_gene_dto: dg_dto,
            hpo_headers, 
            rows,
            hgvs_variants: HashMap::new(),
            structural_variants: HashMap::new(),
        }
    }

    pub fn mendelian_with_variants(
            dg_dto: DiseaseGeneDto,
            hpo_headers: Vec<HeaderDupletDto>, 
            rows: Vec<RowDto>,
            hgvs_variants: HashMap<String, HgvsVariant>,
            structural_variants: HashMap<String, StructuralVariant>
        ) -> Self {
        Self { 
            cohort_type: CohortType::Mendelian, 
            disease_gene_dto: dg_dto,
            hpo_headers, 
            rows,
            hgvs_variants,
            structural_variants,
        }
    }

    pub fn template_type(&self) -> CohortType {
        self.cohort_type
    }

    pub fn is_mendelian(&self) -> bool {
        self.template_type() == CohortType::Mendelian
    }


    pub fn get_disease_dto_list(&self) -> std::result::Result<Vec<DiseaseDto>, String> {
        if ! self.is_mendelian() {
            return Err("Not implemented except for Mendelian".to_string());
        }
        let first_disease = self.rows
            .first()
            .ok_or_else(|| "No rows provided".to_string())?
            .disease_dto_list
            .get(0)
            .ok_or_else(|| "First row has no disease".to_string())?
            .clone();

        for (i, row) in self.rows.iter().enumerate() {
            if row.disease_dto_list.len() != 1 {
                return Err(format!("Row {} does not have exactly one disease", i));
            }
            if row.disease_dto_list[0] != first_disease {
                return Err(format!("Row {} has a different disease", i));
            }
        }

    Ok(vec![first_disease])
}
    
}