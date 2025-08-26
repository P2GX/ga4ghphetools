//! HPO module
//! 
//! Convenience functions for working with HPO data
use std::{collections::HashMap, sync::Arc};

use ontolius::{ontology::csr::FullCsrOntology, TermId};

use crate::{dto::hpo_term_dto::{HpoTermData, HpoTermDuplet}, hpo::{hpo_term_arranger::HpoTermArranger, hpo_util::HpoUtil}};

mod hpo_term_arranger;
mod hpo_util;



/// Arrange a list of HPO terms into a curator-friendly order using depth-first search (DFS).
///
/// This function traverses the HPO graph with a DFS, ensuring that related terms
/// (i.e. from the same branch of the ontology) appear next to each other in the output.
/// The resulting order helps curators review all relevant terms together.
///
/// # Arguments
///
/// * `hpo` - A reference to the full HPO ontology.
/// * `hpo_terms_for_curation` - The list of HPO term IDs to be arranged.
///
/// # Returns
///
/// A `Vec<TermId>` containing the input terms arranged in DFS order.
pub fn hpo_terms_to_dfs_order(
    hpo: Arc<FullCsrOntology>,
    hpo_terms_for_curation: &Vec<TermId>)
-> Vec<TermId> {
    let hpo_arc = hpo.clone();
    let mut term_arrager = HpoTermArranger::new(hpo_arc);    
    term_arrager.arrange_term_ids(hpo_terms_for_curation)
}


/// Arrange a list of HPO terms into a curator-friendly order using depth-first search (DFS),
/// returning term duplets (ID + label).
///
/// Like [`hpo_terms_to_dfs_order`], this function ensures that related HPO terms
/// (i.e. from the same branch of the ontology) are grouped together.  
/// Instead of returning only `TermId`s, it provides full [`HpoTermDuplet`]s
/// containing both the HPO ID and label.
///
/// # Arguments
///
/// * `hpo` - A reference to the full HPO ontology.
/// * `hpo_terms_for_curation` - The list of HPO term IDs to be arranged.
///
/// # Returns
///
/// * `Ok(Vec<HpoTermDuplet>)` — the input terms arranged in DFS order, with both ID and label.  
/// * `Err(String)` — if any of the term IDs cannot be resolved in the ontology.
///
/// # Errors
///
/// Returns an error if any provided `TermId` is not found in the given ontology.
pub fn hpo_terms_to_dfs_order_duplets(
    hpo: Arc<FullCsrOntology>,
    hpo_terms_for_curation: &Vec<TermId>)
-> Result<Vec<HpoTermDuplet>, String> {
    let hpo_arc = hpo.clone();
    let mut term_arrager = HpoTermArranger::new(hpo_arc);    
    term_arrager.arrange_terms(hpo_terms_for_curation)
}



/// Build a mapping from HPO term IDs to their human-readable labels.
///
/// This function takes a list of [`HpoTermData`] DTOs and returns a
/// [`HashMap`] where:
/// - **Key** = [`TermId`] of each HPO term in the input list
/// - **Value** = corresponding label string for that term
///
/// The mapping is restricted to the HPO terms present in `hpo_dto_list`.
/// The ontology context is provided by the given [`FullCsrOntology`].
///
/// # Arguments
/// * `hpo` - Shared reference to the ontology data structure used for term resolution
/// * `hpo_dto_list` - List of HPO term DTOs to be mapped
///
/// # Returns
/// * `Ok(HashMap<TermId, String>)` if all terms were successfully resolved
/// * `Err(String)` if any term could not be resolved
///
/// # Errors
/// Returns an error string if one or more of the provided terms cannot be
/// matched against the ontology.
pub fn term_label_map_from_dto_list(
    hpo: Arc<FullCsrOntology>,
    hpo_dto_list: &Vec<HpoTermData>
) -> std::result::Result<HashMap<TermId, String>, String> {
    let hpo_util = HpoUtil::new(hpo);
    hpo_util.term_label_map_from_dto_list(hpo_dto_list)
}

/// Update HPO term duplets with the latest labels from the ontology.
///
/// This function refreshes a list of [`HpoTermDuplet`]s by ensuring that
/// each HPO ID is associated with its **current, canonical label** as
/// defined in the provided [`FullCsrOntology`].  
///
/// It is primarily used to migrate or validate legacy Excel-based data
/// against the current ontology.  
///
/// # Arguments
/// * `hpo` - Shared reference to the ontology used for term resolution
/// * `hpo_duplets` - List of HPO term duplets (ID + label) to be updated
///
/// # Returns
/// * `Ok(Vec<HpoTermDuplet>)` - Updated duplets with refreshed labels
/// * `Err(String)` - If a label cannot be matched or updated
///
/// # Notes
/// - For **legacy Excel files**, this function helps reconcile outdated
///   term labels with the ontology.  
/// - For **new JSON templates**, a different, more robust solution will
///   eventually be needed.  
pub fn update_hpo_duplets(
    hpo: Arc<FullCsrOntology>,  
    hpo_duplets: &Vec<HpoTermDuplet>,
) -> std::result::Result<Vec<HpoTermDuplet>, String> {
    let hpo_util = HpoUtil::new(hpo);
    hpo_util.update_hpo_duplets(hpo_duplets)
}