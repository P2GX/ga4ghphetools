//! CohortDtoBuilder
//!
//! The struct that creates and edits the [`CohortDto`] object that we use
//! to store information about the Cohort. It uses the PPKtRow object as an intermediate stage in ETL 
//! for each row of the legacy template to be ingested. This class can be simplified
//! after we are finished refactoring the legacy files.
use std::{collections::{HashMap, HashSet}, str::FromStr, sync::Arc, vec};
use ontolius::{
    ontology::{csr::FullCsrOntology, OntologyTerms},
    term::{simple::{SimpleMinimalTerm, SimpleTerm}, MinimalTerm},
    Identified, TermId,
};
use phenopackets::schema::v2::Phenopacket;

use crate::{dto::{cohort_dto::{CohortData, CohortType, DiseaseData, DiseaseGeneData, GeneTranscriptData, IndividualData, RowData}, hgvs_variant::HgvsVariant, hpo_term_dto::CellValue, hpo_term_dto::{HpoTermData, HpoTermDuplet}, structural_variant::{StructuralVariant, SvType}}, hpo::hpo_util::HpoUtil, ppkt::{ppkt_exporter::PpktExporter, ppkt_row::PpktRow}, template::header_duplet_row::HeaderDupletRow, variant::variant_manager::VariantManager};
use crate::{
    hpo::hpo_term_arranger::HpoTermArranger
};


/// All data needed to edit a cohort of phenopackets or export as GA4GH Phenopackets
pub struct CohortDtoBuilder {
    cohort_type: CohortType,
    /// Data structure used to seed new entries in the template (info re: gene[s], disease[s])
    disease_gene_dto: DiseaseGeneData,
     /// Reference to the Ontolius Human Phenotype Ontology Full CSR object
    hpo: Arc<FullCsrOntology>,
}

impl CohortDtoBuilder {

    pub fn new(
        cohort_type: CohortType,
        disease_gene_dto: DiseaseGeneData,
        hpo: Arc<FullCsrOntology>
    ) -> Self {
        Self { cohort_type, disease_gene_dto, hpo}
    }



    /// Create the initial pyphetools template using HPO seed terms
    pub fn create_pyphetools_template_mendelian(
        hpo_term_ids: Vec<TermId>,
        // Reference to the Ontolius Human Phenotype Ontology Full CSR object
        hpo: Arc<FullCsrOntology>,
        disease_gene_dto: DiseaseGeneData,
    ) -> std::result::Result<CohortData, String> {
        let mut hp_header_duplet_list: Vec<HpoTermDuplet> = Vec::new();
        for hpo_id in hpo_term_ids {
            match hpo.term_by_id(&hpo_id) {
                Some(term) => {
                    let hpo_duplet = HpoTermDuplet::new(term.name(), term.identifier().to_string());
                    hp_header_duplet_list.push(hpo_duplet);
                }
                None => {
                    return Err(format!("Could not find HPO identifier '{}'", hpo_id.to_string()));
                }
            }
        }
         Ok(CohortData::mendelian(disease_gene_dto, hp_header_duplet_list, vec![] ))
    }


    fn get_existing_hpos_from_cohort(
        cohort_dto: &CohortData
    ) -> Result<Vec<TermId>, String> {
        let mut tid_list: Vec<TermId> = Vec::new();
        for hdd in &cohort_dto.hpo_headers {
            match hdd.to_term_id() {
                Ok(tid) => tid_list.push(tid),
                Err(e) => { return Err(format!("Could not extract TermIf from {:?}", hdd)); }
            }
        }
        Ok(tid_list)
    }


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

    pub fn get_previous_hpo_id_list(cohort_dto: &CohortData) -> Result<Vec<TermId>, String> {
        let mut previous_tid_list: Vec<TermId> = Vec::new();
        for hdd in &cohort_dto.hpo_headers {
            match hdd.to_term_id() {
                Ok(tid) => previous_tid_list.push(tid),
                Err(_) => { return Err(format!("Could not extract TermId from {:?}", hdd));},
            }
        }
        Ok(previous_tid_list)
    }

    /// We have a CohortDto and want to add new data to create a new row.
    /// We need to integrate the HPO annotations contained in hpo_annotations (which has HPO term id, label, and cell value)
    /// with the existing annoations, which potentially means that we need to rearrange the order of the
    /// HPO terms if we add new HPO terms (We keep DFO order). 
    /// We also assume that the front end has already validated the new Variants (that the corresponding objects are contained
    /// in the HashMaps of CohortDto), and that we are getting the corresponding variant keys.
     pub fn add_new_row_to_cohort(
        &mut self,
        individual_dto: IndividualData, 
        hpo_annotations: Vec<HpoTermData>,
        variant_key_list: Vec<String>,
        cohort_dto: CohortData) 
    -> Result<CohortData, String> {
        let hpo_util = HpoUtil::new(self.hpo.clone());
        // === STEP 1: Extract all HPO TIDs from DTO and classify ===
        let dto_map: HashMap<TermId, String> = hpo_util.term_label_map_from_dto_list(&hpo_annotations)?;
        let mut term_id_set_new: HashSet<TermId>  = dto_map.keys().cloned().collect();
        let term_id_list_existing = Self::get_existing_hpos_from_cohort(&cohort_dto)?;
        term_id_set_new.extend(term_id_list_existing); 
         // === STEP 2: Arrange TIDs before borrowing template mutably ===
        let all_tids: Vec<TermId> = term_id_set_new.into_iter().collect();
        let mut term_arranger = HpoTermArranger::new(self.hpo.clone());
        let arranged_terms = term_arranger.arrange_terms(&all_tids)?;
         // === Step 3: Rearrange the existing PpktRow objects to have the new HPO terms set to "na"
        // 3a. transform the simple terms to HeaderDupletDto objects
        let updated_header_duplet_dto_list = Self::get_updated_header_dto_list(&arranged_terms);
        
        // 3b. Update the existing PpktRow objects
        let mut updated_row_dto_list: Vec<RowData> = Vec::new();
        let mut term_id_map: HashMap<TermId, String> = HashMap::new();
        // Make a map and add "na" as the default value for all terms
        for term in &arranged_terms {
            term_id_map.insert(term.identifier().clone(), "na".to_string());
        }
        let previous_hpo_id_list = Self::get_previous_hpo_id_list(&cohort_dto)?;
        for row in cohort_dto.rows {
            // make a copy of the default map and add the actual values for terms for which we have data
            let tid_map = term_id_map.clone();
            match Self::update_row_dto(row, &tid_map, &arranged_terms, &previous_hpo_id_list) {
                Ok(updated_row) => {updated_row_dto_list.push(updated_row);},
                Err(err) => { return Err(err); },
            }
        }
        // Now add the new RowDto object
        // 1. get map with TermId and Value (e.g., observed) for the new terms
        let mut tid_to_value_map: HashMap<TermId, String> = HashMap::new();
        for dto in   hpo_annotations {
            match dto.ontolius_term_id() {
                Ok(tid) => { tid_to_value_map.insert(tid, dto.entry().to_string()); },
                Err(_) => { return Err(format!("Could not create TermId from {:?}", &dto)); },
            }
        }
      
        let novel_row = Self::new_row_dto(
            &updated_header_duplet_dto_list, 
            individual_dto, 
            variant_key_list, 
            tid_to_value_map, 
            cohort_dto.disease_gene_dto.clone())?;
            
        updated_row_dto_list.push(novel_row);
        
        let updated_cohort_dto = CohortData{
            cohort_type: cohort_dto.cohort_type,
            disease_gene_dto: cohort_dto.disease_gene_dto,
            hpo_headers: updated_header_duplet_dto_list,
            rows: updated_row_dto_list,
            hgvs_variants: cohort_dto.hgvs_variants,
            structural_variants: cohort_dto.structural_variants,
            dto_version: cohort_dto.dto_version,
            cohort_acronym: cohort_dto.cohort_acronym
        };
        Ok(updated_cohort_dto)
        
       
    }

    fn update_row_dto(
        row: RowData, 
        tid_to_value_map: &HashMap<TermId, String>,
        updated_header: &Vec<SimpleTerm>,
        previous_hpo_id_list: &[TermId]
    ) -> Result<RowData, String> {
         // update the tid map with the existing  values
       // let previous_hpo_id_list = row.
       // let previous_hpo_id_list = updated_header.get_hpo_id_list()?;
        let hpo_cell_content_list = row.hpo_data.clone();
        
    /*     if previous_hpo_id_list.len() != hpo_cell_content_list.len() {
            return Err(format!("Mismatched lengths between HPO ID list from header ({}) and HPO content from row ({})",
                previous_hpo_id_list.len(), hpo_cell_content_list.len())); 
        }*/
        //let updated_hpo_id_list = updated_header.get_hpo_id_list()?;
        let updated_tid_list: Vec<TermId> = updated_header.iter().map(|st| st.identifier().clone()).collect();
        let reordering_indices = Self::get_update_vector(&previous_hpo_id_list, &updated_tid_list);

        let updated_hpo = Self::reorder_or_fill_na(&hpo_cell_content_list, 
        &reordering_indices,
        updated_header.len());
        Ok(RowData {
            individual_dto: row.individual_dto,
            disease_dto_list: row.disease_dto_list,
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
    /// * `cohort_dto`- DTO for the entire previous cohort (TODO probably we need a better DTO with the new DiseaseBundle!)
    fn new_row_dto(
        header_dto_list:  &Vec<HpoTermDuplet>, 
        individual_dto: IndividualData,
        variant_key_list: Vec<String>,
        tid_to_value_map: HashMap<TermId, String>, 
        disease_gene_dto: DiseaseGeneData
    ) -> std::result::Result<RowData, String> {
        // Create a list of CellDto objects that matches the new order of HPO headers
        let mut hpo_cell_list: Vec<CellValue> = Vec::with_capacity(header_dto_list.len());
        for hduplet in header_dto_list {
            let tid = hduplet.to_term_id()?;
            let value: String =  tid_to_value_map.get(&tid).map_or("na", |v| v).to_string();
            let cell_value = CellValue::from_str(&value)?;
            hpo_cell_list.push(cell_value);
        }
        if disease_gene_dto.gene_transcript_dto_list.len() != 1 {
            return Err(format!("Only implemented for Mendelian but gene transcript length was {}", disease_gene_dto.gene_transcript_dto_list.len()));
        }
        // Could the alleles
        let mut allele_count_map: HashMap<String, usize> = HashMap::new();
        for allele in variant_key_list {
            *allele_count_map.entry(allele).or_insert(0) += 1;
        }
       let novel_row_dto = RowData{
            individual_dto,
            disease_dto_list: disease_gene_dto.disease_dto_list.clone(),
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
        let mut new_values = vec![CellValue::Na; new_size];
        for (old_idx, &new_idx) in old_to_new_indices.iter().enumerate() {
            new_values[new_idx] = old_values[old_idx].clone();
        }
        new_values
    }



    pub fn create_pyphetools_template(
        template_type: CohortType,
        disease_gene_dto: DiseaseGeneData,
        hpo_term_ids: Vec<TermId>,
        hpo: Arc<FullCsrOntology>,
    ) -> std::result::Result<CohortData, String> {
        let mut smt_list: Vec<SimpleMinimalTerm> = Vec::new();
        for hpo_id in &hpo_term_ids {
            match hpo.term_by_id(hpo_id) {
                Some(term) => {
                    let smt =
                        SimpleMinimalTerm::new(term.identifier().clone(), term.name(), vec![], false);
                    smt_list.push(smt);
                }
                None => {
                    return Err(format!("Could not find HPO identifier '{}'", hpo_id.to_string()));
                }
            }
        }
        if template_type == CohortType::Mendelian {
            let cohort_dto = Self::create_pyphetools_template_mendelian( hpo_term_ids, hpo, disease_gene_dto)?;
            Ok(cohort_dto)
        } else {
            Err(format!("Creation of template of type {:?} not supported", template_type))
        } 
    }

   
    /// We are extract a DiseaseGeneDto from the Excel files (version 1), all of which are
    /// Mendelian. We know the columns are
    /// (0) "PMID", (1) "title", (2) "individual_id", (3)"comment", 
    /// (4*) "disease_id", (5*) "disease_label", (6*) "HGNC_id", (7*) "gene_symbol", 
    ///  (8*)  "transcript", (9) "allele_1", (10) "allele_2", (11) "variant.comment", 
    ///    (12) "age_of_onset", (13)"age_at_last_encounter", (14)  "deceased", (15) "sex", (16) "HPO", 
    /// The columns with asterisk are what we need
    /// Note: This function should be deleted after the Excel files have been converted.
    pub fn get_disease_dto_from_excel(matrix: &Vec<Vec<String>>) -> std::result::Result<DiseaseGeneData, String> {
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
            return Err("DiseaseGeneDto-related columns are not equal in all rows - requires manual check".to_string());
        }
        let disease_dto = DiseaseData{
            disease_id: first.0.clone(),
            disease_label: first.1.clone(),
        };
        let gtr_dto = GeneTranscriptData{
            hgnc_id: first.2.clone(),
            gene_symbol: first.3.clone(),
            transcript: first.4.clone(),
        };
        // Note we will need to manually fix the cohort acronym for legacy files TODO possibly refactor
        let dg_dto = DiseaseGeneData{
            disease_dto_list: vec![disease_dto],
            gene_transcript_dto_list: vec![gtr_dto],
        };
        Ok(dg_dto)
    }

    /// Builds a DTO from a Mendelian template matrix. The function calls VariantValidator to get info about all variants.
    ///
    /// # Arguments
    ///
    /// * `matrix` - A 2D vector of strings representing the Mendelian template (extracted from Excel template file).
    /// * `hpo` - Shared reference to the HPO ontology.
    /// * `update_hpo_labels` - Whether to update HPO labels automatically.
    ///
    /// # Returns
    ///
    /// A CohortDto constructed from the given legacy Excel template.
    /// 
    /// 
    /// 
    pub fn dto_from_mendelian_template<F>(
        matrix: Vec<Vec<String>>,
        hpo: Arc<FullCsrOntology>,
        update_hpo_labels: bool,
        progress_cb: F
    ) -> std::result::Result<CohortData, String> 
        where F: FnMut(u32, u32) {
        let header = HeaderDupletRow::mendelian(&matrix, hpo.clone(), update_hpo_labels)?;
        let header_hpo_count = header.hpo_count();
        const HEADER_ROWS: usize = 2; // first two rows of template are header
        let hdr_arc = Arc::new(header);
        let ppt_rows: Vec<PpktRow> = Vec::new();
        let dg_dto = Self::get_disease_dto_from_excel(&matrix)?;
        let mut vmanager = VariantManager::from_mendelian_matrix(&matrix, progress_cb)?;
        let mut row_dto_list: Vec<RowData> = Vec::new();
         for row in matrix.into_iter().skip(HEADER_ROWS) {
            let hdr_clone = hdr_arc.clone();
            let ppkt_row = PpktRow::from_mendelian_row(hdr_clone, row)?;
            if ppkt_row.hpo_count() != header_hpo_count {
                return Err(format!("Error ({}:l.{}) - PPKtRow has {} HPO columns, but the header has {} HPO columns",
                    file!(), line!(), ppkt_row.hpo_count(), header_hpo_count));
            }
            let mut allele_key_list = vec![]; // TODO
            for gv_dto in ppkt_row.get_gene_var_dto_list() {
                if gv_dto.allele1_is_present() {
                    //gv_dto.get_key_allele1()
                    if gv_dto.allele1_is_hgvs() {
                        let allele_key = HgvsVariant::generate_variant_key(&gv_dto.allele1, &gv_dto.gene_symbol, &gv_dto.transcript);
                        allele_key_list.push(allele_key);
                    } else if gv_dto.allele1_is_sv(){
                        // We do not try to guess the SV type, the user needs to adjust in the GUI
                        let allele_key = StructuralVariant::generate_variant_key(&gv_dto.allele1, &gv_dto.gene_symbol, SvType::Sv);
                        allele_key_list.push(allele_key);
                    } else {
                        return Err(format!("Unknown allele1 type {:?}", gv_dto));
                    }
                }
                if gv_dto.allele2_is_present() {
                    if gv_dto.allele2_is_hgvs() {
                        let allele_key = HgvsVariant::generate_variant_key(&gv_dto.allele2, &gv_dto.gene_symbol, &gv_dto.transcript);
                        allele_key_list.push(allele_key);
                    } else if gv_dto.allele2_is_sv(){
                        // We do not try to guess the SV type, the user needs to adjust in the GUI
                        let allele_key = StructuralVariant::generate_variant_key(&gv_dto.allele2, &gv_dto.gene_symbol, SvType::Sv);
                        allele_key_list.push(allele_key);
                    }else {
                        return Err(format!("Unknown allele2 type {:?}", gv_dto));
                    }
                }
            }
            let row_dto = RowData::from_ppkt_row(&ppkt_row, allele_key_list)?;
            if row_dto.hpo_data.len() != header_hpo_count {
                return Err(format!("Error ({}:l.{}) - RowDto has {} HPO columns, but the header has {} HPO columns",
                    file!(), line!(),row_dto.hpo_data.len(), header_hpo_count));
            }
            row_dto_list.push(row_dto);
        }
        let header_duplet_list = hdr_arc.get_hpo_header_dtos();
        
        let cohort_dto = CohortData::mendelian_with_variants(
            dg_dto, 
            header_duplet_list, 
            row_dto_list,
            vmanager.hgvs_map(), 
            vmanager.sv_map(), 
        );
        Ok(cohort_dto)
    }

    fn check_duplet(&self, duplet: &HpoTermDuplet) -> std::result::Result<(), String> {
        let term_id = match TermId::from_str(duplet.hpo_id()) {
            Ok(tid) => tid,
            Err(e) => { return Err(format!("Could not create TermId from '{}': {}", duplet.hpo_id(), e)); },
        };
        match self.hpo.term_by_id(&term_id) {
            Some(term) => {
                if term.identifier().to_string() != duplet.hpo_id() {
                    return Err(format!("Identifier '{}' did not match primary ID {}", duplet.hpo_id(), term.identifier()));
                } else if term.name() != duplet.hpo_label() {
                    return Err(format!("HPO Label '{}' did not match expected label {}", duplet.hpo_label(), term.name()));
                }
            },
            None => { return Err(format!("Could not retrieve Term from TermId '{:?}'", term_id)); },
        }

        Ok(())
    }

   

    /// Validate the current template
    ///
    ///  * Returns
    ///
    /// - a vector of errors (can be empty)
    ///
    pub fn qc_check(&self) -> Result<(), String> {

        Ok(())
    }






    /// Arranges the given HPO terms into a specific order for curation.
    ///
    /// # Arguments
    ///
    /// * `pmid` - The PubMed identifier for the new phenopacket row.
    /// * `title` - The title of the article corresponding to the pmid.
    /// * `individual_id` - The identifier of an individual described in the PMID
    /// * `hpo_items` - List of [`HpoTermDto`](struct@crate::dto::hpo_term_dto::HpoTermDto) instances describing the observed HPO features
    ///
    /// # Returns
    ///
    /// - A ``Ok(())`` upon success, otherwise ``Err(String)`.
    ///
    /// # Notes
    ///
    /// - Client code should retrieve HpoTermDto objects using the function [`Self::get_hpo_term_dto`]. This function will
    /// additionally rearrange the order of the HPO columns to keep them in "ideal" (DFS) order. Cells for HPO terms (columns) not included
    /// in the list of items but present in the columns of the previous matrix will be set to "na"
   /*  pub fn add_row_with_hpo_data(
        &mut self,
        individual_dto: IndividualDto,
        hpo_dto_items: Vec<HpoTermDto>,
        gene_variant_list: Vec<GeneVariantDto>,
        disease_gene_dto: DiseaseGeneDto
    ) -> std::result::Result<(), String> {
        let hpo_util = HpoUtil::new(self.hpo.clone());
        // === STEP 1: Extract all HPO TIDs from DTO and classify ===
        let dto_map: HashMap<TermId, String> = hpo_util.term_label_map_from_dto_list(&hpo_dto_items)?;
        let mut term_id_set: HashSet<TermId>  = dto_map.keys().cloned().collect();
        let existing_term_ids = self.header.get_hpo_id_list()?;
        term_id_set.extend(existing_term_ids);
         // === STEP 2: Arrange TIDs before borrowing template mutably ===
        let all_tids: Vec<TermId> = term_id_set.into_iter().collect();
        let mut term_arrager = HpoTermArranger::new(self.hpo.clone());
        let arranged_terms = term_arrager.arrange_terms(&all_tids)?;
         // === Step 3: Rearrange the existing PpktRow objects to have the new HPO terms set to "na"
        // 3a. Update the HeaderDupletRow object.
        let update_hdr = self.header.update_old(&arranged_terms);
        let updated_hdr_arc = Arc::new(update_hdr);
        // 3b. Update the existing PpktRow objects
        let mut updated_ppkt_rows: Vec<PpktRow> = Vec::new();
        let mut term_id_map: HashMap<TermId, String> = HashMap::new();
            for term in &arranged_terms {
                term_id_map.insert(term.identifier().clone(), "na".to_string());
            }
        for ppkt in &self.ppkt_rows {
            let mut tid_map = term_id_map.clone();
            match ppkt.update(&mut tid_map, updated_hdr_arc.clone()) {
                Ok(updated_ppkt) => { updated_ppkt_rows.push(updated_ppkt.clone());},
                Err(e) => {verrs.add_errors(e.errors());}
            }
        }
        // Now add the new phenopacket
        // 1. get map with TermId and Value (e.g., observed) for the new terms
        let mut tid_to_value_map: HashMap<TermId, String> = HashMap::new();
        for dto in   hpo_dto_items {
            let tid = dto.ontolius_term_id().map_err(|_| 
                    ValidationErrors::from_one_err(format!("Could not create TermId from {:?}", &dto)))?;
            tid_to_value_map.insert(tid, dto.entry().to_string());
        }
        let new_ppkt_result = PpktRow::from_dtos(updated_hdr_arc.clone(), individual_dto,  gene_variant_list, tid_to_value_map,   disease_gene_dto);
          println!("{}{} -- result id OK?= n={}\n\n",file!(), line!(), new_ppkt_result.is_ok());
        let ppkt_row = match new_ppkt_result {
            Ok(row) => row,
            Err(msg) => {
                verrs.push_str(msg);
                return Err(verrs);
            }
        };
        println!("{}{} -- ppkt_row={:?}\n\n",file!(), line!(), ppkt_row);
        updated_ppkt_rows.push(ppkt_row);
        self.header = updated_hdr_arc;
        self.ppkt_rows = updated_ppkt_rows;
         println!("{}{} -- ppkt rows n={}\n\n",file!(), line!(), self.ppkt_rows.len());
        
        verrs.ok()
    }*/

    

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

    /// Get Phenopackets for all individuals (rows) represented in the cohort
    /// orcid: ORCID id of the biocurator.
    pub fn extract_phenopackets(
        &self,
        cohort_dto: CohortData,
        orcid: &str) 
    -> std::result::Result<Vec<Phenopacket>, String> {
        let ppkt_list: Vec<Phenopacket> = Vec::new();
        let ppkt_exporter = PpktExporter::new(self.hpo.clone(), orcid, cohort_dto);
        ppkt_exporter.get_all_phenopackets()
    }


    /* 
    pub fn add_hpo_term_to_cohort(
        &mut self,
        hpo_id: &str,
        hpo_label: &str) 
    -> std::result::Result<(), String> {
        let tid = TermId::from_str(hpo_id)
                .map_err(|_| format!("Could not arrange terms: {}\n", hpo_id))?;
        let term = self.hpo
            .term_by_id(&tid)
            .ok_or_else(|| format!("could not retrieve HPO term for '{hpo_id}'"))?;
        // === STEP 1: Add new HPO term to existing terms and arrange TIDs ===
        let hpo_util = HpoUtil::new(self.hpo.clone());
        let mut all_tids = self.header.get_hpo_id_list()?;
        if all_tids.contains(&tid) {
            return Err(format!("Not allowed to add term {} because it already is present", &tid));
        }
        all_tids.push(tid);
        let mut term_arrager = HpoTermArranger::new(self.hpo.clone());
        let arranged_terms = term_arrager.arrange_terms(&all_tids)?;
        // === Step 3: Rearrange the existing PpktRow objects to have the new HPO terms and set the new terms to "na"
        // strategy: Make a HashMap with all of the new terms, initialize the values to na. Clone this, pass it to the
        // PpktRow object, and update the map with the current values. The remaining (new) terms will be "na". Then use
        // the new HeaderDupletRow object to write the values.
        // 3a. Update the HeaderDupletRow object.
        let update_hdr = self.header.update_old(&arranged_terms);
        let updated_hdr_arc = Arc::new(update_hdr);
        let mut updated_ppkt_rows: Vec<PpktRow> = Vec::new();
        for ppkt in &self.ppkt_rows {
            match ppkt.update_header(updated_hdr_arc.clone()) {
                Ok(new_ppkt) => {     updated_ppkt_rows.push(new_ppkt); },
                Err(e) => { return Err(e);},
            } 
        }
        self.header = updated_hdr_arc.clone();
        self.ppkt_rows = updated_ppkt_rows;
        Ok(())
    }*/
}


#[cfg(test)]
mod test {
    use crate::{dto::cohort_dto::{DiseaseData, GeneTranscriptData}};
    use ontolius::{io::OntologyLoaderBuilder};
  
    use super::*;
    use std::{fs::File, io::BufReader};
    use rstest::{fixture, rstest};
    use flate2::bufread::GzDecoder;

    #[fixture]
    fn hpo() -> Arc<FullCsrOntology> {
        let path = "resources/hp.v2025-03-03.json.gz";
        let reader = GzDecoder::new(BufReader::new(File::open(path).unwrap()));
        let loader = OntologyLoaderBuilder::new().obographs_parser().build();
        let hpo = loader.load_from_read(reader).unwrap();
        Arc::new(hpo)
    }



    #[fixture]
    fn row1() -> Vec<String> 
    {
        let row: Vec<&str> = vec![
            "PMID", "title", "individual_id", "comment", "disease_id", "disease_label", "HGNC_id",	"gene_symbol", 
            "transcript", "allele_1", "allele_2", "variant.comment", "age_of_onset", "age_at_last_encounter", 
            "deceased", "sex", "HPO",	"Clinodactyly of the 5th finger", "Hallux valgus",	"Short 1st metacarpal", 
            "Ectopic ossification in muscle tissue", "Long hallux", "Pain", "Short thumb"
        ];
        row.into_iter().map(|s| s.to_owned()).collect()
    }

    #[fixture]
    fn row2() -> Vec<String> 
    {
        let row: Vec<&str> = vec![
            "CURIE", "str", "str", "optional", "CURIE", "str", "CURIE", "str", "str", "str", "str", "optional", "age", "age", "yes/no/na", "M:F:O:U", "na",
            "HP:0004209", "HP:0001822", "HP:0010034", "HP:0011987", "HP:0001847", "HP:0012531", "HP:0009778"];
        row.into_iter().map(|s| s.to_owned()).collect()
    }

    #[fixture]
    fn row3() -> Vec<String> {
        let row: Vec<&str> =  vec![
            "PMID:29482508", "Difficult diagnosis and genetic analysis of fibrodysplasia ossificans progressiva: a case report", "current case", "", 
            "OMIM:135100", "Fibrodysplasia ossificans progressiva", "HGNC:171", "ACVR1", 
            "NM_001111067.4", "c.617G>A", "na", "NP_001104537.1:p.(Arg206His)", 
            "P9Y", "P16Y", "no", "M", "na", "na", "P16Y", "na", "P16Y", "P16Y", "P16Y", "na"];
        row.into_iter().map(|s| s.to_owned()).collect()
    }


    #[fixture]
    fn disease_gene_dto() -> DiseaseGeneData {
        let dx_dto = DiseaseData{ 
            disease_id: "OMIM:135100".to_string(), 
            disease_label: "Fibrodysplasia ossificans progressiva".to_string()
        };
        let gv_dto = GeneTranscriptData{ 
            hgnc_id: "HGNC:171".to_string(), 
            gene_symbol: "ACVR1".to_string(), 
            transcript:   "NM_001111067.4".to_string(),
        };
        DiseaseGeneData{ 
            disease_dto_list: vec![dx_dto], 
            gene_transcript_dto_list: vec![gv_dto]
        }
    }


    #[fixture]
    fn original_matrix(row1: Vec<String>, row2: Vec<String>, row3: Vec<String>)  -> Vec<Vec<String>> {
        let mut rows = Vec::with_capacity(3);
        rows.push(row1);
        rows.push(row2);
        rows.push(row3);
        rows
    }

    /// Make sure that our test matrix is valid before we start changing fields to check if we pick up errors
    #[rstest]
    fn test_factory_valid_input(
        original_matrix: Vec<Vec<String>>, 
        hpo: Arc<FullCsrOntology>) {
       // let factory = CohortDtoBuilder::from_mendelian_template(original_matrix, hpo, false);
       // assert!(factory.is_ok());
    }

/*
    /// The second HPO entry is Hallux valgus HP:0001822
    /// The third is Short 1st metacarpal HP:0010034
    /// We replace the entry in column 19
    #[rstest]
    fn test_malformed_hpo_label(mut original_matrix: Vec<Vec<String>>, hpo: Arc<FullCsrOntology>) {
        // "Hallux valgus" has extra white space
        original_matrix[0][20] = "Hallux  valgus".to_string(); 
        let factory = CohortDtoBuilder::from_mendelian_template(original_matrix, hpo, false);
        assert!(&factory.is_err());
        assert!(matches!(&factory, Err(String { .. })));
        let err_msg = factory.err().unwrap();
        let expected = "HP:0011987: expected 'Ectopic ossification in muscle tissue' but got 'Hallux  valgus'";
        assert_eq!(expected, err_msg);
    }
 */

/* TODO refactor test
    /// Test that we detect errors in labels of headings
    #[rstest]
    #[case(0, "PMI", "PMID")]
    #[case(1, "title ", "title")]
    #[case(1, " title ", "title")]
    #[case(1, "titl", "title")]
    #[case(2, "individual", "individual_id")]
    #[case(3, "comm", "comment")]
    #[case(4, "disease_i", "disease_id")]
    #[case(5, "diseaselabel", "disease_label")]
    #[case(6, "hgnc", "HGNC_id")]
    #[case(7, "symbol", "gene_symbol")]
    #[case(8, "tx", "transcript")]
    #[case(9, "allel1", "allele_1")]
    #[case(10, "allele2", "allele_2")]
    #[case(11, "vcomment", "variant.comment")]
    #[case(12, "age", "age_of_onset")]
    #[case(13, "age_last_counter", "age_at_last_encounter")]
    #[case(14, "deceasd", "deceased")]
    #[case(15, "sexcolumn", "sex")]
    #[case(16, "", "HPO")]
    fn test_malformed_title_row(
        mut original_matrix: Vec<Vec<String>>, 
        hpo: Arc<FullCsrOntology>, 
        #[case] idx: usize, 
        #[case] label: &str,
        #[case] expected_label: &str) {
        // Test that we catch malformed labels for the first row
        original_matrix[0][idx] = label.to_string(); 
        let result = CohortDtoBuilder::from_mendelian_template(original_matrix, hpo, false);
        if result.is_ok() {
            println!("{}{} {}", idx, label, expected_label);
        }
        assert!(&result.is_err());
        assert!(matches!(&result, Err(String { .. })));
        let err_msg = result.err().unwrap();
        let expected = format!("Row 0, column {}: Expected '{}' but got '{}'", 
            idx, expected_label, label);
        assert_eq!(expected, err_msg); 
    }

    // test malformed entries
    // we change entries in the third row (which is the first and only data row)
    // and introduce typical potential errors
    #[rstest]
    #[case(0, "PMID29482508", "Invalid CURIE with no colon: 'PMID29482508'")]
    #[case(0, "PMID: 29482508", "Contains stray whitespace: 'PMID: 29482508'")]
    #[case(1, "", "Value must not be empty")]
    #[case(1, "Difficult diagnosis and genetic analysis of fibrodysplasia ossificans progressiva: a case report ", 
        "Trailing whitespace in 'Difficult diagnosis and genetic analysis of fibrodysplasia ossificans progressiva: a case report '")]
    #[case(2, "individual(1)", "Forbidden character '(' found in label 'individual(1)'")]
    #[case(2, " individual A", "Leading whitespace in ' individual A'")]
    #[case(4, "MIM:135100", "Disease id has invalid prefix: 'MIM:135100'")]
    #[case(4, "OMIM: 135100", "Contains stray whitespace: 'OMIM: 135100'")]
    #[case(4, "OMIM:13510", "OMIM identifiers must have 6 digits: 'OMIM:13510'")]
    #[case(5, "Fibrodysplasia ossificans progressiva ", "Trailing whitespace in 'Fibrodysplasia ossificans progressiva '")]
    #[case(6, "HGNC:171 ", "Contains stray whitespace: 'HGNC:171 '")]
    #[case(6, "HGNC171", "Invalid CURIE with no colon: 'HGNC171'")]
    #[case(6, "HGNG:171", "HGNC id has invalid prefix: 'HGNG:171'")]
    #[case(7, "ACVR1 ", "Trailing whitespace in 'ACVR1 '")]
    #[case(8, "NM_001111067", "Transcript 'NM_001111067' is missing a version")]
    #[case(9, "617G>A", "Malformed allele '617G>A'")]
    #[case(10, "", "Value must not be empty")]
    #[case(12, "P2", "Malformed age_of_onset 'P2'")]
    #[case(13, "Adultonset", "Malformed age_at_last_encounter 'Adultonset'")]
    #[case(14, "?", "Malformed deceased entry: '?'")]
    #[case(14, "alive", "Malformed deceased entry: 'alive'")]
    #[case(15, "male", "Malformed entry in sex field: 'male'")]
    #[case(15, "f", "Malformed entry in sex field: 'f'")]
    #[case(18, "Observed", "Malformed entry for Ectopic ossification in muscle tissue (HP:0011987): 'Observed'")]
    #[case(18, "yes", "Malformed entry for Ectopic ossification in muscle tissue (HP:0011987): 'yes'")]
    #[case(18, "exc.", "Malformed entry for Ectopic ossification in muscle tissue (HP:0011987): 'exc.'")]
    fn test_malformed_entry(
        mut original_matrix: Vec<Vec<String>>, 
        hpo: Arc<FullCsrOntology>, 
        #[case] idx: usize, 
        #[case] entry: &str,
        #[case] expected_error_msg: &str) 
    {
        original_matrix[2][idx] = entry.to_string();
        let result = CohortDtoBuilder::from_mendelian_template(original_matrix, hpo, false);
        assert!(result.is_err());
        let err = result.err().unwrap();
        // TODO revise error strings, but let's do this as needed.
        //assert_eq!(expected_error_msg, err.to_string());
    }
 */

}
