use std::{str::FromStr, sync::Arc};

use ontolius::{TermId, ontology::{HierarchyWalks, OntologyTerms, csr::FullCsrOntology}, term::MinimalTerm};
use crate::dto::hpo_term_dto::HpoTermDuplet;
use crate::tauri::models::{HierarchyMapItem};


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

    let children: Vec<HpoTermDuplet> = hpo.iter_child_ids(&tid)
        .filter_map(|child_tid| {
            hpo.term_by_id(child_tid).map(|term| 
                HpoTermDuplet::new(term.name(), child_tid.to_string()))
        })
        .collect();

    let parents: Vec<HpoTermDuplet> = hpo.iter_parent_ids(&tid)
        .filter_map(|parent_tid| {
            hpo.term_by_id(parent_tid).map(|term| 
                HpoTermDuplet::new(term.name(), parent_tid.to_string()))
        })
        .collect();

    HierarchyMapItem {
        current_term_id: term_id.to_string(),
        parents,
        children,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::fixtures::hpo;
    use rstest::rstest;

    /// Find a term by id in a list of duplets
    fn find<'a>(items: &'a [HpoTermDuplet], id: &str) -> Option<&'a HpoTermDuplet> {
        items.iter().find(|t| t.hpo_id() == id)
    }

    #[rstest]
    fn parent_of_phenotypic_abnormality_is_root(hpo: Arc<FullCsrOntology>) {
        let item = get_hpo_parent_and_children_terms("HP:0000118", hpo);
        assert_eq!(item.current_term_id, "HP:0000118");
        assert_eq!(item.parents.len(), 1);
        assert_eq!(item.parents[0].hpo_id(), "HP:0000001");
        assert_eq!(item.parents[0].hpo_label(), "All");
    }

    #[rstest]
    fn root_has_no_parents_and_expected_children(hpo: Arc<FullCsrOntology>) {
        let item = get_hpo_parent_and_children_terms("HP:0000001", hpo);
        assert!(item.parents.is_empty());
        let pa = find(&item.children, "HP:0000118").expect("Phenotypic abnormality is a child of All");
        assert_eq!(pa.hpo_label(), "Phenotypic abnormality");
        let moi = find(&item.children, "HP:0000005").expect("Mode of inheritance is a child of All");
        assert_eq!(moi.hpo_label(), "Mode of inheritance");
    }

    #[rstest]
    fn id_and_label_are_not_swapped(hpo: Arc<FullCsrOntology>) {
        let item = get_hpo_parent_and_children_terms("HP:0000118", hpo);
        for t in item.parents.iter().chain(item.children.iter()) {
            assert!(t.hpo_id().starts_with("HP:"), "id field holds '{}'", t.hpo_id());
            assert!(!t.hpo_label().starts_with("HP:"), "label field holds '{}'", t.hpo_label());
        }
    }

    /// For every child of a term, that term must appear among the child's parents.
    #[rstest]
    fn parent_and_child_links_are_consistent(hpo: Arc<FullCsrOntology>) {
        let term = "HP:0000118";
        let item = get_hpo_parent_and_children_terms(term, hpo.clone());
        assert!(!item.children.is_empty());
        for child in item.children.iter().take(25) {
            let child_item = get_hpo_parent_and_children_terms(child.hpo_id(), hpo.clone());
            assert!(
                find(&child_item.parents, term).is_some(),
                "{} lists no parent {}", child.hpo_id(), term
            );
        }
    }

    #[rstest]
    fn unparseable_term_id_returns_default(hpo: Arc<FullCsrOntology>) {
        let item = get_hpo_parent_and_children_terms("not a term id", hpo);
        assert!(item.current_term_id.is_empty());
        assert!(item.parents.is_empty());
        assert!(item.children.is_empty());
    }

    /// Well-formed id that is not in the ontology: the term id is echoed, no relatives.
    #[rstest]
    fn unknown_but_wellformed_term_id_gives_empty_lists(hpo: Arc<FullCsrOntology>) {
        let item = get_hpo_parent_and_children_terms("HP:9999999", hpo);
        assert_eq!(item.current_term_id, "HP:9999999");
        assert!(item.parents.is_empty());
        assert!(item.children.is_empty());
    }

    /// Pins the JSON shape the Angular side depends on (hpoId / hpoLabel, no termId / label).
    #[rstest]
    fn serialized_shape_uses_duplet_field_names(hpo: Arc<FullCsrOntology>) {
        let item = get_hpo_parent_and_children_terms("HP:0000118", hpo);
        let json = serde_json::to_value(&item).unwrap();
        assert_eq!(json["currentTermId"], "HP:0000118");
        let parent = &json["parents"][0];
        assert_eq!(parent["hpoId"], "HP:0000001");
        assert_eq!(parent["hpoLabel"], "All");
        assert!(parent.get("termId").is_none());
        assert!(parent.get("label").is_none());
    }
}