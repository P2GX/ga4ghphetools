use std::{collections::{HashMap, HashSet}, sync::Arc};

use ontolius::{ontology::{csr::FullCsrOntology, HierarchyQueries}, TermId};


use crate::dto::{cohort_dto::{CohortData, RowData}, hpo_term_dto::HpoTermDuplet};


/// Locally used struct for convenience
/// These are all of the conflicts that can result from the Ontology structure
#[derive(Debug)]
struct ConflictMap {
    na_terms: HashSet<TermId>,
}

impl ConflictMap {
    
    
    fn conflict_count(&self) -> usize {
            self.na_terms.len()
    }
    
    pub fn no_conflict(&self) -> bool {
        return self.conflict_count() == 0;
    }

    pub fn report(&self) -> String {
        format!("Identified {} ontology (redundancy) conflicts", self.conflict_count())
    }

}


pub struct CohortDataQc {
    /// Reference to the Ontolius Human Phenotype Ontology Full CSR object
    hpo: Arc<FullCsrOntology>,
}


impl CohortDataQc {

    pub fn new(
        hpo: Arc<FullCsrOntology>
    ) -> Self {
        Self { hpo}
    }


    /// Validate the current template.
    /// We check that all of the rows are the correct length
    /// We check whether there are any duplicates in the header
    /// ? What else. Some qc is necessarily done during construction
    ///
    ///  * Returns
    ///
    /// - The first error encountered.
    ///
    pub fn qc_check(
        &self, 
        cohort: &CohortData) -> Result<(), String> {
        let n_hpos = cohort.hpo_headers.len();
        // check correct length
        for row in &cohort.rows {
            if row.hpo_data.len() != n_hpos {
                return Err(format!("Length mismatch: Header: {} vs. row: {}", n_hpos, row.hpo_data.len()))
            }
        }
        // check for duplicates
        let mut seen = HashSet::new();
        for duplet in &cohort.hpo_headers {
            if seen.contains(duplet) {
                return Err(format!("Duplicate entry in HPO Header: {} ({})", duplet.hpo_label(), duplet.hpo_id()));
            } else {
                seen.insert(duplet);
            }
        }
        Ok(())
       
    }

    pub fn qc_conflicting_pairs(&self, cohort: &CohortData) -> Result<(), String> {
        let conflicting_pairs = self.get_conflicting_termid_pairs(cohort)?;
        if conflicting_pairs.no_conflict() {
            return Ok(())
        } else {
            return Err(conflicting_pairs.report());
        }   
    }

    /// This function sets to "na" the values that conflict in any row.
    pub fn sanitize(&self, 
        cohort_dto: &CohortData) 
    -> Result<CohortData, String> {
        self.qc_check(cohort_dto)?;
        let term_id_to_index_map = self.generate_term_id_to_index_map(cohort_dto)?;
        let hpo_terms = &cohort_dto.hpo_headers;
        let mut cohort = cohort_dto.clone();
        for row in cohort.rows.iter_mut() {
            let conflict_map = self.get_conflicting_termid_pairs_for_row(row, hpo_terms)?;
            for tid in conflict_map.na_terms {
                let idx = term_id_to_index_map
                    .get(&tid)
                    .ok_or_else(|| format!("Could not get index for {}", tid.to_string()))?;
                row.hpo_data[*idx] = crate::dto::hpo_term_dto::CellValue::Na;
            }
        }
        Ok(cohort)

    }

    fn generate_term_id_to_index_map(&self, cohort: &CohortData) 
    -> Result<HashMap<TermId, usize>, String> {
        cohort
            .hpo_headers
            .iter()
            .enumerate()
            .map(|(i, duplet)| duplet.to_term_id().map(|tid| (tid, i)))
            .collect()
    }


    fn get_conflicting_termid_pairs(&self, cohort: &CohortData) -> Result<ConflictMap, String> {
        let mut na_terms: HashSet<TermId> = HashSet::new();
        let hpo_terms = &cohort.hpo_headers;
        for row in &cohort.rows {
            let conflict_map = self.get_conflicting_termid_pairs_for_row(row, hpo_terms)?;
            na_terms.extend(conflict_map.na_terms);
        }
        Ok(ConflictMap { 
            na_terms
        })
    }

    fn get_conflicting_termid_pairs_for_row(&self, row: &RowData, hpo_terms: &[HpoTermDuplet]) 
    -> Result<ConflictMap, String> {
        let mut na_terms: HashSet<TermId> = HashSet::new();
       
        let hpo = self.hpo.clone();
        let mut observed: Vec<TermId> = Vec::new();
        let mut excluded: Vec<TermId> = Vec::new();
        for (header, val) in hpo_terms.iter().zip(&row.hpo_data) {
            match val {
                crate::dto::hpo_term_dto::CellValue::Observed => {
                    let tid = header.to_term_id()?;
                    observed.push(tid);
                },
                crate::dto::hpo_term_dto::CellValue::Excluded => {
                    let tid = header.to_term_id()?;
                    excluded.push(tid);
                },
                crate::dto::hpo_term_dto::CellValue::Na => {},
                crate::dto::hpo_term_dto::CellValue::OnsetAge(onset) => {
                    let tid = header.to_term_id()?;
                    observed.push(tid);
                },
                crate::dto::hpo_term_dto::CellValue::Modifier(modifier)  => {
                    let tid = header.to_term_id()?;
                    observed.push(tid);
                },
            }
        }
    
        // If we get here, then all of the observed/excluded terms are in the two lists
        // We can now look for pairs that are redundant
        // basically, a term and its ancestor cannot be in the same phenopacket. We want to retain 
        // the most specific term
        for tid1 in &observed {
            for tid2 in &observed {
                if hpo.is_ancestor_of(tid1, tid2) {
                    // here, tid1 is the ancestor and tid2 is the descendent
                    // we keep only the specific term (descendent)
                    na_terms.insert(tid1.clone());
                }
            }
            for tid2 in &excluded {
                if hpo.is_ancestor_of(tid1, tid2) {
                    // tid1 (observed) is an ancestor of tid2 (excluded)
                    // we assume that tid2 is incorrect because a specific ancestor was annotate
                    na_terms.insert(tid2.clone());
                }
            }
        }
        for tid1 in &excluded {
            for tid2 in &excluded {
                if hpo.is_descendant_of(tid1, tid2) {
                    // tid1 (descendent) is ancestor of tid2 (anscetor) - both excluded
                    // for excluded terms, the more general the term is, the more information it has
                    // therefore, we retain the ancestor
                    na_terms.insert(tid1.clone());
                }
            }
        }
        Ok(ConflictMap {
            na_terms
        })
    }

}