//! HPO Annotations
//! 
//! This module contains functions for writing a TSV file in the [HPOA format](https://hpo.jax.org/app/help/annotations).
//! 

use std::{path::PathBuf, sync::Arc};

use ontolius::ontology::csr::FullCsrOntology;

use crate::{dto::cohort_dto::CohortData, hpoa::hpoa_table::HpoaTable};


mod counted_hpo_term;
mod hpoa_onset_calculator;
mod hpoa_table;
mod hpoa_table_row;
mod hpo_term_counter;





/// Write HPO annotations to a TSV file in the [HPOA format](https://hpo.jax.org/app/help/annotations).
///
/// # Arguments
///
/// * `cohort` - The cohort data to be annotated.
/// * `hpo` - The ontology used to resolve HPO terms.
/// * `biocurator` - The biocurator identifier to record in the output.
/// * `path` - The file path where the TSV should be written.
///
/// # Returns
///
/// * `Ok(())` if the file was successfully written.
/// * `Err(String)` if an error occurred during table creation or file writing.
pub fn write_hpoa_tsv(
    cohort: CohortData, 
    hpo: Arc<FullCsrOntology>,
    biocurator: &str,
    path: &PathBuf
) -> std::result::Result<(), String> {
    let hpoa = HpoaTable::new(cohort, hpo, biocurator)?;
    hpoa.write_tsv(path).map_err(|e| e.to_string())
}

/// Generate an in-memory dataframe representation of HPO annotations
/// in the [HPOA format](https://hpo.jax.org/app/help/annotations).
///
/// Unlike [`write_hpoa_tsv`], this function does not write to disk but returns
/// a 2D vector of strings suitable for downstream processing.
///
/// # Arguments
///
/// * `cohort` - The cohort data to be annotated.
/// * `hpo` - The ontology used to resolve HPO terms.
/// * `biocurator` - The biocurator identifier to include in the annotation.
///
/// # Returns
///
/// * `Ok(Vec<Vec<String>>)` where each inner vector represents one TSV row.
/// * `Err(String)` if an error occurred during table creation.
pub fn get_hpoa_dataframe(
    cohort: CohortData, 
    hpo: Arc<FullCsrOntology>,
    biocurator: &str,
 )-> std::result::Result<Vec<Vec<String>>, String> {
    let hpoa = HpoaTable::new(cohort, hpo, biocurator)?;
    Ok(hpoa.get_dataframe())
}


