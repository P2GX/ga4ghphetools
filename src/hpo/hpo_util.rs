//! HpoUtil
//! A collection of utility functions for working with HPO

use crate::dto::hpo_term_dto::HpoTermData;
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
            let term_id = dto.ontolius_term_id()?;
            dto_map.insert(term_id.clone(), dto.label().to_string());
        }
        Ok(dto_map)
    }

    pub fn term_label_map_from_duplet_list(
        &self, 
        hpo_duplet_list: &Vec<HpoTermDuplet>
    ) -> std::result::Result<HashMap<TermId, String>, OntologyError> {
        let mut dto_map: HashMap<TermId, String> = HashMap::new();
        for dto in hpo_duplet_list {
            let term_id = dto.to_term_id()?;
            dto_map.insert(term_id.clone(), dto.hpo_label().to_string());
        }
        Ok(dto_map)
    }


    /// Update the HPO duplets with the current term names from the ontology
    /// Useful if a template is using term identifiers/labels that have subsequently been edited 
    /// in the hp.json (as a result of merging/obsoleting terms)
    pub fn update_hpo_duplets(
        &self,
        hpo_duplets: &[HpoTermDuplet],
    ) -> std::result::Result<Vec<HpoTermDuplet>, OntologyError> {
        let mut updated_duplets = vec![];
        for duplet in hpo_duplets {
            let tid =  duplet.to_term_id()?;
            let term = self.hpo.term_by_id(&tid).ok_or_else(||OntologyError::term_not_found(tid.to_string()))?;
            if tid != *term.identifier() {
                println!("[INFO] Updating outdated term id {}->{}", tid, term.identifier());
            }
            if term.name() != duplet.hpo_label() {
                // Output to shell, this is expected behavior.
                println!("[INFO] Updating HPO label {}->{} for {}",
                    duplet.hpo_label(), term.name(), duplet.hpo_id()); 
            }
            updated_duplets.push(HpoTermDuplet::new(term.name(), term.identifier().to_string()));
        }
        Ok(updated_duplets)
    }


    /// Here we check if any of the term ids in a list of HPO headers is not up to date, meaning that
    /// at least one TermId is not the current primary id or at least one label is not the current label
    /// This can happen if an HPO term is modified or merged after a Cohort is curated. If this method
    /// returns true, we will use the uupdate_hpo_duplets method to revise
    pub fn needs_update(&self, hpo_dup_list: &Vec<HpoTermDuplet>) -> Result<bool, OntologyError> {
        for hpo_dup in hpo_dup_list {
            println!("'{}' '{}'", hpo_dup.hpo_label(), hpo_dup.hpo_id());
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

    /// Check a list of HpoTermDuplet objects and return an error at the first case where a Term identifier or label
    /// does not match the corresponding items from the HPO Ontolius object. The indended use case is to catch
    /// ids/labels that are out of date because the corresponding term has been revised (merged, obsoleted) in the HPO
    pub fn check_hpo_duplets(&self, hpo_dup_list: &Vec<HpoTermDuplet>) -> std::result::Result<(), String> {
        for hpo_dup in hpo_dup_list {
            let tid = hpo_dup.to_term_id().map_err(|e|e.to_string())?;
            let term = self.hpo.term_by_id(&tid).ok_or_else(||format!("No HPO Term found for '{}'", &tid))?;
            if tid != *term.identifier() {
                return Err(format!("Expected primary term id '{}' but got '{}' for Term '{}'",
                                term.identifier(), tid.to_string(), term.name()));
            }
            if term.name() != hpo_dup.hpo_label() {
                return Err(format!("Expected label '{}' but got '{}' for TermId '{}'",
                                term.name(), hpo_dup.hpo_label(), tid.to_string()));
            }
        }
        Ok(())
    }

}
