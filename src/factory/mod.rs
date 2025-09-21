use std::sync::Arc;

use ontolius::ontology::csr::FullCsrOntology;

use crate::{dto::cohort_dto::CohortData, factory::{cohort_qc::CohortDataQc}};

pub mod disease_bundle;
pub mod excel;
pub mod gene_variant_bundle;
pub mod header_duplet_row;
pub mod individual_bundle;
pub mod phetools;
pub mod cohort_factory;
mod cohort_qc;



/// Generate the filename for a JSON cohort template.
///
/// The template filename follows the convention:
/// 
/// ```text
/// {gene_symbol}_{cohort_acronym}_individuals.json
/// ```
///
/// # Arguments
/// * `cohort_dto` - A [`CohortData`] object representing the cohort.
///
/// # Returns
/// * `Ok(String)` containing the template filename if the cohort is Mendelian,
///   contains exactly one gene transcript, and has a cohort acronym.
/// * `Err(String)` if:
///   - The cohort is not Mendelian,
///   - No disease data is available,
///   - More than one gene transcript is present (non-Mendelian case not implemented),
///   - Or the cohort acronym is missing.
///
/// # Errors
/// This function returns an error with a descriptive message when the filename
/// cannot be generated. 
///
/// # Example
/// ```ignore
/// let filename = extract_template_name(&cohort_data)?;
/// // e.g., "ACVR1_FOP_individuals.json"
/// ```
pub fn extract_template_name(cohort_dto: &CohortData) -> Result<String, String> {
    if ! cohort_dto.is_mendelian() {
        return Err(format!("Templates are not supported for non-Mendelian inheritance.")); 
    };
    let disease_data = match cohort_dto.disease_list.first() {
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


pub fn qc_assessment(
    hpo: Arc<FullCsrOntology>,
    cohort_dto: &CohortData)
-> Result<(), String> {
    let cohort_qc = CohortDataQc::new(hpo);
    cohort_qc.qc_check(cohort_dto)
}


pub fn sanitize(hpo: Arc<FullCsrOntology>,
    cohort_dto: &CohortData)
-> CohortData {
    let cohort_qc = CohortDataQc::new(hpo);
    match cohort_qc.sanitize(cohort_dto) {
        Ok(dto) =>   dto.clone(),
        Err(_) => cohort_dto.clone()
    }
}
