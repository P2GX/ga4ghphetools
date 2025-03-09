use ontolius::io::OntologyLoaderBuilder;
use ontolius::ontology::csr::MinimalCsrOntology;
use ontolius::prelude::*;
use std::collections::HashMap;


/// We offer a simple HPO implementation that checks validity of individual Term identifiers and labels
/// We may also implement a version that keeps track of the Ontology object to perform other checks in the future TODO
pub trait HPO {
    // Define methods that types implementing the trait must provide
    fn is_valid_term_id(&self, tid: &str) -> Result<bool, String>;
    fn is_valid_term_label(&self, tid: &str, label: &str) -> Result<bool, String>;

}


/// The purpose of this struct is to extract all terms from the Human Phenotype Ontology (HPO) JSON file
/// 
/// The rest of the application does not perform ontology analysis, instead, we demand that
/// HPO columns contain the correct HPO identifier and label. If an out-of-date identifier is
/// used then we output an error message that allows the user to find the current identifier. 
/// Likewise if the identifier is correct but the label is incorrect, we output the correct
/// label to help the user to correct the error in the template input file.
pub struct SimpleHPO {
    obsolete_d: HashMap<String, String>,
    tid_to_label_d: HashMap<String, String>,
}

impl HPO for SimpleHPO {
    fn is_valid_term_id(&self, tid: &str) -> Result<bool, String> {
        if self.tid_to_label_d.contains_key(tid) {
            return Ok(true);
        } else if self.obsolete_d.contains_key(tid) {
            return Err(format!("Obsolete term id: {} -> replace with {:?}",
                tid,
                self.obsolete_d.get(tid)
            ));
        } else {
            Err(format!("Unrecognized HPO Term ID '{}'", tid))
        }
    }

    fn is_valid_term_label(&self, tid: &str, label: &str) -> Result<bool, String> {
        if self.tid_to_label_d.contains_key(tid) {
            let expected = self.tid_to_label_d.get(tid).expect("could not retrieve label (should never happen)");
            if expected == label {
                return Ok(true);
            } else {
                return Err(format!("Wrong label for {}. Expected '{}' but got '{}'",
                    tid,
                    expected,
                    label));
            }
        } else {
            Err(format!("Invalid HPO id {}", tid))
        }
    }
}


impl SimpleHPO {

    pub fn new(hpo_json_path: &str) -> Result<Self, String> {
        let loader = OntologyLoaderBuilder::new()
                .obographs_parser::<usize>() 
               .build();
            let hpo: MinimalCsrOntology = loader.load_from_path(hpo_json_path)
                                                .expect("HPO should be loaded");
            let mut obsolete_identifiers: HashMap<String,String> = HashMap::new();
            let mut tid_to_label_d: HashMap<String,String> = HashMap::new();
            for term_id in hpo.iter_all_term_ids() {
                let primary_tid = hpo.primary_term_id(term_id);
                match primary_tid {
                    Some(primary_hpo_id) => {
                        if term_id != primary_hpo_id {
                            obsolete_identifiers.insert(term_id.to_string(), primary_hpo_id.to_string());
                        } else {
                            let term = hpo.id_to_term(term_id);
                            match term {
                                Some(term) => {
                                    tid_to_label_d.insert(term_id.to_string(), term.name().to_string());
                                } 
                                None => return Err(format!("Could not retrieve Term for {}", term_id))// should never happen
                            }
                        }
                    },
                    None => return Err(format!("Could not retrieve primary ID for {}", term_id))// should never happen
                } 
            }
           return Ok(SimpleHPO{obsolete_d: obsolete_identifiers, tid_to_label_d: tid_to_label_d});
        }


    
}