//! # ETL DTO Module
//!
//! This module provides functionality for transforming heterogeneous external
//! tabular data into a standardized [`CohortData`] format.  
//!
//! ## Purpose
//!
//! In biomedical and clinical research, supplemental files describing cohorts
//! are often provided as tables. These tables can vary widely in structure and
//! column naming, but they usually contain information such as:
//!
//! - Human Phenotype Ontology (HPO) terms
//! - Sets of HPO terms
//! - Individual identifiers
//! - Demographics (e.g., sex, age)
//! - Genetic variants
//!
//! The goal of this module is to normalize such diverse sources into a unified
//! representation that can be consumed by downstream analyses.
//!
//! ## Workflow
//!
//! 1. **Initial ingestion**: The external table is first wrapped in an
//!    [`EtlDto`] structure, which captures the table contents in a flexible,
//!    intermediate form.
//! 2. **Column transformations**: Each column of the `EtlDto` is progressively
//!    mapped into standardized representations (e.g., HPO sets, identifiers).
//! 3. **Finalization**: Once all relevant columns have been transformed, the
//!    resulting data is consolidated into a [`CohortData`] structure.
//!
//! This stepwise approach makes it possible to adapt to the many different
//! formats found in real-world supplemental datasets, while still converging
//! to a consistent internal representation.
//!
//! ## Key Types
//!
//! - [`EtlDto`]: The intermediate representation of an external table.
//! - [`CohortData`]: The normalized, final representation of a cohort.

use std::sync::Arc;

use ontolius::ontology::csr::FullCsrOntology;

use crate::{dto::{cohort_dto::CohortData, etl_dto::EtlDto}, etl::etl_tools::EtlTools};



mod etl_tools;

/// Transform an [`EtlDto`] into a [`CohortData`] structure.
///
/// This function takes as input:
/// - An [`Arc`] to the [`FullCsrOntology`] representation of the Human Phenotype Ontology.
/// - An [`EtlDto`] representing the external table to be transformed.
///
/// Internally, this function uses [`EtlTools`] to progressively map columns
/// from the `EtlDto` into standardized forms, and finally produces a
/// [`CohortData`] instance.
///
/// # Errors
///
/// Returns an `Err(String)` if the transformation fails (e.g., due to
/// malformed input or missing ontology references).
///
/// # Note
/// 
/// This function should be called after the stepwise transformation of the EtlDto has
/// been performed. If this is not the case, an Error will be thrown.
/// ```
pub fn get_cohort_data_from_etl_dto(
    hpo: Arc<FullCsrOntology>,
    dto: EtlDto,
) -> Result<CohortData, String> {
    let mut etl_tools = EtlTools::from_dto(hpo, &dto);
    etl_tools.get_cohort_data()
}


pub fn process_allele_column(
    hpo: Arc<FullCsrOntology>,
    etl: EtlDto,
    col: usize
) -> Result<EtlDto, String> {
    let etl_tools = EtlTools::from_etl(etl, hpo);
    etl_tools.process_allele_column(col)
}
