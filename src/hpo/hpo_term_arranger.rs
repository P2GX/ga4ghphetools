

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
        let phenotypic_abnormality = TermId::from_str("HP:000118").unwrap();
        let neoplasm = TermId::from_str("HP:0002664").unwrap();
        let mut visited: HashSet<TermId> = HashSet::new();
        let mut result: Vec<TermId> = Vec::new();
        let mut neoplasm_terms = Vec::new();
        // TODO gather terms in subhierarchy neoplasm 
        self.dfs(neoplasm, &mut visited, &mut neoplasm_terms);
        self.dfs(phenotypic_abnormality, &mut visited, &mut result);
        result.extend(neoplasm_terms);
        result
    }

}

   
#[cfg(test)]
mod tests {
    use std::time::Instant;

    use ontolius::{io::OntologyLoaderBuilder, prelude::MinimalTerm};

    use super::*;

    #[test]
    #[ignore]
    fn test_term_rerrange() {
        let liver_leiomyoma = TermId::from_str("HP:4000154").unwrap();
        let renal_cortical_hyperechogenicity  = TermId::from_str("HP:0033132").unwrap();
        let gait_ataxia = TermId::from_str("HP:0002066").unwrap();
        let vsd = TermId::from_str("HP:0001629").unwrap();
        let dysarthria = TermId::from_str("HP:0001260").unwrap();
        let subvalvular_as = TermId::from_str("HP:0001682").unwrap();
        let absent_epiphysis  = TermId::from_str("HP:0009321").unwrap();
        let gdd = TermId::from_str("HP:0001263").unwrap();
        let hepatic_hemangioma  = TermId::from_str("HP:0031207").unwrap();
        let p_wave_inversion  = TermId::from_str("HP:0031600").unwrap();
        let renal_cell_carcinoma = TermId::from_str("HP:0005584").unwrap();
        let renal_corticomedullary_cysts  = TermId::from_str("HP:0000108").unwrap();
        let portal_vein_hypoplasia = TermId::from_str("HP:0034548").unwrap();
        let myocardial_sarcomeric_disarray = TermId::from_str("HP:0031333").unwrap();
        let fractured_thumb_phalanx  = TermId::from_str("HP:0041239").unwrap();
        let term_list = vec![liver_leiomyoma, absent_epiphysis, gait_ataxia, myocardial_sarcomeric_disarray, renal_cell_carcinoma, vsd, dysarthria, renal_cortical_hyperechogenicity, subvalvular_as, fractured_thumb_phalanx,
            hepatic_hemangioma, gdd, p_wave_inversion,renal_corticomedullary_cysts, portal_vein_hypoplasia];
        let start = Instant::now();
        let loader = OntologyLoaderBuilder::new()
            .obographs_parser()
            .build();
        let hp_json = "/Users/robin/data/hpo/hp.json";
        let hpo: MinimalCsrOntology = loader.load_from_path(hp_json).expect("could not unwrap");
        let duration = start.elapsed();
        println!("Loaded HPO: {:?}", duration);
        let start = Instant::now();
        let mut arranger = HpoTermArranger::new(&hpo);
        let ordered_terms = arranger.arrange_terms(&term_list);
        let duration = start.elapsed();
        println!("Arranged terms: {:?}", duration);
        for t in ordered_terms {
            let result = hpo.id_to_term(&t);
            match result {
                Some(term) => println!("{} ({})",term.name(), t),
                None => eprint!("Could not retrieve term for {}.", t)
            } 
        }
       
    }
} 
