//! Factory functions for creating CohortData objects from Excel pyphetools or external input files.
//! 


use std::sync::Arc;
use ontolius::ontology::csr::FullCsrOntology;
use crate::{dto::{cohort_dto::{CohortData, IndividualData}, etl_dto::ColumnTableDto, hpo_term_dto::HpoTermData}, factory::{cohort_factory::CohortFactory, cohort_qc::CohortDataQc}};

pub(crate) mod disease_bundle;
pub mod excel;
pub mod gene_variant_bundle;
pub mod header_duplet_row;
pub(crate) mod individual_bundle;
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


/// Adds a new phenopacket row to an existing cohort.
///
/// This function is called when the user enters information for a new phenopacket
/// that should be added to an existing cohort.  
///
/// The cohort is represented by `cohort_data`, which serves as the source of truth
/// for cohort information (derived from front end). The data is updated here and then returned to the frontend.  
///
/// If no prior data exists, it will be used to seed a new template.  
///
/// # Arguments
///
/// * `hpo` - Reference to the full HPO ontology.
/// * `individual_data` - Metadata for the new row, including PMID, individual, and demographics.
/// * `hpo_annotations` - Observed or excluded HPO terms for the individual.
/// * `variant_key_list` - List of gene/variant identifiers for the new row.
/// * `cohort_data` - Existing cohort data (source of truth) to which the new row will be added.  
///
/// # Returns
///
/// * `Ok(CohortData)` - The updated cohort, if successful.  
/// * `Err(String)` - An error message if the operation fails (e.g., unsupported cohort type).  
pub fn add_new_row_to_cohort(
    hpo: Arc<FullCsrOntology>,
    individual_data: IndividualData, 
    hpo_annotations: Vec<HpoTermData>,
    variant_key_list: Vec<String>,
    cohort_data: CohortData) 
-> Result<CohortData, String> {
    if ! cohort_data.is_mendelian() {
        return Err("add new row not implmented yet for non-Mendelian".to_string());
    }
    let mut builder = CohortFactory::new(hpo);
    builder.add_new_row_to_cohort(individual_data, hpo_annotations, variant_key_list, cohort_data)
}

/// Reads an **external Excel file** for ETL purposes and converts it into
/// a `ColumnTableDto` suitable for further transformation in the ETL pipeline.
///
/// This function is intended for external Excel files (**Not the internal phenopacket-store templates**).
/// The output separates columns into DTOs that include:
/// - `column_type` (initially set to `Raw`)
/// - `transformed` flag (initially `false`)
/// - `header` string
/// - `values` vector for each column
///
/// # Parameters
/// - `file_path`: Path to the Excel file to read.
/// - `row_based`: Determines whether the Excel file is already **row-based** (`true`) or **column-major** (`false`).
///   - If `false`, the function will **transpose** the matrix so that each vector represents a column.
///
/// # Behavior
/// 1. Reads the first worksheet from the Excel file via [`get_list_of_rows_from_excel`].
/// 2. Ensures the file has at least 3 rows; otherwise returns an error.
/// 3. Optionally transposes the matrix if `row_based` is `false`.
/// 4. Uses the first row as headers.
/// 5. Remaining rows are treated as data, mapped into `ColumnDto` structs.
/// 6. Each column gets a `Vec<String>` of values, maintaining order.
///
/// # Returns
/// On success, returns a `ColumnTableDto` containing:
/// - `file_name`: the input file path as string
/// - `columns`: vector of `ColumnDto`, each representing a column with header and values.
///
/// # Errors
/// Returns `Err(String)` if:
/// - The file cannot be opened or read (from `get_list_of_rows_from_excel`)
/// - The file has fewer than 3 rows
pub fn read_external_excel_file(
    file_path: &str, 
    row_based: bool
) -> Result<ColumnTableDto, String> {
    excel::read_external_excel_to_dto(file_path, row_based)
}
