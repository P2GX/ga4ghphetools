use std::{fmt::{self, Display}, sync::Arc};

use ontolius::ontology::csr::FullCsrOntology;

use crate::{dto::etl_dto::{ColumnTableDto, RawTableDto}, template::excel};



pub struct EtlTools {
     /// Reference to the Ontolius Human Phenotype Ontology Full CSR object
    hpo: Arc<FullCsrOntology>,
    raw_table: ColumnTableDto,
}


impl EtlTools {

    pub fn new( hpo: Arc<FullCsrOntology>, excel_file_path: &str) -> Result<Self, String> {
        let raw_table = excel::read_external_excel_to_df(excel_file_path)
            .map_err(|e| e.to_string())?;
        Ok(Self { hpo, raw_table })
    }

    pub fn raw_table(&self) -> &ColumnTableDto {
        &self.raw_table
    }
  
}




impl fmt::Display for ColumnTableDto {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "File: {}", self.file_name)?;
        writeln!(f, "Columns:")?;

        for column in &self.columns {
            let first_value = column.values.first().cloned().unwrap_or_else(|| "<empty>".to_string());
            writeln!(f, "- {}: {}", column.header, first_value)?;
        }

        Ok(())
    }
}

