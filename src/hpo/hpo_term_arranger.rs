

use std::{collections::HashSet, str::FromStr};

///! HpoTermArranger
/// Use a DFS to arrange a list of HPO terms for curation into an easily grokable order
/// 
/// 
use ontolius::{base::{term::simple::SimpleMinimalTerm, Identified, TermId}, ontology::csr::MinimalCsrOntology, prelude::{ChildNodes, HierarchyAware, Term, TermAware}};

/// Objects of this class are created to perform a DSF to find a good way of arranging HPO term columns
/// We do not need to take ownership of the ontology, therefore indicate explicit lifetime 
pub struct HpoTermArranger<'a> {
    ontology: &'a MinimalCsrOntology,
    hpo_curation_term_id_set: HashSet<TermId>,
}


impl<'a> HpoTermArranger<'a> {

    pub fn new(ontology: &'a MinimalCsrOntology) -> Self {
        Self {
            ontology,
            hpo_curation_term_id_set: HashSet::new()
        }
    }

    /// Perform a depth-first search to arrange the terms for curation into an order that
    /// tends to keep related terms together
    /// We only store the terms we are interested in in ordered_tids.
    fn dfs(&self, start: TermId, visited: &mut HashSet<TermId>, ordered_tids: &mut Vec<TermId>) {
        if visited.contains(&start) {
            return;
        }
        visited.insert(start.clone());
        let result = self.ontology.id_to_idx(&start);
        if result.is_none() {
            return; // consider adding to error vector of struct?
        }
        let idx = result.unwrap();
        if self.hpo_curation_term_id_set.contains(&start) {
            ordered_tids.push(start); // Only include terms we want to curate!
        }
        let hierarchy = self.ontology.hierarchy();
        let children: Vec<&SimpleMinimalTerm> = hierarchy.iter_children_of(idx)
            .flat_map(|idx| self.ontology.idx_to_term(idx))
            .collect();
        for child in children {
            let child_id = child.identifier().clone();
            self.dfs(child_id, visited, ordered_tids);
        }
    }

    pub fn arrange_terms(&mut self, hpo_terms_for_curation: &Vec<TermId>) -> 
    Vec<TermId> {
        self.hpo_curation_term_id_set.clear();
        for smt in hpo_terms_for_curation {
            self.hpo_curation_term_id_set.insert(smt.identifier().clone());
        }
        let phenotypic_abnormality = TermId::from_str("HP:000168").unwrap();
        let mut visited: HashSet<TermId> = HashSet::new();
        let mut result: Vec<TermId> = Vec::new();
        self.dfs(phenotypic_abnormality, &mut visited, &mut result);
        result
    }

}

   
    
