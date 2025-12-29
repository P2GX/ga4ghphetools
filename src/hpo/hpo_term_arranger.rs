//! # HpoTermArranger
//!
//! Use a DFS to arrange a list of HPO terms for curation into an easily grokable order.
//!
//! Objects of this class are created to perform a DSF to find a good way of arranging HPO term columns
//! We do not need to take ownership of the ontology, therefore indicate explicit lifetime
use std::{collections::HashSet, str::FromStr, sync::Arc};

use ontolius::{
    common::hpo::PHENOTYPIC_ABNORMALITY, ontology::{csr::FullCsrOntology, HierarchyQueries, HierarchyWalks, OntologyTerms}, term::MinimalTerm, TermId
};

use crate::dto::hpo_term_dto::HpoTermDuplet;

/// Arranges HPO terms into a meaningful order for curation using DFS.
pub struct HpoTermArranger {
    hpo: Arc<FullCsrOntology>,
    hpo_curation_term_id_set: HashSet<TermId>,
    errors: Vec<String>,
}

impl HpoTermArranger {
    /// Create a new HpoTermArranger
    ///
    /// We use a specified life time so we can borrow a reference to the ontology.
    ///
    /// # Arguments
    ///
    /// * `ontology` - reference to an Ontolius HPO ontology.
    pub fn new(ontology: Arc<FullCsrOntology>) -> Self {
        Self {
            hpo: ontology,
            hpo_curation_term_id_set: HashSet::new(),
            errors: Vec::new(),
        }
    }

    /// Perform a depth-first search to arrange the terms for curation into an order that
    /// tends to keep related terms together
    /// We only store the terms we are interested in in ordered_tids.
    fn dfs(
        &mut self,
        start_tid: &TermId,
        visited: &mut HashSet<TermId>,
        ordered_tids: &mut Vec<TermId>,
    ) {
        if visited.contains(start_tid) {
            return;
        }

        visited.insert(start_tid.clone());

        if !self
            .hpo
            .is_equal_or_descendant_of(start_tid, &PHENOTYPIC_ABNORMALITY)
        {
            self.errors.push(format!(
                "TermId {} does not belong to phenotypic abnormality subhierarchy",
                start_tid
            ));
            return;
        }

        if self.hpo_curation_term_id_set.contains(start_tid) {
            ordered_tids.push(start_tid.clone()); // Only include terms we want to curate!
        }

        let children: Vec<TermId> = self.hpo.iter_child_ids(start_tid).cloned().collect();
        for child in &children {
            self.dfs(child, visited, ordered_tids);
        }
    }

    /// Arrange the terms chosen for the template using Depth-First Search (DFS)
    ///
    /// Perform separate DFS for Neoplasm to arrange all neoplasm terms together
    ///
    /// # Arguments
    ///
    /// * `hpo_terms_for_curation` - TermIds of the Terms chosen to initialize the template, generally terms we expect to curate
    ///
    /// * Returns:
    ///
    /// A Vector of TermIds in the order that they should be displayed in the template
    pub fn arrange_term_ids(&mut self, hpo_terms_for_curation: &Vec<TermId>) 
    -> Vec<TermId> {
        self.hpo_curation_term_id_set.clear();
        for smt in hpo_terms_for_curation {
            self.hpo_curation_term_id_set.insert(smt.clone());
        }

        let neoplasm = TermId::from_str("HP:0002664").unwrap();
        let mut visited: HashSet<TermId> = HashSet::new();
        let mut ordered_term_id_list: Vec<TermId> = Vec::new();
        let mut neoplasm_terms = Vec::new();
        // First get any Neoplasm terms
        self.dfs(&neoplasm, &mut visited, &mut neoplasm_terms);
        // then arrange the remaining terms according to organ system
        self.dfs(&PHENOTYPIC_ABNORMALITY, &mut visited, &mut ordered_term_id_list);
        ordered_term_id_list.extend(neoplasm_terms);
        ordered_term_id_list
    }


    pub fn arrange_terms(
        &mut self, 
        hpo_terms_for_curation: &Vec<TermId>)
    -> std::result::Result<Vec<HpoTermDuplet>, String> {
        let arranged_tids = self.arrange_term_ids(hpo_terms_for_curation);
        let mut arranged_terms: Vec<HpoTermDuplet> = Vec::new();
        for tid in arranged_tids {
            match self.hpo.term_by_id(&tid) {
                Some(term) => {
                    arranged_terms.push(HpoTermDuplet::new(term.name(), tid.to_string()));
                },
                None => {
                    return Err(format!("arrange_terms, could not find {}", &tid));
                }
            }
        }
        Ok(arranged_terms)
    }

}

#[cfg(test)]
mod tests {
    use std::time::Instant;
    use crate::test_utils::fixtures::hpo;
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[ignore]
    fn test_term_rerrange(hpo: Arc<FullCsrOntology>) {
        let liver_leiomyoma = "HP:4000154".parse().unwrap();
        let renal_cortical_hyperechogenicity = TermId::from_str("HP:0033132").unwrap();
        let gait_ataxia = TermId::from_str("HP:0002066").unwrap();
        let vsd = TermId::from_str("HP:0001629").unwrap();
        let dysarthria = TermId::from_str("HP:0001260").unwrap();
        let subvalvular_as = TermId::from_str("HP:0001682").unwrap();
        let absent_epiphysis = TermId::from_str("HP:0009321").unwrap();
        let gdd = TermId::from_str("HP:0001263").unwrap();
        let hepatic_hemangioma = TermId::from_str("HP:0031207").unwrap();
        let p_wave_inversion = TermId::from_str("HP:0031600").unwrap();
        let renal_cell_carcinoma = TermId::from_str("HP:0005584").unwrap();
        let renal_corticomedullary_cysts = TermId::from_str("HP:0000108").unwrap();
        let portal_vein_hypoplasia = TermId::from_str("HP:0034548").unwrap();
        let myocardial_sarcomeric_disarray = TermId::from_str("HP:0031333").unwrap();
        let fractured_thumb_phalanx = TermId::from_str("HP:0041239").unwrap();
        let term_list = vec![
            liver_leiomyoma,
            absent_epiphysis,
            gait_ataxia,
            myocardial_sarcomeric_disarray,
            renal_cell_carcinoma,
            vsd,
            dysarthria,
            renal_cortical_hyperechogenicity,
            subvalvular_as,
            fractured_thumb_phalanx,
            hepatic_hemangioma,
            gdd,
            p_wave_inversion,
            renal_corticomedullary_cysts,
            portal_vein_hypoplasia,
        ];
        let start = Instant::now();
        let mut arranger = HpoTermArranger::new(hpo.clone());
        let ordered_terms = arranger.arrange_term_ids(&term_list);
        let duration = start.elapsed();
        for t in ordered_terms {
            let result = hpo.term_by_id(&t);
            match result {
                Some(term) => println!("{} ({})", term.name(), t),
                None => eprint!("Could not retrieve term for {}.", t),
            }
        }
    }
}
