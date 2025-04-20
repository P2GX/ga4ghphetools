//! TODO - obsolete this class, we can do everything with ontolius directly now

use crate::error::{self, Error, Result};
use ontolius::io::OntologyLoaderBuilder;
use ontolius::ontology::csr::{FullCsrOntology, MinimalCsrOntology};
use ontolius::ontology::OntologyTerms;
use ontolius::term::MinimalTerm;
use std::collections::HashMap;

/// We offer a simple HPO implementation that checks validity of individual Term identifiers and labels
/// We may also implement a version that keeps track of the Ontology object to perform other checks in the future TODO
pub trait HPO {
    // Define methods that types implementing the trait must provide
    fn is_valid_term_id(&self, tid: &str) -> Result<bool>;
    fn is_valid_term_label(&self, tid: &str, label: &str) -> Result<bool>;
}


/// The purpose of this struct is to extract all terms from the Human Phenotype Ontology (HPO) JSON file
///
/// The rest of the application does not perform ontology analysis, instead, we demand that
/// HPO columns contain the correct HPO identifier and label. If an out-of-date identifier is
/// used then we output an error message that allows the user to find the current identifier.
/// Likewise if the identifier is correct but the label is incorrect, we output the correct
/// label to help the user to correct the error in the template input file.
#[derive(Debug)]
pub struct SimpleHPOMapper {
    obsolete_d: HashMap<String, String>,
    tid_to_label_d: HashMap<String, String>,
}

impl HPO for SimpleHPOMapper {
    fn is_valid_term_id(&self, tid: &str) -> Result<bool> {
        if self.tid_to_label_d.contains_key(tid) {
            return Ok(true);
        } else if self.obsolete_d.contains_key(tid) {
            match self.obsolete_d.get(tid) {
                Some(replacement) => Err(Error::ObsoleteTermId {
                    id: tid.to_string(),
                    replacement: replacement.clone(),
                }),
                None => Err(Error::ObsoleteTermId {
                    id: tid.to_string(),
                    replacement: "?".to_string(),
                }),
            }
        } else {
            Err(Error::HpIdNotFound {
                id: tid.to_string(),
            })
        }
    }

    fn is_valid_term_label(&self, tid: &str, label: &str) -> Result<bool> {
        if self.tid_to_label_d.contains_key(tid) {
            let expected = self
                .tid_to_label_d
                .get(tid)
                .expect("could not retrieve label (should never happen)");
            if expected == label {
                return Ok(true);
            } else {
                Err(Error::wrong_hpo_label_error(tid, label, expected))
            }
        } else {
            Err(Error::HpIdNotFound {
                id: tid.to_string(),
            })
        }
    }
}

impl SimpleHPOMapper {
    pub fn new(hpo: &FullCsrOntology) -> Result<Self> {
        let mut obsolete_identifiers: HashMap<String, String> = HashMap::new();
        let mut tid_to_label_d: HashMap<String, String> = HashMap::new();
        for term_id in hpo.iter_all_term_ids() {
            let primary_tid = hpo.primary_term_id(term_id);
            match primary_tid {
                Some(primary_hpo_id) => {
                    if term_id != primary_hpo_id {
                        obsolete_identifiers
                            .insert(term_id.to_string(), primary_hpo_id.to_string());
                    } else {
                        let term = hpo.term_by_id(term_id);
                        match term {
                            Some(term) => {
                                tid_to_label_d.insert(term_id.to_string(), term.name().to_string());
                            }
                            None => {
                                return Err(Error::HpIdNotFound {
                                    id: term_id.to_string(),
                                })
                            } // should never happen
                        }
                    }
                }
                None => {
                    return Err(Error::HpIdNotFound {
                        id: term_id.to_string(),
                    })
                } // should never happen
            }
        }
        return Ok(SimpleHPOMapper {
            obsolete_d: obsolete_identifiers,
            tid_to_label_d: tid_to_label_d,
        });
    }
}
