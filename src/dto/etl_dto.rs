
 use serde::{Deserialize, Serialize};




/// DTOs for transforming external Excel tables 
/// We ingest an Excel file and transform it column by column to a structure we can use to import phenopackets.
/// Each column will be transformed one by one. Columns start off as RAW and then are changed to the other
///types listed here
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum EtlColumnType {
    Raw,
    FamilyId,
    PatientId,
    SingleHpoTerm,
    MultipleHpoTerm,
    GeneSymbol,
    Variant,
    Disease,
    Age,
    Sex,
    Ignore
}

 #[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
 pub struct ColumnDto {
    pub column_type: EtlColumnType,
    pub transformed: bool,
    pub header: String,
    pub values: Vec<String>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct ColumnTableDto {
    pub file_name: String,
    pub columns: Vec<ColumnDto>,
    pub total_rows: usize,
    pub total_columns: usize,
}





