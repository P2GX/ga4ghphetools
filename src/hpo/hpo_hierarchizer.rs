//! HPO Hierarchizer
//! Function to create a map with the top-level terms for each term in a list of HPO Term Duplets (e.g., from a cohort)
use std::{collections::HashMap, str::FromStr, sync::Arc};
use ontolius::{ontology::{csr::FullCsrOntology, HierarchyQueries, HierarchyWalks, OntologyTerms}, term::MinimalTerm, TermId};
use crate::dto::hpo_term_dto::HpoTermDuplet;

/// Convenience structure, not to be used outside of this module.
struct HpoTriplet {
    duplet: HpoTermDuplet,
    tid: TermId
}



fn get_triplet_from_duplet(
    duplet: &HpoTermDuplet
) -> Result<HpoTriplet, String> {
    let tid = TermId::from_str(&duplet.hpo_id).map_err(|e|e.to_string())?;
    Ok(HpoTriplet { duplet: duplet.clone(), tid })
}

fn get_triplet_from_tid(
    tid: &TermId,
    hpo: Arc<FullCsrOntology>
)-> Result<HpoTriplet, String> {
    let term = match hpo.term_by_id(tid) {
        Some(hpo_term) => hpo_term.clone(),
        None => { return Err(format!("Could not retrieve term for {}", tid));}
    };
    let duplet = HpoTermDuplet{
        hpo_id: tid.to_string(),
        hpo_label: term.name().to_string()
    };
    Ok(HpoTriplet { duplet, tid: tid.clone() })
}



/// We want to get a map of HPO term according to top level terms
/// For instance, Arachnodactyly HP:0001166 should be classified
/// as a descendent of Abnormality of the musculoskeletal system HP:0033127
/// This can be used to focus on the terms for a specific organ system in the front end.
pub fn get_hpo_terms_by_toplevel(
    hpo_duplets: Vec<HpoTermDuplet>,
    hpo: Arc<FullCsrOntology>
) -> Result<HashMap<HpoTermDuplet, Vec<HpoTermDuplet>>, String> {
    let mut by_top_level_map: HashMap<HpoTermDuplet, Vec<HpoTermDuplet>> = HashMap::new();
    let pheno_abnormality = TermId::from_str("HP:0000118").map_err(|e| e.to_string())?;
    let top_level_terms: Vec<HpoTriplet> = hpo
        .iter_child_ids(&pheno_abnormality)
        .map(|tid| get_triplet_from_tid(tid, hpo.clone()))
        .collect::<Result<Vec<_>, _>>()?;
    let cohort_terms = hpo_duplets
        .iter()
        .map(|duplt| get_triplet_from_duplet(&duplt))
         .collect::<Result<Vec<_>, _>>()?;
    for cohort_term in cohort_terms {
        for top_level in &top_level_terms {
            if hpo.is_descendant_of(&cohort_term.tid, &top_level.tid) {
                by_top_level_map
                    .entry(top_level.duplet.clone())
                    .or_insert_with(Vec::new)
                    .push(cohort_term.duplet.clone());
            }
        }
    }
    Ok(by_top_level_map)
}


#[cfg(test)]
mod test {
    use ontolius::{io::OntologyLoaderBuilder};
    use super::*;
    use std::{fs::File, io::BufReader};
    use rstest::{fixture, rstest};
    use flate2::bufread::GzDecoder;


  #[fixture]
    fn hpo() -> Arc<FullCsrOntology> {
        let path = "resources/hp.v2025-03-03.json.gz";
        let reader = GzDecoder::new(BufReader::new(File::open(path).unwrap()));
        let loader = OntologyLoaderBuilder::new().obographs_parser().build();
        let hpo = loader.load_from_read(reader).unwrap();
        Arc::new(hpo)
    }

    #[rstest]
    fn test_hierarchizer(
        hpo: Arc<FullCsrOntology>
    ) {
       let arachnodactyly = HpoTermDuplet { 
        hpo_label: "Arachnodactyly".to_string(), 
        hpo_id: "HP:0001166".to_string() 
       };
           
       let musculoskel = HpoTermDuplet { 
        hpo_label: "Abnormality of the musculoskeletal system".to_string(), 
        hpo_id: "HP:0033127".to_string() 
       };

       let asd = HpoTermDuplet {
        hpo_id: "HP:0001631".to_string(),
        hpo_label: "Atrial septal defect".to_string(),
       };
       let cv = HpoTermDuplet{
        hpo_id: "HP:0001626".to_string(),
        hpo_label: "Abnormality of the cardiovascular system".to_string()
       };
       let limbs = HpoTermDuplet {
        hpo_id: "HP:0040064".to_string(),
        hpo_label: "Abnormality of limbs".to_string()
       };
  
       let result = get_hpo_terms_by_toplevel(vec![arachnodactyly, asd], hpo.clone());
       assert!(result.is_ok());
       let hpo_map = result.unwrap();
       println!("{:?}", hpo_map);
  
       assert_eq!(3, hpo_map.len());
       assert!(hpo_map.contains_key(&musculoskel));
       let skel_vec = hpo_map.get(&musculoskel).unwrap();
       assert_eq!(1, skel_vec.len());
       assert!(hpo_map.contains_key(&limbs));
       let limbs_vec = hpo_map.get(&limbs).unwrap();
       assert_eq!(1, limbs_vec.len());
       assert!(hpo_map.contains_key(&cv));
       let cv_vec = hpo_map.get(&cv).unwrap();
       assert_eq!(1, cv_vec.len());
    }

}