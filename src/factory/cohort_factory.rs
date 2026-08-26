//! CohortDtoBuilder
//!
//! The struct that creates and edits the [`CohortDto`] object that we use
//! to store information about the Cohort. It uses the PPKtRow object as an intermediate stage in ETL 
//! for each row of the legacy template to be ingested. This class can be simplified
//! after we are finished refactoring the legacy files.
use std::{collections::{HashMap, HashSet}, str::FromStr, sync::Arc, vec};
use ontolius::{
    Identified, TermId, ontology::{MetadataAware, OntologyTerms, csr::FullCsrOntology}, term::{MinimalTerm, simple::{SimpleMinimalTerm, SimpleTerm}},
};
use crate::{
    dto::{cohort_dto::{CohortData, CohortType, DiseaseData, IndividualData, RowData}, 
    hgvs_variant::HgvsVariant, hpo_term_dto::{CellValue, HpoTermData, HpoTermDuplet}, 
    structural_variant::StructuralVariant}, 
    error::{PheToolsError, cohort_error::CohortError, ontology_error::OntologyError, annotation_error::AnnotationError}, 
    hpo, 
    ppkt::ppkt_row::PpktRow};



/// All data needed to edit a cohort of phenopackets or export as GA4GH Phenopackets
pub struct CohortFactory {
     /// Reference to the Ontolius Human Phenotype Ontology Full CSR object
    hpo: Arc<FullCsrOntology>,
}

impl CohortFactory {

    pub fn new(
        hpo: Arc<FullCsrOntology>
    ) -> Self {
        Self { hpo}
    }

    /// Create the initial phetools template using HPO seed terms
    pub fn create_phetools_template_mendelian(
        // Reference to the Ontolius Human Phenotype Ontology Full CSR object
        hpo: Arc<FullCsrOntology>,
        disease_gene_dto: DiseaseData,
    ) -> std::result::Result<CohortData, String> {
        let hp_header_duplet_list: Vec<HpoTermDuplet> = Vec::new();
         Ok(CohortData::mendelian(disease_gene_dto, hp_header_duplet_list, vec![], hpo.version() ))
    }


    fn get_existing_hpos_from_cohort(
        cohort_dto: &CohortData
    ) -> Result<Vec<TermId>, OntologyError> {
        let mut tid_list: Vec<TermId> = Vec::new();
        for hdd in &cohort_dto.hpo_headers {
            let tid = hdd.to_term_id()?;
            tid_list.push(tid);
        }
        Ok(tid_list)
    }

    #[deprecated]
    pub fn get_updated_header_dto_list(arranged_terms: &Vec<SimpleTerm>) 
    -> Vec<HpoTermDuplet> {
        let mut dto_list: Vec<HpoTermDuplet> = Vec::new();
        for st in arranged_terms {
            let dto = HpoTermDuplet{
                hpo_label: st.name().to_string(),
                hpo_id: st.identifier().to_string()
            };
            dto_list.push(dto);
        }
        dto_list
    }

    pub fn get_previous_hpo_id_list(cohort_dto: &CohortData) -> Result<Vec<TermId>, OntologyError> {
        let mut previous_tid_list: Vec<TermId> = Vec::new();
        for hdd in &cohort_dto.hpo_headers {
            let tid = hdd.to_term_id()?;
            previous_tid_list.push(tid);
        }
        Ok(previous_tid_list)
    }


    /// Check the formatting of the HPO annotations (intended to be used by the add_new_row_to_cohort function)
    fn qc_hp_annotations(&self, hpo_annotations: &Vec<HpoTermData>) -> Result<(), PheToolsError> {
        for hp_annot in hpo_annotations {
            if ! hp_annot.is_observed() {
                if hp_annot.has_modifier() {
                    return Err(AnnotationError::misplaced_modifier(hp_annot).into());
                }
            }
        }

        Ok(())
    }

    /// We have a CohortDto and want to add new data to create a new row.
    /// We need to integrate the HPO annotations contained in hpo_annotations (which has HPO term id, label, and cell value)
    /// with the existing annoations, which potentially means that we need to rearrange the order of the
    /// HPO terms if we add new HPO terms (We keep DFO order). 
    /// We also assume that the front end has already validated the new Variants (that the corresponding objects are contained
    /// in the HashMaps of CohortDto), and that we are getting the corresponding variant keys.
     pub fn add_new_row_to_cohort(
        &mut self,
        individual_data: IndividualData, 
        hpo_annotations: Vec<HpoTermData>,
        variant_key_list: Vec<String>,
        cohort_dto: CohortData) 
    -> Result<CohortData, PheToolsError> {
        // == STEP 0: Q/C
        self.qc_hp_annotations(&hpo_annotations)?;
        // === STEP 1: Extract all HPO TIDs from DTO and classify ===
        let dto_map: HashMap<TermId, String> = hpo::term_label_map_from_dto_list(self.hpo.clone(), &hpo_annotations)?;
        let mut term_id_set_new: HashSet<TermId>  = dto_map.keys().cloned().collect();
        let term_id_list_existing = Self::get_existing_hpos_from_cohort(&cohort_dto)?;
        term_id_set_new.extend(term_id_list_existing); 
         // === STEP 2: Arrange TIDs before borrowing template mutably ===
        let all_tids: Vec<TermId> = term_id_set_new.into_iter().collect();
        //let mut term_arranger = HpoTermArranger::new(self.hpo.clone());
        let arranged_terms = hpo::hpo_terms_to_dfs_order_duplets(self.hpo.clone(), &all_tids)?;
        //let arranged_terms = hpo::hpo_terms_to_dfs_order(hpo, &all_tids).arrange_terms()?;
         // === Step 3: Rearrange the existing PpktRow objects to have the new HPO terms set to "na"
        // 3a. transform the simple terms to HeaderDupletDto objects
        let updated_header_duplet_dto_list = arranged_terms.clone();// Self::get_updated_header_dto_list(&arranged_terms);
        
        // 3b. Update the existing PpktRow objects
        let mut updated_row_dto_list: Vec<RowData> = Vec::new();
        let mut term_id_map: HashMap<TermId, String> = HashMap::new();
        // Make a map and add "na" as the default value for all terms
        for term in &arranged_terms {
            term_id_map.insert(term.to_term_id()?, "na".to_string());
        }
        let previous_hpo_id_list = Self::get_previous_hpo_id_list(&cohort_dto)?;
        for row in cohort_dto.rows {
            // make a copy of the default map and add the actual values for terms for which we have data
            let tid_map = term_id_map.clone();
            match Self::update_row_dto(row, &tid_map, &arranged_terms, &previous_hpo_id_list) {
                Ok(updated_row) => {updated_row_dto_list.push(updated_row);},
                Err(err) => { return Err(PheToolsError::Ontology(err)); },
            }
        }
        // Now add the new RowDto object
        // 1. get map with TermId and Value (e.g., observed) for the new terms
        let mut tid_to_value_map: HashMap<TermId, CellValue> = HashMap::new();
        for dto in hpo_annotations {
            let tid = dto.ontolius_term_id()?;
            tid_to_value_map.insert(tid, dto.entry);
        }

        if cohort_dto.disease_list.is_empty() {
            return Err(PheToolsError::Cohort(CohortError::empty_disease_list()));
        }

        let novel_row = Self::new_row_dto(
            &updated_header_duplet_dto_list, 
            individual_data, 
            variant_key_list, 
            tid_to_value_map, 
            &cohort_dto.disease_list)?;
            
        updated_row_dto_list.push(novel_row);
        
        let updated_cohort_dto = CohortData{
            cohort_type: cohort_dto.cohort_type,
            disease_list: cohort_dto.disease_list,
            hpo_headers: updated_header_duplet_dto_list,
            rows: updated_row_dto_list,
            hgvs_variants: cohort_dto.hgvs_variants,
            structural_variants: cohort_dto.structural_variants,
            intergenic_variants: cohort_dto.intergenic_variants,
            phetools_schema_version: cohort_dto.phetools_schema_version,
            hpo_version: self.hpo.version().to_string(),
            cohort_acronym: cohort_dto.cohort_acronym,
            curation_history: cohort_dto.curation_history
        };
        Ok(updated_cohort_dto)
        
       
    }

    fn update_row_dto(
        row: RowData, 
        tid_to_value_map: &HashMap<TermId, String>,
        updated_header: &Vec<HpoTermDuplet>,
        previous_hpo_id_list: &[TermId]
    ) -> Result<RowData, OntologyError> {
        let hpo_cell_content_list = row.hpo_data.clone();
        let updated_tid_list = updated_header
            .iter()
            .map(|st| st.to_term_id())
            .collect::<Result<Vec<_>, _>>()?;
        let reordering_indices = Self::get_update_vector(&previous_hpo_id_list, &updated_tid_list);

        let updated_hpo = Self::reorder_or_fill_na(&hpo_cell_content_list, 
        &reordering_indices,
        updated_header.len());
        Ok(RowData {
            individual_data: row.individual_data,
            disease_id_list: row.disease_id_list,
            allele_count_map: row.allele_count_map,
            hpo_data: updated_hpo,
        })
    }


    /// Create a new RowDto. This is used when we create a row (phenopacket) with terms that
    /// may not be included in the previous phenopackets and which may not have values for all of the
    /// terms in the previous phenopackets. 
    /// Note that we assume the variants have been previously validated; we get the corresponding variant_keys as a list,
    /// one for each allele found in the individual (thus, we may get two identical alleles for homozygosity).
    ///  # Arguments
    ///
    /// * `header` - Header with all HPO terms in previous cohort and new phenopacket, ordered by DFS
    /// * `individual_dto` - DTO with demographic information about the new individual
    /// * `variant_key_list` - List of variant keys (one per allele) for this individual
    /// * `tid_to_value_map` - this has values (e.g., observed, na, P32Y2M) for which we have information in the new phenopacket
    /// * `dg_data`- list of diseases diagnosed in this cohort/individual
    fn new_row_dto(
        header_dto_list:  &Vec<HpoTermDuplet>, 
        individual_dto: IndividualData,
        variant_key_list: Vec<String>,
        tid_to_value_map: HashMap<TermId, CellValue>, 
        disease_data_list: &Vec<DiseaseData>
    ) -> std::result::Result<RowData, PheToolsError> {
        // Create a list of CellDto objects that matches the new order of HPO headers
        let mut hpo_cell_list: Vec<CellValue> = Vec::with_capacity(header_dto_list.len());
        for hduplet in header_dto_list {
            let tid = hduplet.to_term_id()?;
            let cell_value: CellValue = match tid_to_value_map.get(&tid) {
                Some(cv) => cv.clone(),
                None => CellValue::na()
            };
            hpo_cell_list.push(cell_value);
        }
        let disease_id_list: Vec<String> = disease_data_list.iter().map(|d| d.disease_id.clone()).collect();
        // Could the alleles
        let mut allele_count_map: HashMap<String, usize> = HashMap::new();
        for allele in variant_key_list {
            *allele_count_map.entry(allele).or_insert(0) += 1;
        }
       let novel_row_dto = RowData{
            individual_data: individual_dto,
            disease_id_list: disease_id_list,
            allele_count_map,
            hpo_data: hpo_cell_list,
        };
        Ok(novel_row_dto)
    }

    /// Given a previous list of `TermId`s and an updated list, this function
    /// returns a vector of indices representing where each element of the
    /// `previous_hpo_list` now appears in the `updated_hpo_list`.
    ///
    /// This is useful for tracking how terms from an earlier template are
    /// rearranged after updating the template (e.g., after inserting or reordering terms).
    /// The returned vector can be used to remap associated data (e.g., column values)
    /// to their new positions.
    ///
    /// # Arguments
    /// - `previous_hpo_list`: The list of HPO term IDs before the update.
    /// - `updated_hpo_list`: The reordered or expanded list of HPO term IDs after the update.
    ///                       It must contain all terms from `previous_hpo_list`.
    ///
    /// # Returns
    /// A `Vec<usize>` where each element `i` gives the index in `updated_hpo_list`
    /// of the `i`-th term in `previous_hpo_list`.
    ///
    /// # Panics
    /// This function will panic if any term from `previous_hpo_list` is not found in `updated_hpo_list`.
    ///
    pub fn get_update_vector(
        previous_hpo_list: &[TermId],
        updated_hpo_list: &[TermId])
    -> Vec<usize> {
        let id_to_new_index: HashMap<TermId, usize> = updated_hpo_list
            .iter()
            .enumerate()
            .map(|(i, tid)| (tid.clone(), i))
            .collect();
        let new_indices: Vec<usize> = previous_hpo_list
            .iter()
            .map(|tid| id_to_new_index[tid])
            .collect();
        new_indices
    }

    /// Given the old values and a mapping from old indices to new indices,
    /// return a new vector of the size of the updated list, where each element
    /// from the original list is moved to its new index, and all other positions
    /// are filled with `"na"`.
    ///
    /// # Arguments
    /// - `old_values`: The values associated with the old HPO list (same order).
    /// - `old_to_new_indices`: A vector where `old_to_new_indices[i]` gives the
    ///                         index in the new list where the `i`th old value should go.
    /// - `new_size`: The size of the new list (typically, `updated_hpo_list.len()`).
    ///
    /// # Returns
    /// A `Vec<String>` of length `new_size` where old values are in their new positions,
    /// and new (missing) entries are `"na"`.
    fn reorder_or_fill_na(
        old_values: &[CellValue],
        old_to_new_indices: &[usize],
        new_size: usize,
    ) -> Vec<CellValue> {
        let mut new_values = vec![CellValue::na(); new_size];
        for (old_idx, &new_idx) in old_to_new_indices.iter().enumerate() {
            new_values[new_idx] = old_values[old_idx].clone();
        }
        new_values
    }



    pub fn create_pyphetools_template(
        template_type: CohortType,
        disease_data: DiseaseData,
        hpo: Arc<FullCsrOntology>,
    ) -> std::result::Result<CohortData, String> {
        let smt_list: Vec<SimpleMinimalTerm> = Vec::new();
        if template_type == CohortType::Mendelian {
            let cohort_dto = Self::create_phetools_template_mendelian(hpo, disease_data)?;
            Ok(cohort_dto)
        } else {
            Err(format!("Creation of template of type {:?} not supported", template_type))
        } 
    }

   
    /// We are extract a DiseaseGeneData from the Excel files (version 1), all of which are
    /// Mendelian. We know the columns are
    /// (0) "PMID", (1) "title", (2) "individual_id", (3)"comment", 
    /// (4*) "disease_id", (5*) "disease_label", (6*) "HGNC_id", (7*) "gene_symbol", 
    ///  (8*)  "transcript", (9) "allele_1", (10) "allele_2", (11) "variant.comment", 
    ///    (12) "age_of_onset", (13)"age_at_last_encounter", (14)  "deceased", (15) "sex", (16) "HPO", 
    /// The columns with asterisk are what we need
    /*
    pub fn get_disease_dto_from_excel(matrix: &Vec<Vec<String>>) -> std::result::Result<DiseaseData, String> {
        let rows: Vec<&Vec<String>> = matrix.iter().skip(2).collect();
        if rows.is_empty() {
            return Err("Could not extract DTO because less than three rows found".to_string());
        };
        let mut extracted_data: Vec<(String, String, String, String, String)> = Vec::new();
        for (row_idx, row) in rows.iter().enumerate() {
            if row.len() <= 16 {
                return Err(format!("Row {} (after skipping 2) has only {} columns, need at least 16", 
                                row_idx, row.len()));
            }
            extracted_data.push((row[4].clone(), row[5].clone(), row[6].clone(), row[7].clone(), row[8].clone()));
        }
        let first = &extracted_data[0];
        let all_identical = extracted_data.iter().all(|tuple| tuple == first);
        if ! all_identical {
            return Err("DiseaseGeneData-related columns are not equal in all rows - requires manual check".to_string());
        }
       
        let gtr_data = GeneTranscriptData{
            hgnc_id: first.2.clone(),
            gene_symbol: first.3.clone(),
            transcript: first.4.clone(),
        };
         let disease_data = DiseaseData{
            disease_id: first.0.clone(),
            disease_label: first.1.clone(),
            mode_of_inheritance_list: vec![],
            gene_transcript_list: vec![gtr_data],
        };       
        Ok(disease_data)
    }*/


    fn check_duplet(&self, duplet: &HpoTermDuplet) -> std::result::Result<(), OntologyError> {
        let term_id = TermId::from_str(duplet.hpo_id())
            .map_err(|_|OntologyError::term_id_creation(duplet.hpo_id()))?;
        let term: &SimpleTerm = self.hpo.term_by_id(&term_id)
            .ok_or_else(|| OntologyError::term_not_found(term_id.to_string()))?;
        if term.identifier().to_string() != duplet.hpo_id() {
            return Err(OntologyError::tid_mismatch(term.identifier().to_string(), duplet.hpo_id()));
        } else if term.name() != duplet.hpo_label() {
                return Err(OntologyError::label_mismatch(duplet.hpo_label(),  term.name()));
        }
        Ok(())
    }

    

    fn get_hgvs_variants(
        &self,
        row: &PpktRow,
        hgvs_dict: &HashMap<String, HgvsVariant>) 
    -> std::result::Result<Vec<HgvsVariant>, String> {
        let mut var_list: Vec<HgvsVariant> = Vec::new();
        for dto in row.get_gene_var_dto_list() {
            if dto.allele1_is_hgvs() {
                let key = dto.get_key_allele1();
                let variant = hgvs_dict
                    .get(&key)
                    .ok_or_else(|| format!("No variant found for allele1: '{}'", key))?;
                var_list.push(variant.clone());
            }
            if dto.allele2_is_hgvs() {
                let key = dto.get_key_allele2();
                let variant = hgvs_dict
                    .get(&key)
                    .ok_or_else(|| format!("No variant found for allele2: '{}'", key))?;
                var_list.push(variant.clone());
            }
        }
        Ok(var_list)
    }

    fn get_structural_variants(
        &self,
        row: &PpktRow,
        sv_dict: &HashMap<String, StructuralVariant>) 
    -> std::result::Result<Vec<StructuralVariant>, String> {
        let mut var_list: Vec<StructuralVariant> = Vec::new();
        for dto in row.get_gene_var_dto_list() {
            if ! dto.allele1_is_sv() {
                let key = dto.get_key_allele1();
                let variant = sv_dict
                    .get(&key)
                    .ok_or_else(|| format!("No structural variant found for allele1: '{}'", key))?;
                var_list.push(variant.clone());
            }
            if dto.allele2_is_sv() {
                let key = dto.get_key_allele2();
                let variant = sv_dict
                    .get(&key)
                    .ok_or_else(|| format!("No structural variant found for allele2: '{}'", key))?;
                var_list.push(variant.clone());
            }
        }
        Ok(var_list)
    }


    /// Compare the old header with the new header and update the RowData object to have the new HPO columns but set the value to na for these columns because we do not yet have the value for them
    fn update_hpo_row_with_new_term(
        &self,
        oldrow: &RowData, 
        previous_duplets: &Vec<HpoTermDuplet>,
        update_duplets: &Vec<HpoTermDuplet>,
        term_id_to_na_map: &HashMap<TermId, CellValue>) 
    -> Result<RowData, OntologyError> {
        if oldrow.hpo_data.len() != previous_duplets.len() {
            return Err(OntologyError::header_length_mismatch_err(oldrow.hpo_data.len(), previous_duplets.len()));
        }
        let mut term_id_map = term_id_to_na_map.clone();
        for (duplet, value) in previous_duplets.iter().zip(oldrow.hpo_data.clone()) {
            let tid = duplet.to_term_id()?;
            if ! term_id_map.contains_key(&tid) {
                return Err(OntologyError::missing_tid(tid.to_string(), "term_id_to_na_map"));
            }
            term_id_map.insert(tid, value);
        }
        let mut content: Vec<CellValue> = Vec::new();
        for duplet in update_duplets {
            match term_id_map.get(&duplet.to_term_id()?) {
                Some(cv) => {
                    content.push(cv.clone());
                },
                None => {
                    // should never happen, if it does, there is a problem with the arguments to the function
                    return Err(OntologyError::missing_tid(duplet.hpo_id(), "HPO row update"));
                }
            }
        }
        let mut newrow = oldrow.clone();
        newrow.hpo_data = content;
        Ok(newrow)
    }
     
    pub fn add_hpo_term_to_cohort(
        &mut self,
        hpo_id: &str,
        hpo_label: &str,
        cohort: CohortData
    ) 
    -> std::result::Result<CohortData, OntologyError> {
        let new_tid = TermId::from_str(hpo_id)
            .map_err(|_| OntologyError::term_id_creation(hpo_id))?;
        let term = self.hpo
            .term_by_id(&new_tid)
            .ok_or_else(|| OntologyError::term_not_found(hpo_id))?;
        // === STEP 1: Add new HPO term to existing terms and arrange TIDs ===
        let all_tid_result: Result<Vec<TermId>, OntologyError> =
            cohort.hpo_headers
                .iter()
                .map(|duplet| duplet.to_term_id())
                .collect();
        let mut all_tids = all_tid_result?;
        if all_tids.contains(&new_tid) {
            return Err(OntologyError::redundant_tid(new_tid.to_string()));
        }
        all_tids.push(new_tid.clone());
        let arranged_hpo_duplets = hpo::hpo_terms_to_dfs_order_duplets(self.hpo.clone(), &all_tids)?;
        // === Step 3: Rearrange the existing RowData objects to have the new HPO terms and set the new terms to "na"
        // This will be modified so that the new rows have the old value for the old terms and na for the new terms.
        let mut term_id_to_na_map: HashMap<TermId, CellValue> = HashMap::new(); 
        for duplet in &arranged_hpo_duplets {
            term_id_to_na_map.insert(duplet.to_term_id()?.clone(), CellValue::na());
        }
        // strategy: Make a HashMap with all of the new terms, initialize the values to na. Update the map with the current values. The remaining (new) terms will be "na". 
        let mut updated_cohort = cohort.clone();
        updated_cohort.hpo_headers = arranged_hpo_duplets;
        let mut updated_ppkt_rows: Vec<RowData> = Vec::new();
        for oldrow in cohort.rows {
            let newrow = self.update_hpo_row_with_new_term(&oldrow, &cohort.hpo_headers, &updated_cohort.hpo_headers, &term_id_to_na_map)?;
            updated_ppkt_rows.push(newrow);
        }
        updated_cohort.rows = updated_ppkt_rows;
        Ok(updated_cohort)
    }


    /// Basic sanity check that we are not merging the wrong cohort, but this should actually never happen.
    pub fn disease_data_identity_validation(previous: &CohortData, transformed: &CohortData) -> Result<(), String>{
        if previous.disease_list.len() != transformed.disease_list.len() {
            return Err(format!("Disease list length mismatch: previous {}, new {}",
                previous.disease_list.len(), transformed.disease_list.len()));
        }
        for (prev, transf) in previous.disease_list.iter().zip(transformed.disease_list.iter()) {
            if prev.disease_id != transf.disease_id {
                return Err(format!("Previous disease ID '{:?}'; new disease ID: '{:?}'", prev.disease_id, transf.disease_id));
            }
            if prev.disease_label != transf.disease_label {
                return Err(format!("Previous disease label '{:?}'; new disease label: '{:?}'", prev.disease_label, transf.disease_label));
            }
            // note we do not care if the MOI list matches, this can be adjusted as needed
            if prev.gene_transcript_list.len() != transf.gene_transcript_list.len() {
                return Err(format!("Previous disease genes '{:?}'; new disease genes: '{:?}'", prev.gene_transcript_list.len(), transf.gene_transcript_list.len()));
            }
        }
        Ok(())
    }

    /// Get the combined HPO TermId list (filter out duplicates) from both cohorts
    fn get_combined_tids(previous: &CohortData, transformed: &CohortData) -> Result<Vec<TermId>, OntologyError> {
        let new_tids: Vec<TermId> = transformed
            .hpo_headers
            .iter()
            .map(|duplet| duplet.to_term_id())
            .collect::<Result<Vec<_>, _>>()?;
        let prev_tids: Vec<TermId> = previous
            .hpo_headers
            .iter()
            .map(|duplet| duplet.to_term_id())
            .collect::<Result<Vec<_>, _>>()?;
        let mut seen = HashSet::new();
        let combined: Vec<TermId> = new_tids
            .into_iter()
            .chain(prev_tids.into_iter())
            .filter(|tid| seen.insert(tid.clone())) // only keep if not seen before
            .collect();
        Ok(combined)
    }

    /// With this function, we are added data from a new cohort (transformed from an ETL) to an existing cohort
    /// We need to alter the HPO headers to include terms from both cohorts
    /// We need to add an "NA" for columns where the previous row does not have data
    pub fn merge_cohort_data(self, previous: CohortData, transformed: CohortData) -> Result<CohortData, PheToolsError>{
        let all_tids: Vec<TermId> = Self::get_combined_tids(&previous, &transformed)?;
        let arranged_hpo_duplets = hpo::hpo_terms_to_dfs_order_duplets(self.hpo.clone(), &all_tids)?;
        // === Step 3: Rearrange the existing RowData objects to have the new HPO terms and set the new terms to "na"
        // This will be modified so that the new rows have the old value for the old terms and na for the new terms.
        let mut term_id_to_na_map: HashMap<TermId, CellValue> = HashMap::new(); 
        for duplet in &arranged_hpo_duplets {
            term_id_to_na_map.insert(duplet.to_term_id()?.clone(), CellValue::na());
        }
        // strategy: Make a HashMap with all of the new terms, initialize the values to na. Update the map with the current values. The remaining (new) terms will be "na". 
        let mut updated_cohort = previous.clone();
        updated_cohort.hpo_headers = arranged_hpo_duplets;
        let mut updated_ppkt_rows: Vec<RowData> = Vec::new();
        for oldrow in &previous.rows {
            let newrow = self.update_hpo_row_with_new_term(oldrow, &previous.hpo_headers, &updated_cohort.hpo_headers, &term_id_to_na_map)?;
            updated_ppkt_rows.push(newrow);
        }
        // Now the same for the transformed rows!
        for tr_row in &transformed.rows {
            let newrow = self.update_hpo_row_with_new_term(tr_row, &transformed.hpo_headers, &updated_cohort.hpo_headers, &term_id_to_na_map)?;
            updated_ppkt_rows.push(newrow);
        }
        updated_cohort.rows = updated_ppkt_rows;

        // Merge HashMaps - entries from 'transformed' will overwrite conflicting keys in 'self'
        // but they should be the same from the way we construct the map
        updated_cohort.hgvs_variants.extend(transformed.hgvs_variants);
        updated_cohort.structural_variants.extend(transformed.structural_variants); 
        updated_cohort.intergenic_variants.extend(transformed.intergenic_variants);

        Ok(updated_cohort)
    }
}


#[cfg(test)]
mod test {
    use crate::{dto::cohort_dto::{DiseaseData, GeneTranscriptData}};
    use crate::test_utils::fixtures::hpo;
    use super::*;
    use rstest::{fixture, rstest};


  


    #[fixture]
    fn disease_gene_dto() -> DiseaseData {
       
        let gv_dto = GeneTranscriptData{ 
            hgnc_id: "HGNC:171".to_string(), 
            gene_symbol: "ACVR1".to_string(), 
            transcript:   "NM_001111067.4".to_string(),
        };
         let dx_dto = DiseaseData{
            disease_id:"OMIM:135100".to_string(),
            disease_label:"Fibrodysplasia ossificans progressiva".to_string(),
            mode_of_inheritance_list:vec![], 
            gene_transcript_list: vec![gv_dto] 
        };
        dx_dto
    }

    // Define a basic rstest fixture for standard HpoTermData
    #[fixture]
    fn high_palate_observed_with_modifier() -> HpoTermData {
        // Construct according to your struct definition
        HpoTermData {
            term_duplet: HpoTermDuplet {
                hpo_label: "High palate".to_string(),
                hpo_id: "HP:0000218".to_string(),
            },
            entry: CellValue {
                entry: crate::dto::hpo_term_dto::CellValueInner::Observed, 
                modifiers: vec!["HP:0012828".to_string()],
            },
        }
    }

    #[rstest]
    fn test_qc_hp_annotations_with_invalid_modifier(
        high_palate_observed_with_modifier: HpoTermData,
        hpo: Arc<FullCsrOntology>
    ) {
        let annotations = vec![high_palate_observed_with_modifier];
        let factory = CohortFactory::new(hpo.clone());
        let result = factory.qc_hp_annotations(&annotations);
        assert!(result.is_ok());
    }



   
 

}
