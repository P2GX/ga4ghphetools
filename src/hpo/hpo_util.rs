//! TODO - obsolete this class, we can do everything with ontolius directly now

use crate::dto::hpo_term_dto::HpoTermData;
use crate::dto::validation_errors::ValidationErrors;
use crate::dto::hpo_term_dto::HpoTermDuplet;
use ontolius::ontology::csr::FullCsrOntology;
use ontolius::ontology::OntologyTerms;
use ontolius::term::MinimalTerm;
use ontolius::TermId;
use std::collections::HashMap;
use std::sync::Arc;



/// The purpose of this struct is to extract all terms from the Human Phenotype Ontology (HPO) JSON file
///
/// The rest of the application does not perform ontology analysis, instead, we demand that
/// HPO columns contain the correct HPO identifier and label. If an out-of-date identifier is
/// used then we output an error message that allows the user to find the current identifier.
/// Likewise if the identifier is correct but the label is incorrect, we output the correct
/// label to help the user to correct the error in the template input file.
pub struct HpoUtil {
    hpo: Arc<FullCsrOntology>
}

impl HpoUtil {
    pub fn new(hpo_arc: Arc<FullCsrOntology>) -> Self {
        Self {
            hpo: hpo_arc
        }
    }

    
    pub fn term_label_map_from_dto_list(
        &self, 
        hpo_dto_list: &Vec<HpoTermData>
    ) -> std::result::Result<HashMap<TermId, String>, String> {
        let mut dto_map: HashMap<TermId, String> = HashMap::new();
        for dto in hpo_dto_list {
            match dto.ontolius_term_id() {
                Ok(term_id) => {dto_map.insert(term_id.clone(), dto.label().to_string());},
                Err(_) => {
                    return Err(format!("Could not map termId: '{}'", dto.term_id()));
                },
            } 
        }
        Ok(dto_map)
    }

    pub fn term_label_map_from_duplet_list(
        &self, 
        hpo_duplet_list: &Vec<HpoTermDuplet>
    ) -> std::result::Result<HashMap<TermId, String>, String> {
        let mut dto_map: HashMap<TermId, String> = HashMap::new();
        for dto in hpo_duplet_list {
            match dto.to_term_id() {
                Ok(term_id) => {dto_map.insert(term_id.clone(), dto.hpo_label().to_string());},
                Err(_) => {
                    return Err(format!("Could not map termId: '{}'", dto.hpo_id()));
                },
            } 
        }
        Ok(dto_map)
    }



    /// Check the validity of the HPO TermId/label pairs in the DTO objects and return corresponding HpoTermDuplet list
    pub fn hpo_duplets_from_dto(&self, hpo_dto_list: &Vec<HpoTermData>) -> std::result::Result<Vec<HpoTermDuplet>, ValidationErrors> {
        let mut hpo_duplets: Vec<HpoTermDuplet> = Vec::with_capacity(hpo_dto_list.len());
        let mut verr = ValidationErrors::new();
        for hpo_dto in hpo_dto_list {
            let tid = match hpo_dto.ontolius_term_id() {
                Ok(tid) => tid,
                Err(e) => {
                    return Err(ValidationErrors::from_one_err(
                    format!("Could not create TermId from {:?}", hpo_dto)));}
            };
            if let Some(term) = self.hpo.term_by_id(&tid) {
                if term.name() != hpo_dto.label() {
                    verr.push_str(format!("Expected label '{}' but got '{}' for TermId '{}'",term.name(), hpo_dto.label(), tid));
                }
                hpo_duplets.push(HpoTermDuplet::new(term.name(), tid.to_string()));
            } else {
                verr.push_str(format!("Could not find term for {}", hpo_dto.term_id()));
            }
        }
        if verr.has_error() {
            Err(verr)
        } else {
            Ok(hpo_duplets)
        }
    }



    /// Update the HPO duplets with the current term names from the ontology
    /// This will automatically update term labels if they have changed
    /// This function is only used for the legacy Excel files and we will
    /// need a better solution for the new JSON templates
    /// update_labels: if true, automatically update outdated labels. Otherwise, throw an error if a label does not match.
    pub fn update_hpo_duplets(
        &self,
        hpo_duplets: &Vec<HpoTermDuplet>,
    ) -> std::result::Result<Vec<HpoTermDuplet>, String> {
        let mut updated_duplets = vec![];
        for duplet in hpo_duplets {
            let tid = match duplet.to_term_id() {
                Ok(tid) => tid,
                Err(e) => { return Err(format!("Failed to parse TermId from row2: {} (converting duplet: {:?})", duplet.hpo_id(), duplet)); },
            };
            if let Some(term) = self.hpo.term_by_id(&tid) {
                if term.name() != duplet.hpo_label() {
                    // This usually happens if the name of the HPO term was changed after the Excel template
                    // was created. If the user chooses to update labels, this is fixed automatically here.
                    let err_str = format!("{}: expected '{}' but got '{}'", duplet.hpo_id(), term.name(), duplet.hpo_label());
                    updated_duplets.push(HpoTermDuplet::new(term.name(), tid.to_string()));
                    print!("[INFO] Updating HPO label {err_str}"); // Output to shell, this is expected behavior.
                    // consider sending a signal to update user
                } else {
                    updated_duplets.push(HpoTermDuplet::new(term.name(), tid.to_string()));
                }
            } else {
                return Err(format!("No HPO Term found for '{}'", &tid));
            }
        }
        Ok(updated_duplets)
    }

    pub fn check_hpo_duplets(&self, hpo_dup_list: &Vec<HpoTermDuplet>) -> std::result::Result<(), String> {
        for hpo_dup in hpo_dup_list {
            match hpo_dup.to_term_id() {
                Ok(tid) => {
                    match self.hpo.term_by_id(&tid) {
                        Some(term) => {
                            if term.name() != hpo_dup.hpo_label() {
                                return Err(format!("Expected label '{}' but got '{}' for TermId '{}'",
                                                term.name(), hpo_dup.hpo_label(), tid.to_string()));
                            }
                        },
                        None => {
                            return Err( format!("No HPO Term found for '{}'", &tid));
                        },
                    }
                },
                Err(_) => {
                    return Err(format!("Failed to parse TermId: {}", hpo_dup.hpo_id()));
                },
            }
        }
        Ok(())
    }

}
