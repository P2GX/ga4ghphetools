//! PheTools
//!
//! Users interact with the library via the PheTools structure.
//! The library does not expose custom datatypes, and errors are translated
//! into strings to simplify the use of rphetools in applications
//! 
//! ## Features
//! 
//! - Quality assessment of phenopackets template files
//! - Generation of GA4GH Phenopackets
//! - API for curation tools




use crate::error::Error;
use crate::pptcolumn::ppt_column::PptColumn;
use crate::template::template_row_adder::TemplateRowAdder;
use crate::pptcolumn::disease_gene_bundle::DiseaseGeneBundle;
use crate::hpo::hpo_term_arranger::HpoTermArranger;
use crate::dto::{case_dto::CaseDto, hpo_term_dto::HpoTermDto};

use ontolius::ontology::{MetadataAware, OntologyTerms};
use ontolius::term::MinimalTerm;
use ontolius::{ontology::csr::FullCsrOntology, TermId};
use serde_json::to_string;
use crate::template::itemplate_factory::IndividualTemplateFactory;
use crate::template::pt_template::PheToolsTemplate;
use crate::template::template_row_adder::MendelianRowAdder;
use crate::template::{excel, template_creator};
use crate::rphetools_traits::PyphetoolsTemplateCreator;
use std::collections::{HashMap, HashSet};
use std::fmt::{self};
use std::sync::Arc;
use std::{fmt::format, str::FromStr, vec};

use super::header_index::HeaderIndexer;


pub struct PheTools {
    /// Reference to the Ontolius Human Phenotype Ontology Full CSR object
    hpo: Arc<FullCsrOntology>,
    /// Template with matrix of all values, quality control methods, and export function to GA4GH Phenopacket Schema
    template: Option<PheToolsTemplate>,
}

impl PheTools {
    /// Creates a new instance of `PheTools`.
    ///
    /// # Arguments
    ///
    /// * `hpo` - A reference to a `FullCsrOntology` that provides hierarchical phenotype data.
    ///
    /// # Returns
    ///
    /// A new `PheTools` instance.
    ///
    /// # Example
    ///
    /// ```ignore
    ///  let loader = OntologyLoaderBuilder::new()
    ///                 .obographs_parser()
    ///                 .build();
    ///  let hpo: FullCsrOntology = loader.load_from_path("hp.json")
    ///                 .expect("HPO should be loaded");
    ///  let pyphetools = PheTools::new(&hpo);
    /// ```
    pub fn new(hpo: Arc<FullCsrOntology>) -> Self {
        PheTools {
            hpo: hpo,
            template: None,
        }
    }


    /// Creates a template to be used for curating phenopackets
    ///
    /// A 2D matrix of Strings is provided for curation with the intention that curation software will
    /// fill in the matrix with additional Strings representing the cases to be curated.
    ///
    /// # Arguments
    ///
    /// * `disease_id` - A string slice representing the disease identifier.
    /// * `disease_name` - A string slice representing the name of the disease.
    /// * `hgnc_id` - A string slice representing the HGNC identifier for the gene.
    /// * `gene_symbol` - A string slice representing the gene symbol.
    /// * `transcript_id` - A string slice representing the transcript identifier.
    /// * `hpo_term_ids` - A vector of `TermId` objects representing associated HPO terms.
    ///
    /// # Returns
    ///
    /// A `Result` containing:
    /// - `Ok(())` - empty result signifying success.
    /// - `Err(String)` - An error if template generation fails.
    ///
    pub fn create_pyphetools_template(
        &mut self,
        disease_id: &str,
        disease_name: &str,
        hgnc_id: &str,
        gene_symbol: &str,
        transcript_id: &str,
        hpo_term_ids: Vec<TermId>,
    ) -> Result<(), String> {
        let dgb = DiseaseGeneBundle::new_from_str(
            disease_id,
            disease_name,
            hgnc_id,
            gene_symbol,
            transcript_id,
        ).map_err(|e| e.to_string())?;
        let template = template_creator::create_pyphetools_template(
            dgb, 
            hpo_term_ids, 
            &self.hpo
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Arranges the given HPO terms into a specific order for curation.
    ///
    /// # Arguments
    ///
    /// * `hpo_terms_for_curation` - A vector reference containing `TermId` elements that need to be arranged.
    ///
    /// # Returns
    ///
    /// A `Vec<TermId>` containing the reordered HPO terms.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let phetools = PheTools::new(&ontology);
    /// let terms = vec![TermId::from_str("HP:0001250"), TermId::from_str("HP:0004322")];
    /// let arranged_terms = phetools.arrange_terms(&terms);
    /// ```
    ///
    /// # Notes
    ///
    /// - Terms are ordered using depth-first search (DFS) over the HPO hierarchy so that related terms are displayed near each other
    pub fn arrange_terms(&self, hpo_terms_for_curation: &Vec<TermId>) -> Vec<TermId> {
        let hpo_arc = Arc::clone(&self.hpo);
        let hpo_ref = hpo_arc.as_ref();
        let mut term_arrager = HpoTermArranger::new(hpo_ref);
        let arranged_terms = term_arrager.arrange_terms(hpo_terms_for_curation);
        arranged_terms
    }

    /// Get a table of values as Strings for display/export
    /// 
    /// # Returns
    ///
    /// A `Vec<Vec<String>>` containing all of the data in the template as Strings
    pub fn get_string_matrix(&self) -> Result<Vec<Vec<String>>, String> {
        match &self.template {
            Some(template) => {
                let matrix = template.get_string_matrix().map_err(|e| e.to_string())?;
                return Ok(matrix);
            }
            None => {
                return Err(format!("Template is not initialized"));
            }
        }
    }

    /// Get a focused table of values as Strings for display/export
    /// The table contains the PMID, title, individual_id, and one HPO column (only)
    /// 
    /// # Returns `Vec<Vec<String>>` containing data for the four columns mentioned above as Strings
    pub fn get_hpo_col_with_context(&mut self, col: usize) -> Result<Vec<Vec<String>>, String> {
        match &mut self.template {
            Some(template) => {
                let matrix = template
                    .get_hpo_col_with_context(col)
                    .map_err(|e| e.to_string())?;
                return Ok(matrix);
            }
            None => {
                return Err(format!("Template is not initialized"));
            }
        }
    }

    /// Load a two dimensional String matrix representing the entire PheTools template
    pub fn load_matrix(
        &mut self, 
        matrix: Vec<Vec<String>>
    ) -> Result<(), String> 
    {
        match PheToolsTemplate::from_string_matrix(matrix, &self.hpo) {
            Ok(ppt) => {
                self.template = Some(ppt);
                Ok(())
            },
            Err(e) => {
                        eprint!("Could not create pt_template");
                        let err_string = e.iter().map(|e| e.to_string()).collect();
                        return Err(err_string);
                    }
            }
    }

    /// Transform an excel file (representing a PheTools template) into a matrix of Strings
    fn excel_template_to_matrix(
        phetools_template_path: &str,
    ) -> Result<Vec<Vec<String>>, String> 
    {
        excel::read_excel_to_dataframe(phetools_template_path)
            .map_err(|e| e.to_string())
    }

    /// Load an Excel file representing the entire PheTools template
    pub fn load_excel_template(
        &mut self,
        phetools_template_path: &str,
    ) -> Result<(), String> {
        let matrix = Self::excel_template_to_matrix( phetools_template_path)?;
        self.load_matrix(matrix)?;
        Ok(())
    }

    pub fn template_qc(&self) -> Vec<String> {
        match &self.template {
            None => {
                let msg = format!("template not initialized");
                let errs = vec![msg];
                return errs;
            }
            Some(template) => {
                vec![]
            }
        }
    }

    /// Return true if this column contains data about an HPO term
    pub fn is_hpo_col(&self, col: usize) -> bool {
        match &self.template {
            Some(tplt) => {
                tplt.is_hpo_column(col)
            },
            None => false,
        }
    }

    pub fn col_type_at(&self, col: usize) -> String {
        match &self.template {
            Some(tplt) => {
                if col >= tplt.column_count() {
                    return Error::column_index_error(col, tplt.column_count()).to_string();
                }
                match tplt.get_column_name(col) {
                    Ok(ctype) => {
                        return format!("{:?}", ctype);
                    }
                    Err(e) => {
                        return format!("{}", e);
                    }
                }
            }
            None => {
                return format!("col_type_at: template not initialized");
            }
        }
    }

    /// Adds a new row to the template but fills in only the contstant fields (disease id/label, gene id/symbol, transcript)
    pub fn add_empty_row(
        &mut self,
        pmid: &str,
        title: &str,
        individual_id: &str,
    ) -> Result<(), String> {
        match &mut self.template {
            Some(template) => {
                if template.is_mendelian() {
                    let row_adder = MendelianRowAdder {};
                    row_adder
                        .add_row(pmid, title, individual_id, template)
                        .map_err(|e| e.to_string())?;
                    Ok(())
                } else {
                    return Err(format!("Non-Mendelian not implemented"));
                }
            }
            None => Err(format!("Attempt to add row to null template!")),
        }
    }

    /// Arranges the given HPO terms into a specific order for curation.
    ///
    /// # Arguments
    ///
    /// * `pmid` - The PubMed identifier for the new phenopacket row.
    /// * `title` - The title of the article corresponding to the pmid.
    /// * `individual_id` - The identifier of an individual described in the PMID
    /// * `hpo_items` - List of [`HpoTermDto`](struct@crate::hpo::HpoTermDto) instances describing the observed HPO features
    ///
    /// # Returns
    ///
    /// A ``Ok(())`` upon success, otherwise ``Err(String)`.
    ///
    /// # Notes
    ///
    /// - Client code should retrieve HpoTermDto objects using the function [`Self::get_hpo_term_dto`]. This function will
    /// additionally rearrange the order of the HPO columns to keep them in "ideal" (DFS) order. Cells for HPO terms (columns) not included
    /// in the list of items but present in the columns of the previous matrix will be set to "na"
    pub fn add_row_with_hpo_data(
        &mut self,
        case_dto: CaseDto,
        hpo_dto_items: Vec<HpoTermDto>
    ) -> Result<(), String> {
        if self.template.is_none() {
            return Err("Template not initialized".to_string());
        }
    
        // === STEP 1: Extract all HPO TIDs from DTO and classify ===
        let dto_tid_list: Vec<String> = hpo_dto_items.iter().map(|dto| dto.term_id()).collect();
    
        let mut new_hpo_tids: Vec<TermId> = Vec::new();
        let mut dto_map: HashMap<TermId, HpoTermDto> = HashMap::new();
    
        for dto in hpo_dto_items {
            let tid = TermId::from_str(&dto.term_id())
                .map_err(|_| format!("HPO TermId {} in DTO not found", dto.term_id()))?;
    
            dto_map.insert(tid.clone(), dto);
            new_hpo_tids.push(tid);
        }
    
        // === STEP 2: Arrange TIDs before borrowing template mutably ===
        let mut all_tids: Vec<TermId> = {
            let hpo_tids = self.template.as_ref().unwrap()
                .get_hpo_term_ids()
                .map_err(|e| e.to_string())?;
    
            let mut hpo_set: HashSet<_> = hpo_tids.iter().cloned().collect();
            for tid in &new_hpo_tids {
                hpo_set.insert(tid.clone());
            }
            hpo_set.into_iter().collect()
        };
    
        let arranged_tids = self.arrange_terms(&all_tids);
    
        // === STEP 3: Create updated non-HPO columns ===
        let template = self.template.as_mut().unwrap();
    
        let indexer = HeaderIndexer::mendelian();
        let mut new_pt_columns = template.update_non_hpo_columns(case_dto, indexer)
            .map_err(|e| e.to_string())?;
    
        let mut hpo_col_map = template.get_hpo_column_map().map_err(|e| e.to_string())?;
    
        // === STEP 4: Create new columns for new HPO terms ===
        let n_existing_phenopackets = template.phenopacket_count();
        for tid in &new_hpo_tids {
            if !hpo_col_map.contains_key(tid) {
                let label = self.hpo.term_by_id(tid)
                    .ok_or_else(|| format!("Could not find {} in ontology", tid))?
                    .name().to_string();
    
                let new_col = PptColumn::new_column_with_na(tid.to_string(), label, n_existing_phenopackets)
                    .map_err(|e| e.to_string())?;
    
                hpo_col_map.insert(tid.clone(), new_col);
            }
        }
    
        // === STEP 5: Populate row from DTO map ===
        for tid in &arranged_tids {
            match hpo_col_map.get_mut(tid) {
                Some(column) => {
                    match dto_map.get(tid) {
                        Some(dto) if dto.is_excluded() => column.add_excluded(),
                        Some(dto) if dto.has_onset() => column.add_entry(dto.onset().unwrap()).map_err(|e| e.to_string())?,
                        Some(dto) if ! dto.is_ascertained() => column.add_na(),
                        Some(_) => column.add_observed(),
                        None => column.add_na(),
                    }
                    new_pt_columns.push(column.clone());
                },
                None => return Err(format!("Could not retrieve column for {}", tid)),
            }
        }
    
        // === STEP 6: Finalize ===
        template.update_columns(new_pt_columns);
        template.qc_check().map_err(|e| e.to_string())?;
    
        Ok(())
    }

    /// Edit (set) the value at a particular table cell.
    ///
    /// # Arguments
    ///
    /// * `row` - row index
    /// * `col` - column index
    /// * `value` - value to set the corresponding table cell to
    ///
    /// # Returns
    ///
    /// ``Ok(())`` if successful, otherwise ``Err(String)``
    /// # Notes
    /// 
    /// The row index includes the first two (header rows), so that the index of the first phenopacket row is 2
    pub fn set_value(
        &mut self,
        row: usize,
        col: usize,
        value: &str,
    ) -> Result<(), String> {
        match &mut self.template {
            Some(template) => {
                template
                    .set_value(row, col, value)
                    .map_err(|e| e.to_string())?;
                return Ok(());
            }
            None => {
                return Err(format!("template not initialized"));
            }
        }
    }

    /// Get a vector of options that apply for the selected table cell 
    /// (row 0 is header 1, row 1 is header 2, row 2.. are the phenopacket rows)
    ///  # Arguments
    ///
    /// * `row` - row index
    /// * `col` - column index
    /// * `addtl` - List of additional options to show
    ///
    /// # Returns
    ///
    /// ``Vec<String>`` if successful (list of options), otherwise ``Err(String)``
    /// # Notes
    /// 
    /// This function is intended to be used to create the items needed for an option menu upon right click.
    pub fn get_edit_options_for_table_cell(
        &self,
        row: usize,
        col: usize,
        addtl: Vec<String>,
    ) -> Result<Vec<String>, String> {
        match &self.template {
            Some(template) => match template.get_options(row, col, addtl) {
                Ok(options) => Ok(options),
                Err(e) => Err(e.to_string()),
            },
            None => {
                return Err(format!("template not initialized"));
            }
        }
    }

    /// Get a vector of options that apply for the selected table cell 
    /// (row 0 is header 1, row 1 is header 2, row 2.. are the phenopacket rows)
    ///  # Arguments
    ///
    /// * `row` - row index
    /// * `col` - column index
    /// * `operation` - Operation to be performed on the table cell, e.g., "na" (set value to na). See 
    ///
    /// # Returns
    ///
    /// ``Vec<String>`` if successful (list of options), otherwise ``Err(String)``
    /// # Notes
    /// 
    /// This function is intended to be used to create the items needed for an option menu upon right click.
    /// See the (private) `template::operations` module for a list of implemented operations.
    pub fn execute_operation(
        &mut self,
        row: usize,
        col: usize,
        operation: &str) -> Result<(), String>
    {
        match &mut self.template {
            Some(template) =>  {
                template.execute_operation(row, col, operation).map_err(|e| e.to_string())?;
                return Ok(());
            },
            None => {
                return Err(format!("template not initialized"));
            }
        } 
    }

    pub fn delete_row(&mut self, row: usize) -> Result<(), String> {
        match &mut self.template {
            Some(template) => {
                template.delete_row(row);
                Ok(())
            }
            None => Err(format!("template not initialized")),
        }
    }

    pub fn ncols(&self) -> usize {
        match &self.template {
            Some(template) => template.column_count(),
            None => 0,
        }
    }

    pub fn get_string_column(&self, idx: usize) -> Result<Vec<String>, String> {
        match &self.template {
            Some(template) => {
                let col = template.get_string_column(idx).map_err(|e| e.to_string())?;
                Ok(col)
            }
            None => Err(format!("phetools template not initialized")),
        }
    }

    /// get number of rows including header
    pub fn nrows(&self) -> usize {
        match &self.template {
            Some(template) => template.phenopacket_count() + template.header_row_count(),
            None => 0,
        }
    }

    /// idx refers to row including the two headers
    pub fn get_string_row(&self, idx: usize) -> Result<Vec<String>, String> {
        match &self.template {
            Some(template) => {
                let mut row: Vec<String> = Vec::new();
                for col in template.columns_iter() {
                    let elem = col.get_string(idx).map_err(|e| e.to_string())?;
                    row.push(elem);
                }
                return Ok(row);
            }
            None => Err(format!("phetools template not initialized")),
        }
    }

    pub fn template_qc_excel_file(&self, pyphetools_template_path: &str) -> Vec<String> {
        let mut err_list = Vec::new();
        let row_result = excel::read_excel_to_dataframe(pyphetools_template_path);
        match row_result {
            Ok(list_of_rows) => {
                let hpo_arc = self.hpo.clone();
                let result = IndividualTemplateFactory::new(hpo_arc, list_of_rows.as_ref());
                match result {
                    Ok(template_factory) => {
                        let result = template_factory.get_templates();
                        match result {
                            Ok(template_list) => {
                                println!(
                                    "[INFO] We parsed {} templates successfully.",
                                    template_list.len()
                                );
                                vec![]
                            }
                            Err(err) => {
                                eprintln!("[ERROR] {err}");
                                vec![]
                            }
                        }
                    }
                    Err(e) => {
                        err_list.push(e);
                        return vec![];
                    }
                }
            }
            Err(e) => {
                return vec![];
            }
        }
    }

    pub fn get_template_summary(&self) -> Result<HashMap<String, String>, String> {
        match &self.template {
            Some(template) => {
                let summary = template.get_summary();
                if summary.is_empty() {
                    return Err(format!("Empty tempalte"));
                } else {
                    return Ok(summary);
                }
            },
            None => Err(format!("Phetools template not initialized"))
        }
    }

    pub fn get_hpo_data(&self) -> HashMap<String, String> {
       let hpo_clone = Arc::clone(&self.hpo);
        let mut hpo_map: HashMap<String, String> = HashMap::new();
        let hpo_version = "ontolius update".to_ascii_lowercase(); //hpo_clone.version();
        hpo_map.insert("version".to_string(), hpo_version);
        let n_terms = hpo_clone.len();
        hpo_map.insert("n_terms".to_string(), format!("{n_terms}"));
        hpo_map      
    }

    pub fn get_hpo_term_dto(
        &self,
        tid: impl Into<String>,
        label: impl Into<String>,
        entry: impl Into<String>
    )  -> Result<HpoTermDto, String> {
        let dto = HpoTermDto::new(tid, label, entry);
        let tid: TermId = dto.term_id().parse().map_err(|e: anyhow::Error| e.to_string())?;
        let label = dto.label();
        match self.hpo.term_by_id(&tid) {
            Some(term) => {
                if term.name() != label {
                    Err(format!("Malformed HPO label {} for {} (expected: {})", label, tid.to_string(), term.name()))
                } else {
                    Ok(dto)
                }
            },
            None => {
                Err(format!("Could not find HPO term identifier {} in the ontology", tid.to_string()))
            }
        }
    }


}

impl core::fmt::Display for PheTools {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> fmt::Result {
        match &self.template {
            Some(tplt) => {
                let gene_sym = tplt.gene_symbol();
                let hgnc = tplt.hgnc();
                let dis = tplt.disease();
                let ds_id = tplt.disease_id();
                let ppkt_n = tplt.phenopacket_count();
                let hpo_v = "HPO: to-do update ontolius".to_string(); // TODO
                write!(
                    fmt,
                    r#"
{hpo_v}
phenopackets: {ppkt_n}
Gene: {gene_sym}
HGNC: {hgnc}
Disease: {dis}
Disease id: {ds_id}
"#
                )
            }
            None => write!(fmt, "Phetype template not initialized"),
        }
    }
}

// region:    --- Tests

#[cfg(test)]
mod tests {
    type Error = Box<dyn std::error::Error>;
    type Result<T> = core::result::Result<T, Error>; // For tests.

    use ontolius::io::OntologyLoaderBuilder;

    use super::*;

    #[test]
    #[ignore]
    fn test_name() -> Result<()> {
        let hpo_json = "../../data/hpo/hp.json";
        let template = "../phenopacket-store/notebooks/FBN2/input/FBN2_CCA_individuals.xlsx";
        let loader = OntologyLoaderBuilder::new().obographs_parser().build();
        let hpo: FullCsrOntology = loader
            .load_from_path(hpo_json)
            .expect("HPO should be loaded");
        let hpo_arc = Arc::new(hpo);
        let mut pyphetools = PheTools::new(hpo_arc);
        pyphetools.load_excel_template(template);
        let errors = pyphetools.template_qc();
        assert!(errors.is_empty());
        let matrix = pyphetools.get_string_matrix();
        match matrix {
            Ok(mat) => {
                println!("{:?}", mat);
            }
            Err(e) => {
                println!("{}", e)
            }
        }
        println!("setting value");
        match pyphetools.set_value(9, 31, "observed") {
            Ok(_) => println!("Able to set value"),
            Err(e) => println!("error: {}", e)
        }
        Ok(())
    }
}

// endregion: --- Tests
