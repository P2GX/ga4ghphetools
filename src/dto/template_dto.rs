use std::str::FromStr;


use phenopackets::schema::v1::core::disease;
use serde::{de, Deserialize, Serialize};
use crate::header::duplet_item::DupletItem;
use crate::ppkt::ppkt_row::PpktRow;
use crate::template::excel::read_excel_to_dataframe;
use crate::error::{Error, Result};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IndividualBundleDto {
    pub pmid: String,
    pub title: String,
    pub individual_id: String,
    pub comment: String,
}

impl IndividualBundleDto {
    pub fn new(
        pmid: &str,
        title: &str,
        individual_id: &str,
        comment: &str,) -> Self{
            Self { 
                pmid: pmid.to_string(), 
                title: title.to_string(), 
                individual_id: individual_id.to_string(), 
                comment: comment.to_string() 
            }
    }
}


#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneVariantBundleDto {
    pub hgnc_id: String,
    pub gene_symbol: String,
    pub transcript: String,
    pub allele1: String,
    pub allele2: String,
    pub variant_comment: String,
}


impl GeneVariantBundleDto {
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
}


#[derive(Clone, Debug, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DemographicDto {
    pub age_of_onset: String,
    pub age_at_last_encounter: String,
    pub deceased: String,
    pub sex: String
}


impl DemographicDto {
    pub fn new(
        age_of_onset: &str,
        age_at_last_encounter: &str,
        deceased: &str,
        sex: &str,
    ) -> Self {
        Self {
            age_of_onset: age_of_onset.to_string(),
            age_at_last_encounter: age_at_last_encounter.to_string(),
            deceased: deceased.to_string(),
            sex: sex.to_string(),
        }
    }
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
    pub gene_vars: Vec<GeneVariantBundleDto>, // 1 or 2 depending on template
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
    pub individual_dto: IndividualBundleDto,
    pub disease_dto_list: Vec<DiseaseDto>,
    pub gene_var_dto_list: Vec<GeneVariantBundleDto>,
    pub demographic_dto: DemographicDto,
    pub hpo_data: Vec<CellDto>
}

impl RowDto {
    pub fn from_ppkt_row(ppkt_row: PpktRow) -> Self {
        Self { individual_dto: ppkt_row.get_individual_dto(), 
            disease_dto_list: ppkt_row.get_disease_dto_list(), 
            gene_var_dto_list: ppkt_row.get_gene_var_dto_list(), 
            demographic_dto: ppkt_row.get_demographic_dto(), 
            hpo_data: ppkt_row.get_hpo_dto_list()
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HeaderDupletDto {
    pub h1: String,
    pub h2: String,
}

impl HeaderDupletDto {
    pub fn new(row1: impl Into<String>, row2: impl Into<String>) -> Self {
        Self { h1: row1.into(), h2: row2.into() }
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



#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HeaderDto {
    pub individual_header: Vec<HeaderDupletDto>,
    pub data: Vec<HeaderDupletDto>,
}

impl HeaderDto {
    pub fn mendelian(individual_header: Vec<HeaderDupletDto>,data: Vec<HeaderDupletDto>) -> Self {
        Self { individual_header, data }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateDto {
    pub cohort_type: String,
    pub header: HeaderDto,
    pub rows: Vec<RowDto>
}

impl TemplateDto {
    pub fn mendelian(header: HeaderDto, rows: Vec<RowDto>) -> Self {
        Self { cohort_type: "mendelian".to_string(), header, rows }
    }
    
}