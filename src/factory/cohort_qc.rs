use std::{collections::{HashMap, HashSet}, str::FromStr, sync::Arc};

use ontolius::{Identified, TermId, ontology::{HierarchyQueries, OntologyTerms, csr::FullCsrOntology}, term::MinimalTerm};


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
        self.conflict_count() == 0
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
        self.check_hpo_ids_and_labels(cohort)?;
        self.check_for_duplicate_rows(cohort)?;
        Ok(())
    }


    /// Check the cohort for duplicate entries
    /// We rely on the cross product of PMID and individual id
    fn check_for_duplicate_rows(&self, cohort: &CohortData) -> Result<(), String> {
        let mut seen_entries: HashSet<String> = HashSet::new();
        for row in &cohort.rows {
            let key = format!("{}-{}",row.individual_data.individual_id, row.individual_data.pmid);
            if seen_entries.contains(&key) {
                return Err(format!("Duplicate entry: {}", key));
            } else {
                seen_entries.insert(key);
            }
        }

        Ok(())
    }


    pub fn qc_conflicting_pairs(&self, cohort: &CohortData) -> Result<(), String> {
        let conflicting_pairs = self.get_conflicting_termid_pairs(cohort)?;
        if conflicting_pairs.no_conflict() {
            Ok(())
        } else {
            Err(conflicting_pairs.report())
        }   
    }

    /// Check that the TermId and labels are up to date. Fail on the first error.
    fn check_hpo_ids_and_labels(&self, cohort: &CohortData) -> Result<(), String> {
        for hpo_duplet in &cohort.hpo_headers {
            let hpo_term_id = TermId::from_str(&hpo_duplet.hpo_id)
                .map_err(|e| e.to_string())?;
            let term = self.hpo.term_by_id(&hpo_term_id)
                    .ok_or_else(|| format!("Could not find HPO term for {}", hpo_term_id))?;
            if term.identifier() != &hpo_term_id {
                return Err(format!("{} is not the primary id ({}) for {}",
                    hpo_term_id, term.identifier(), hpo_duplet.hpo_label()));
            }
            if term.name() != hpo_duplet.hpo_label() {
                 return Err(format!("{} is not the current label ({}) for {}",
                    hpo_duplet.hpo_label(), term.name(), hpo_term_id));
            }
        }
        Ok(())
    }


    pub fn check_metadata(&self, cohort: &CohortData) -> Result<(), String> {
        let diseases = &cohort.disease_list;
        if diseases.is_empty() {
            return Err("Disease list empty".to_string());
        }
        for disease in diseases {
            if disease.mode_of_inheritance_list.is_empty() {
                return Err(format!("No mode of inheritance provided for {} ({})", disease.disease_label, disease.disease_id));
            }
        }
        Ok(())
    }


    pub fn sanitize_header(&self, duplets: &Vec<HpoTermDuplet>) -> Result<Vec<HpoTermDuplet>, String> {
        let mut sanitized: Vec<HpoTermDuplet> = Vec::new();
        for duplet in duplets {
            let hpo_term_id = TermId::from_str(duplet.hpo_id()).map_err(|e|e.to_string())?;
            let hpo_term = self.hpo.term_by_id(&hpo_term_id)
                .ok_or_else(|| format!("Could not retrieve term for {}", hpo_term_id))?;
            let sanitized_duplet = HpoTermDuplet::new(hpo_term.name(), hpo_term.identifier().to_string());
            sanitized.push(sanitized_duplet);
        }
        Ok(sanitized)
    }

    /// This function sets to "na" the values that conflict in any row.
    pub fn sanitize(&self, 
        cohort_dto: &CohortData) 
    -> Result<CohortData, String> {
       
        let term_id_to_index_map = self.generate_term_id_to_index_map(cohort_dto)?;
        let hpo_terms = self.sanitize_header(&cohort_dto.hpo_headers)?;
        let mut cohort = cohort_dto.clone();
        cohort.hpo_headers = hpo_terms.clone();
        for row in cohort.rows.iter_mut() {
            let conflict_map = self.get_conflicting_termid_pairs_for_row(row, &hpo_terms)?;
            for tid in conflict_map.na_terms {
                let idx = term_id_to_index_map
                    .get(&tid)
                    .ok_or_else(|| format!("Could not get index for {}", tid))?;
                row.hpo_data[*idx] = crate::dto::hpo_term_dto::CellValue::Na;
            }
        }
        // One more check!
        self.qc_check(&cohort)?;
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
                if hpo.is_ancestor_of(tid2, tid1) {
                    // tid1 (observed) is an ancestor of tid2 (excluded)
                    // we assume that tid2 is incorrect because a specific ancestor was annotate
                    na_terms.insert(tid2.clone());
                }
            }
        }
        for tid1 in &excluded {
            for tid2 in &excluded {
                if hpo.is_descendant_of(tid1, tid2) {
                    // tid1 (descendent) is ancestor of tid2 (ancestor) - both excluded
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


#[cfg(test)]
mod tests {
    use rstest::{fixture, rstest};
    use crate::test_utils::fixtures::hpo;
    use super::*;


    #[fixture]
    fn duplets_with_outdated_hpo_id() -> Vec<HpoTermDuplet> {
        let mut duplets: Vec<HpoTermDuplet> = Vec::new();
        // The correct primary id for Gait Disturbance is  HP:0001288, and
        // HP:0002355 is an alternate_id
        let gait_disturbance = HpoTermDuplet::new("Gait disturbance", "HP:0002355");
        let intoeing = HpoTermDuplet::new("Intoeing", "HP:6001054");
        let dystonia = HpoTermDuplet::new("Dystonia", "HP:0001332");
        duplets.push(gait_disturbance);
        duplets.push(intoeing);
        duplets.push(dystonia);
        duplets
    }


    #[rstest]
    fn test_sanitize_duplets(
        hpo: Arc<FullCsrOntology>,
        duplets_with_outdated_hpo_id: Vec<HpoTermDuplet>
    ) {
        let qc = CohortDataQc::new(hpo.clone());
        let result = qc.sanitize_header(&duplets_with_outdated_hpo_id);
        assert!(result.is_ok());
        let sanitized = result.unwrap();
        assert_eq!(duplets_with_outdated_hpo_id.len(), sanitized.len());
        let duplet_0 = sanitized[0].clone(); // should have updated HPO id
        assert_eq!("HP:0001288", duplet_0.hpo_id());
        assert_eq!("Gait disturbance", duplet_0.hpo_label());
        // the other two terms should be unchanged
        assert_eq!(duplets_with_outdated_hpo_id[1], sanitized[1]);
        assert_eq!(duplets_with_outdated_hpo_id[2], sanitized[2]);
    }


}