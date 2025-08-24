//! TODO - obsolete this class, we can do everything with ontolius directly now

use crate::dto::hpo_term_dto::HpoTermDto;
use crate::dto::validation_errors::ValidationErrors;
use crate::error::{Error, Result};
use crate::header::hpo_term_duplet::HpoTermDuplet;
use ontolius::ontology::csr::FullCsrOntology;
use ontolius::ontology::OntologyTerms;
use ontolius::term::MinimalTerm;
use ontolius::TermId;
use std::collections::HashMap;
use std::str::FromStr;
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
        hpo_dto_list: &Vec<HpoTermDto>
    ) -> std::result::Result<HashMap<TermId, String>, String> {
        let mut dto_map: HashMap<TermId, String> = HashMap::new();
        for dto in hpo_dto_list {
            let result =  TermId::from_str(dto.term_id());
            match result {
                Ok(term_id) => {dto_map.insert(term_id.clone(), dto.label().clone());},
                Err(_) => {
                    return Err(format!("Could not map termId: '{}'", dto.term_id()));
                },
            } 
        }
        Ok(dto_map)
    }

/* TODO, do we still need this?
    pub fn simple_terms_from_dto(&self, hpo_dto_list: &Vec<HpoTermDto>) -> Result<Vec<SimpleTerm>> {
        let mut simple_terms = vec![];
        for hpo_dto in hpo_dto_list {
            let tid = TermId::from_str(hpo_dto.term_id()).map_err(|e| Error::TermIdError { msg: "Could not map termId".to_string() })?;
            if let Some(term) = self.hpo.term_by_id(&tid) {
                simple_terms.push(term.clone());
            } else {
                return Err(Error::TermError { msg: format!("Could not find term for {}", hpo_dto.term_id()) })
            }
        }
        Ok(simple_terms)
    }*/

    /// Check the validity of the HPO TermId/label pairs in the DTO objects and return corresponding HpoTermDuplet list
    pub fn hpo_duplets_from_dto(&self, hpo_dto_list: &Vec<HpoTermDto>) -> std::result::Result<Vec<HpoTermDuplet>, ValidationErrors> {
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

    /// Check that the HPO Term Id and label used in the DTO object are correct
    pub fn check_hpo_dto(&self, hpo_dto_items: &Vec<HpoTermDto>) -> Result<()> {
        for dto in hpo_dto_items {
            let tid = dto.ontolius_term_id()?;
            let term = self.hpo.term_by_id(&tid).ok_or_else(|| Error::HpoError {
                msg: format!("Term ID not found in ontology: '{}'", dto.term_id()),
            })?;

            if term.name() != dto.label() {
                return Err(Error::HpoError {
                    msg: format!(
                        "Label mismatch for {}: expected '{}', got '{}'",
                        dto.term_id(),
                        term.name(),
                        dto.label()
                    ),
                });
            }
        }
        Ok(())
    }

    /// Update the HPO duplets with the current term names from the ontology
    /// This will automatically update term labels if they have changed
    /// This function is only used for the legacy Excel files and we will
    /// need a better solution for the new JSON templates
    /// update_labels: if true, automatically update outdated labels. Otherwise, throw an error if a label does not match.
    pub fn update_hpo_duplets(
        &self,
        hpo_duplets: &Vec<HpoTermDuplet>,
        update_labels: bool
    ) -> std::result::Result<Vec<HpoTermDuplet>, String> {
        let mut updated_duplets = vec![];
        for duplet in hpo_duplets {
            let tid = match  TermId::from_str(&duplet.row2()) {
                Ok(tid) => tid,
                Err(e) => { return Err(format!("Failed to parse TermId from row2: {}", duplet.row2())); },
            };
            if let Some(term) = self.hpo.term_by_id(&tid) {
                if term.name() != duplet.row1() {
                    // This usually happens if the name of the HPO term was changed after the Excel template
                    // was created. If the user chooses to update labels, this is fixed automatically here.
                    let err_str = format!("{}: expected '{}' but got '{}'", duplet.row2(), term.name(), duplet.row1());
                    if update_labels {
                         updated_duplets.push(HpoTermDuplet::new(term.name(), tid.to_string()));
                         eprint!("Updating HPO label {err_str}"); // Output to shell, this is expected behavior.
                         // consider sending a signal to update user
                    } else {
                        return Err(err_str);
                    }
                }
                updated_duplets.push(HpoTermDuplet::new(term.name(), tid.to_string()));
            } else {
                return Err(format!("No HPO Term found for '{}'", &tid));
            }
        }
        Ok(updated_duplets)
    }

    pub fn check_hpo_duplets(&self, hpo_dup_list: &Vec<HpoTermDuplet>) -> std::result::Result<(), ValidationErrors> {
        let mut verrs = ValidationErrors::new();
        for header_dup in hpo_dup_list {
            let row2 = header_dup.row2();
            let row1 = header_dup.row1();
            match TermId::from_str(&row2) {
                Ok(tid) => {
                    match self.hpo.term_by_id(&tid) {
                        Some(term) => {
                            if term.name() != row1 {
                                verrs.push_str(format!("Expected label '{}' but got '{}' for TermId '{}'",
                                                term.name(), row1, tid.to_string()));
                            }
                        },
                        None => {
                            verrs.push_str( format!("No HPO Term found for '{}'", &tid));
                        },
                    }
                },
                Err(_) => {
                    verrs.push_str(format!("Failed to parse TermId: {}", &row2));
                },
            }
        }
        verrs.ok()
    }

}
