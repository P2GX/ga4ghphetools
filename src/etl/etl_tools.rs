use std::{collections::HashMap, fmt, fs, sync::Arc};

use ontolius::ontology::{csr::FullCsrOntology, MetadataAware};

use crate::{dto::{cohort_dto::{CohortData, CohortType}, etl_dto::{ColumnMetadata, ColumnTableDto}, hpo_term_dto::HpoTermDuplet}, factory::excel, hpo};



pub struct EtlTools {
     /// Reference to the Ontolius Human Phenotype Ontology Full CSR object
    hpo: Arc<FullCsrOntology>,
    raw_table: ColumnTableDto,
}


impl EtlTools {

    pub fn new( hpo: Arc<FullCsrOntology>, excel_file_path: &str, row_based: bool) -> Result<Self, String> {
        let raw_table = excel::read_external_excel_to_df(excel_file_path, row_based)
            .map_err(|e| e.to_string())?;
        Ok(Self { hpo, raw_table })
    }

    pub fn from_dto(hpo: Arc<FullCsrOntology>, dto: &ColumnTableDto) -> Self {
        Self{
            hpo,
            raw_table: dto.clone(),
        }
    }

    pub fn from_json(
        etl_file_path: &str,
        hpo: Arc<FullCsrOntology>
    ) -> Result<Self, String> {
        let table = EtlTools::load_column_table_from_json(etl_file_path)?;
        Ok(Self { raw_table: table, hpo }) 
    }

    pub fn raw_table(&self) -> &ColumnTableDto {
        &self.raw_table
    }

    // Function to load JSON file and deserialize to ColumnTableDto
    pub fn load_column_table_from_json(file_path: &str) -> Result<ColumnTableDto, String> {
        let json_content = fs::read_to_string(file_path)
            .map_err(|e| e.to_string())?;
        let column_table: ColumnTableDto = serde_json::from_str(&json_content)
            .map_err(|e| e.to_string())?;
        Ok(column_table)
    }



    
    /// Retrieve all HPO Duplets from the Single and Multiple HPO columns
    /// We need this to know how many HPO terms we have altogether for the CohortData
    pub fn all_hpo_duplets(&self) -> Vec<HpoTermDuplet> {
        self.raw_table.columns.iter()
            .filter_map(|col| {
                if let ColumnMetadata::HpoTerms(duplets) = &col.header.metadata {
                    Some(duplets.clone())
                } else {
                    None
                }
            })
            .flatten()
            .collect()
    }

     /// Note that only Mendelian is supported for Excel file bulk imports
    /// Ohter MOIs are too complicated to be reliably imported in this way.
    pub fn get_dto(&self) -> Result<CohortData, String> {
        let hpo_duplets = Self::all_hpo_duplets(&self);
        let header = hpo::arrange_hpo_duplets(self.hpo.clone(), &hpo_duplets)?;
        Ok(CohortData { 
            cohort_type: CohortType::Mendelian, 
            disease_list: vec![], 
            hpo_headers: header, 
            rows: vec![], 
            hgvs_variants: HashMap::new(), 
            structural_variants: HashMap::new(), 
            phetools_schema_version: CohortData::phenopackets_schema_version(), 
            hpo_version: self.hpo.version().to_string(), 
            cohort_acronym: None 
        })
    }
  
}




impl fmt::Display for ColumnTableDto {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "File: {}", self.file_name)?;
        writeln!(f, "Columns:")?;

        for column in &self.columns {
            let first_value = column.values.first().cloned().unwrap_or_else(|| "<empty>".to_string());
            writeln!(f, "- {}: {}", column.header.original, first_value)?;
        }

        Ok(())
    }
}

