//! TODO - obsolete this class, we can do everything with ontolius directly now

use crate::dto::hpo_term_dto::{self, HpoTermDto};
use crate::error::{self, Error, Result};
use crate::header::header_duplet::{HeaderDuplet, HeaderDupletItem};
use crate::header::hpo_term_duplet::HpoTermDuplet;
use ontolius::io::OntologyLoaderBuilder;
use ontolius::ontology::csr::{FullCsrOntology, MinimalCsrOntology};
use ontolius::ontology::OntologyTerms;
use ontolius::term::simple::SimpleTerm;
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
    ) -> Result<HashMap<TermId, String>> {
        let mut dto_map: HashMap<TermId, String> = HashMap::new();
        for dto in hpo_dto_list {
            let tid = TermId::from_str(&dto.term_id()).map_err(|e| Error::TermIdError { msg: format!("Could not map termId") })?;
            dto_map.insert(tid.clone(), dto.label().clone());
        }
        Ok(dto_map)
    }


    pub fn simple_terms_from_dto(&self, hpo_dto_list: &Vec<HpoTermDto>) -> Result<Vec<SimpleTerm>> {
        let mut simple_terms = vec![];
        for hpo_dto in hpo_dto_list {
            let tid = TermId::from_str(&hpo_dto.term_id()).map_err(|e| Error::TermIdError { msg: format!("Could not map termId") })?;
            if let Some(term) = self.hpo.term_by_id(&tid) {
                simple_terms.push(term.clone());
            } else {
                return Err(Error::TermError { msg: format!("Could not find term for {}", hpo_dto.term_id()) })
            }
        }
        Ok(simple_terms)
    }

    /// Check the validity of the HPO TermId/label pairs in the DTO objects and return corresponding HpoTermDuplet list
    pub fn hpo_duplets_from_dto(&self, hpo_dto_list: &Vec<HpoTermDto>) -> Result<Vec<HpoTermDuplet>> {
        let mut hpo_duplets: Vec<HpoTermDuplet> = Vec::with_capacity(hpo_dto_list.len());
        for hpo_dto in hpo_dto_list {
            let tid = TermId::from_str(&hpo_dto.term_id()).map_err(|e| Error::termid_parse_error(hpo_dto.term_id()))?;
            if let Some(term) = self.hpo.term_by_id(&tid) {
                if term.name() != hpo_dto.label() {
                    return Err(Error::invalid_hpo_label(term.name(), hpo_dto.label(), tid.to_string()));
                }
                hpo_duplets.push(HpoTermDuplet::new(term.name(), tid.to_string()));
            } else {
                return Err(Error::TermError { msg: format!("Could not find term for {}", hpo_dto.term_id()) })
            }
        }

        Ok(hpo_duplets)
    }

    /// Check that the HPO Term Id and label used in the DTO object are correct
    pub fn check_hpo_dto(&self, hpo_dto_items: &Vec<HpoTermDto>) -> Result<()> {
        for dto in hpo_dto_items {
            let tid = TermId::from_str(&dto.term_id())
                .map_err(|_| Error::HpoError {
                    msg: format!("Invalid term ID: '{}'", dto.term_id()),
                })?;
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

    pub fn check_hpo_duplets(&self, header_dup_list: &Vec<HeaderDuplet>) -> Result<()> {
        for header_dup in header_dup_list {
            let row2 = header_dup.row2();
            if ! row2.starts_with("HP:") {
                continue; // skip the constant parts
            }
            let row1 = header_dup.row1();
            match TermId::from_str(&row2) {
                Ok(tid) => {
                    match self.hpo.term_by_id(&tid) {
                        Some(term) => {
                            if term.name() != row1 {
                                return Err(Error::invalid_hpo_label(term.name(), row1, tid.to_string()));
                            }
                        },
                        None => {
                            return Err(Error::term_not_found(&tid));
                        },
                    }
                },
                Err(_) => {
                    return Err(Error::termid_parse_error(&row2));
                },
            }
        }
        Ok(())
    }

}
