use std::{collections::HashSet, sync::Arc};
use std::sync::LazyLock;
use ontolius::{TermId, ontology::{HierarchyWalks, OntologyTerms, csr::FullCsrOntology}, term::MinimalTerm};

use crate::dto::hpo_term_dto::HpoTermDuplet;

 pub static CLINICAL_MODIFIER: LazyLock<TermId> = LazyLock::new(|| {
    let term_id: TermId = "HP:0012823"
        .parse()
        .expect("Critical: Could not parse hardcoded CLINICAL_MODIFIER ID");
    term_id
 });

 pub static ONSET_TERM: LazyLock<TermId> = LazyLock::new(|| {
    let term_id: TermId = "HP:0003674"
        .parse()
        .expect("Critical: Could not parse hardcoded ONSET ID");
    term_id
 });

 pub static MORTALITY_AGING: LazyLock<TermId> = LazyLock::new(||{
     let term_id: TermId = "HP:0040006"
        .parse()
        .expect("Critical: Could not parse hardcoded Mortality/Aging ID");
    term_id
 });

 pub static EXCLUDED_GROUPING_TERMS: LazyLock<HashSet<TermId>> = LazyLock::new(||{
    let mut excluded: HashSet<TermId> = HashSet::new();
    for (label, tid_str) in vec![
        ("Clinical course","HP:0031797"),
        ("Temporal pattern", "HP:0011008"),
        ("Pace of progression", "HP:0003679"),
        ("Position", "HP:0012830")]{
         let term_id: TermId = tid_str
            .parse()
            .unwrap_or_else(|_| panic!("Critical: Could not parse hardcoded {} ID", label));
        excluded.insert(term_id);
    }
    excluded
 });




/// Return a list of HPO modifier terms
/// Do not include terms related to onset or mortality

 pub fn get_modifiers(hpo: Arc<FullCsrOntology>) -> Result<Vec<HpoTermDuplet>, String> {
    let mut excluded: HashSet<TermId> = HashSet::new();
    excluded.insert((*ONSET_TERM).clone());
    hpo.iter_descendant_ids(&*ONSET_TERM)
        .for_each(|tid| {
            excluded.insert(tid.clone());
    });
    excluded.insert((*MORTALITY_AGING).clone());
    hpo.iter_descendant_ids(&*MORTALITY_AGING)
        .for_each(|tid| {
            excluded.insert(tid.clone());
    });
    excluded.extend(EXCLUDED_GROUPING_TERMS.iter().cloned());

    hpo.iter_descendant_ids(&*CLINICAL_MODIFIER)
        .filter(|tid|  ! excluded.contains(tid) )
        .filter(|tid| hpo.iter_child_ids(*tid).count() == 0) // remove non-leaf
        .map(|tid| {
            hpo.term_by_id(tid)
                .map(|term| HpoTermDuplet::new(term.name(), tid.to_string()))
                .ok_or_else(|| format!("Could not retrieve term for {}", tid))
        })
        .collect() 
}


#[cfg(test)]
mod tests {
    use ontolius::ontology::csr::FullCsrOntology;
    use std::sync::Arc;
    use crate::hpo::hpo_modifiers::get_modifiers;
    use crate::test_utils::fixtures::hpo;
    use rstest::rstest;



    #[rstest]
    #[case("Moderate", "HP:0012826", true)]
    #[case("Mild", "HP:0012825", true)]
    #[case("Ameliorated by ethosuximide", "HP:0034759", true)]
    #[case("Bronchocentric", "HP:0033815", true)]
    #[case("Pain exacerbated by wrist radial deviation", "HP:6001152", true)]
    #[case("Late onset", "HP:0003584", false)]
    #[case("Neonatal onset", "HP:0003623", false)]
    #[case("Neonatal death", "HP:0003811", false)]
    #[case("Long philtrum", "HP:0000343", false)]
    #[case("Clinical course","HP:0031797", false)]
    #[case("Temporal pattern", "HP:0011008", false)]
    #[case("Pace of progression", "HP:0003679", false)]
    #[case("Position", "HP:0012830", false)]
    fn test_modifiers(
        hpo: Arc<FullCsrOntology>,
        #[case]label: &str,
        #[case] tid_str: &str,
        #[case] is_valid_modifier: bool) {
        let modifiers = get_modifiers(hpo).expect("get modifiers failed");
        //Moderate  HP:0012826
        let included = modifiers.iter().any(|duplet| duplet.hpo_id == tid_str);
        assert_eq!(is_valid_modifier, included, 
            "{}: modifier status {} but we got {}", label, is_valid_modifier, included);
    }


}