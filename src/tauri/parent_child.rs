use std::{str::FromStr, sync::Arc};

use ontolius::{TermId, ontology::{HierarchyWalks, OntologyTerms, csr::FullCsrOntology}, term::MinimalTerm};

use crate::tauri::models::{HierarchyMapItem, HpoTermMinimal};


/// Retrieves the immediate parent and children terms for a given Human Phenotype Ontology (HPO) term.
///
/// This function looks up an HPO term by its string identifier within the provided ontology graph 
/// and populates a [`HierarchyMapItem`] containing lists of its direct ancestor (parent) 
/// and descendant (children) terms.
///
/// If the provided `term_id` cannot be successfully parsed into a [`TermId`], an empty 
/// [`HierarchyMapItem::default()`] is returned.
///
/// # Arguments
///
/// * `term_id` - A string slice representing the HPO term identifier (e.g., `"HP:0000118"`).
/// * `hpo` - An `Arc` pointer to the [`FullCsrOntology`] graph instance.
///
/// # Returns
///
/// Returns a [`HierarchyMapItem`] containing the original term ID along with vectors of its 
/// parsed parents and children as [`HpoTermMinimal`]. If the term ID is invalid or cannot be found, 
/// the returned struct fields will be empty.
pub fn get_hpo_parent_and_children_terms(
    term_id: &str,
    hpo: Arc<FullCsrOntology> 
) -> HierarchyMapItem {
    let tid = match TermId::from_str(term_id) {
        Ok(tid) => tid,
        Err(_) => return HierarchyMapItem::default(),
    };

    let children: Vec<HpoTermMinimal> = hpo.iter_child_ids(&tid)
        .filter_map(|child_tid| {
            hpo.term_by_id(child_tid).map(|term| HpoTermMinimal {
                term_id: child_tid.to_string(),
                label: term.name().to_string(),
            })
        })
        .collect();

    let parents: Vec<HpoTermMinimal> = hpo.iter_parent_ids(&tid)
        .filter_map(|parent_tid| {
            hpo.term_by_id(parent_tid).map(|term| HpoTermMinimal {
                term_id: parent_tid.to_string(),
                label: term.name().to_string(),
            })
        })
        .collect();

    HierarchyMapItem {
        current_term_id: term_id.to_string(),
        parents,
        children,
    }
}