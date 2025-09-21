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
    cohort_qc.qc_check(cohort_dto)?;
    cohort_qc.qc_conflicting_pairs(cohort_dto)
}

/// Sanitizes and validates cohort data using HPO ontology validation rules.
///
/// This function attempts to clean the provided cohort data by applying
/// ontology redundancy filters to ensure consistency of the annotations with respect to 
/// the HPO (Human Phenotype Ontology). If validation
/// succeeds, it returns the sanitized data; otherwise, it falls back to returning
/// the original data unchanged.
///
/// # Arguments
///
/// * `hpo` - A thread-safe reference to the full HPO ontology used for validation
/// * `cohort_dto` - Reference to the cohort data structure to be sanitized
///
/// # Returns
///
/// A `CohortData` instance containing either:
/// - The sanitized and validated cohort data (if validation succeeds)
/// - A clone of the original cohort data (if validation fails)
///
/// # Notes
///
/// - This function never panics - it will always return a valid `CohortData` instance
/// - 1. observed with observed ancestor. If a term (say Nuclear cataract) and an ancestor term (e.g. Cataract) are both present (redudant), change the cell value for the ancestor term to Na.
/// - 2. observed with excluded ancestors. If some term is observed, then all of its ancestors are inferred to be also observed. If an ancestor is found to be excluded, this is a conflict. We assume that this comes from different columns of the input data, and change the excluded ancestor to Na to remove the conflict.
/// - 3. excluded with excluded descendent. This is analogous to item 1.
/// - 4. excluded with observed descendent. This is analogous to item 2.
///
/// # See Also
///
/// * [`CohortDataQc::new`] - Creates a new quality control validator
/// * [`CohortDataQc::sanitize`] - Performs the actual sanitization logic
pub fn sanitize_cohort_data(
    hpo: Arc<FullCsrOntology>,
    cohort_dto: &CohortData)
-> Result<CohortData, String> {
    let cohort_qc = CohortDataQc::new(hpo);
    cohort_qc.sanitize(cohort_dto)
}
