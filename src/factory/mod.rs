use crate::dto::cohort_dto::CohortData;

pub mod disease_bundle;
pub mod excel;
pub mod gene_variant_bundle;
pub mod header_duplet_row;
pub mod individual_bundle;
pub mod phetools;
pub mod cohort_factory;




/// create name of JSON cohort template file, {gene}_{disease}_individuals.json
fn extract_template_name(cohort_dto: &CohortData) -> Result<String, String> {
    
    if ! cohort_dto.is_mendelian() {
        return Err(format!("Templates are not supported for non-Mendelian inheritance.")); 
    };
    let disease_data = match &cohort_dto.disease_list.first() {
        Some(data) => data.clone(),
        None => { return Err(format!("Could not extract disease data from Mendelian cohort"));},
    };
    if disease_data.gene_transcript_list.len() != 1 {
        return Err(format!("Todo-code logic for non-Mendelian templates.")); 
    };
    let symbol = &disease_data.gene_transcript_list[0].gene_symbol;
    match &cohort_dto.cohort_acronym {
        Some(acronym) => Ok(format!("{}_{}_individuals.json", symbol, acronym)),
        None => Err(format!("Cannot get template name if acronym is missing.")),
    }

}
