

use crate::dto::hpo_term_dto::HpoTermData;
use crate::dto::validation_errors::ValidationErrors;
use crate::dto::hpo_term_dto::HpoTermDuplet;
use crate::error::ontology_error::OntologyError;
use ontolius::ontology::csr::FullCsrOntology;
use ontolius::ontology::OntologyTerms;
use ontolius::term::MinimalTerm;
use ontolius::{Identified, TermId};
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
    ) -> std::result::Result<HashMap<TermId, String>, OntologyError> {
        let mut dto_map: HashMap<TermId, String> = HashMap::new();
        for dto in hpo_dto_list {
            match dto.ontolius_term_id() {
                Ok(term_id) => {dto_map.insert(term_id.clone(), dto.label().to_string());},
                Err(_) => {
                    return Err(OntologyError::term_not_found(dto.term_id()));
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


    /// Update the HPO duplets with the current term names from the ontology
    /// This will automatically update term labels if they have changed
    /// This function is only used for the legacy Excel files and we will
    /// need a better solution for the new JSON templates
    /// update_labels: if true, automatically update outdated labels. Otherwise, throw an error if a label does not match.
    pub fn update_hpo_duplets(
        &self,
        hpo_duplets: &Vec<HpoTermDuplet>,
    ) -> std::result::Result<Vec<HpoTermDuplet>, OntologyError> {
        let mut updated_duplets = vec![];
        for duplet in hpo_duplets {
            let tid = match duplet.to_term_id() {
                Ok(tid) => tid,
                Err(_) => { return Err(OntologyError::term_duplet_conversion_error(duplet)) ; }
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
                return Err(OntologyError::term_not_found(tid.to_string()));
            }
        }
        Ok(updated_duplets)
    }


    /// Here we check if any of the term ids in a list of HPO headers is not up to date, meaning that
    /// at least one TermId is not the current primary id or at least one label is not the current label
    /// This can happen if an HPO term is modified or merged after a Cohort is curated. If this method
    /// returns true, we will use the uupdate_hpo_duplets method to revise
    pub fn needs_update(&self, hpo_dup_list: &Vec<HpoTermDuplet>) -> Result<bool, OntologyError> {
        for hpo_dup in hpo_dup_list {
            let tid = hpo_dup.to_term_id()?;
            let term = self.hpo.term_by_id(&tid).ok_or_else(|| OntologyError::term_not_found(hpo_dup.hpo_id()))?;
            if term.name() != hpo_dup.hpo_label() {
                return Ok(true);
            }
            let primary_id = term.identifier();
            if tid != *primary_id {
                return Ok(true);
            }
        }

        Ok(false)
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
