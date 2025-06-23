use std::str::FromStr;


use serde::{de, Deserialize, Serialize};
use crate::template::excel::read_excel_to_dataframe;
use crate::error::{Error, Result};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IndividualDto {
    pub pmid: String,
    pub title: String,
    pub individual_id: String,
    pub comment: String,
}

impl IndividualDto {
    pub fn new(
        pmid: String,
        title: String,
        individual_id: String,
        comment: String,) -> Self{
            Self { pmid, title, individual_id, comment }
     }
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
    pub data: Vec<CellDto>
}

impl RowDto {
    pub fn new(individual_dto: IndividualDto, data: Vec<CellDto>) -> Self {
        Self { individual_dto, data }
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