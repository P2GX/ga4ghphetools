//! The main struct for interacting with this library




use crate::dto::etl_dto::{EtlDto};
use crate::dto::cohort_dto::{CohortData, GeneTranscriptData};
use crate::dto::hpo_term_dto::HpoTermDuplet;
use crate::dto::variant_dto::VariantDto;
use crate::etl::etl_tools::EtlTools;
use crate::dto::{hpo_term_dto::HpoTermData};
use crate::persistence::dir_manager::DirManager;
use crate::variant::variant_manager::VariantManager;
use ontolius::ontology::{MetadataAware, OntologyTerms};
use ontolius::term::MinimalTerm;
use ontolius::{ontology::csr::FullCsrOntology, TermId};
use crate::factory::excel;
use core::option::Option::Some;
use std::collections::HashMap;
use std::fmt::{self};
use std::path::{Path, PathBuf};
use std::sync::Arc;


pub struct PheTools {
    /// Reference to the Ontolius Human Phenotype Ontology Full CSR object
    hpo: Arc<FullCsrOntology>,
    /// Template with matrix of all values, quality control methods, and export function to GA4GH Phenopacket Schema
   // template: Option<CohortDtoBuilder>,
    /// Data transfer object to represent entire cohort. 
    /// TODO maybe we do not need to keep a copy of the DTO here since we will regard the front-end
    /// as the source of truth. The backend will need to modify and return the DTO, to read it from persistance,
    /// and to serialize it (either to save the Cohort as a JSON file or to export GA4GH phenopackets.)
    /// TODO - try to remove the PheToolsTemplate to simplify operations!
    cohort: Option<CohortData>,
    /// Manager to validate and cache variants
    manager: Option<DirManager>, 
    etl_tools: Option<EtlTools>,
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
            hpo,
            cohort: None,
            manager: None,
            etl_tools: None,
        }
    }

/* 
    /// Creates a new template to be used for curating phenopackets, initializing the disease/gene/transcript columns with the data provided.
    ///
    /// A 2D matrix of Strings is provided for curation with the intention that curation software will
    /// fill in the matrix with additional Strings representing the cases to be curated.
    ///
    /// # Arguments
    ///
    /// * `disease_id` - the disease identifier.
    /// * `disease_name` - the name of the disease.
    /// * `hgnc_id` - the HUGO Gene Nomenclature Committee (HGNC) identifier for the gene.
    /// * `gene_symbol` - the gene symbol.
    /// * `transcript_id` - the transcript identifier, e.g., NM_020928.2
    /// * `hpo_term_ids` - A vector of `TermId` objects representing associated HPO terms.
    ///
    /// # Returns
    ///
    /// A `Result` containing:
    /// - `Ok(TemplateDto)` - a cohort template object that can be used to add phenopacket data.
    /// - `Err(String)` - An error if template generation fails.
    ///
    /// # TODO - implement Melded/Digenic
    pub fn create_cohort_dto_from_seeds(
        &mut self,
        template_type: CohortType,
        disease_data: DiseaseData,
        dir_path: PathBuf,
        hpo_term_ids: Vec<TermId>,
    ) -> std::result::Result<CohortData, String> {
        if template_type != CohortType::Mendelian {
            return Err("TemplateDto generation for non-Mendelian not implemented yet".to_string());
        }
        let dirman = DirManager::new(dir_path)?;
        let hpo_arc = self.hpo.clone();
        let cohort_dto = CohortFactory::create_pyphetools_template(
            template_type, 
            disease_data,
            hpo_term_ids, 
            hpo_arc
        ).map_err(|e| e.to_string())?;
        self.manager = Some(dirman);
        Ok(cohort_dto)
    }
*/

    /// Load a two dimensional String matrix representing the entire PheTools template
    /// # Arguments
    ///
    /// * `matrix` - A 2D vector of strings representing the Mendelian template (extracted from Excel template file).
    /// * `update_hpo_labels` - Whether to update HPO labels automatically.
    /*
    pub fn load_matrix<F>(
        &mut self, 
        matrix: Vec<Vec<String>>,
        update_hpo_labels: bool,
        progress_cb: F
    ) -> Result<CohortData, String> 
        where F: FnMut(u32,u32),{
        let cohort_data = CohortFactory::dto_from_mendelian_template(matrix, self.hpo.clone(), update_hpo_labels, progress_cb)?;
        Ok(cohort_data)
    }
 */
   

    /// Transform an excel file (representing a PheTools template) into a matrix of Strings
    /// This is used to create the CohortDto object
    /// TODO delete this method once we have converted all of the existing Excel templates.
    fn excel_template_to_matrix(
        phetools_template_path: &str,
    ) -> Result<Vec<Vec<String>>, String> 
    {
        excel::read_excel_to_dataframe(phetools_template_path)
    }

 /*   /// Load an Excel file representing a legacy Pyphentools template (Mendelian).
    /// This function can be removed once we have transformed all legacy templates.
    /// # Arguments
    ///
    /// * `template_path` - path to excel file with Phetools cohort template
    /// * `update_hpo_labels` - Whether to update HPO labels automatically.
    pub fn load_excel_template<F>(
        &mut self,
        phetools_template_path: &str,
        update_hpo_labels: bool,
        progress_cb: F
    ) -> Result<CohortData, String> 
    where F: FnMut(u32,u32) {
        let matrix = Self::excel_template_to_matrix( phetools_template_path)?;
        self.load_matrix(matrix, update_hpo_labels, progress_cb)
    } */

   

    /// Todo: update documentation
    pub fn set_external_template_dto(
        &mut self,
        dto: &EtlDto
    ) -> Result<(), String> {
        let etl = EtlTools::from_dto(self.hpo.clone(), dto);
        self.etl_tools = Some(etl);
        Ok(())
    }


    /// Return information about the version and number of terms of the HPO 
    pub fn get_hpo_data(&self) -> HashMap<String, String> {
        let hpo_clone = Arc::clone(&self.hpo);
        let mut hpo_map: HashMap<String, String> = HashMap::new();
        let hpo_version = hpo_clone.version();
        hpo_map.insert("version".to_string(), hpo_version.to_string());
        let n_terms = hpo_clone.len();
        hpo_map.insert("n_terms".to_string(), format!("{n_terms}"));
        hpo_map      
    }

    /// Checks whether the HPO id and label are correct for an HpoTermDto object
    ///
    /// Not sure if we need to keep this function, maybe the QC happens somewhere else, since we are getting 
    /// the terms from the phetools HPO object anyway
    pub fn get_hpo_term_dto(
        &self,
        tid: &str,
        label: &str,
        entry: &str
    )  -> Result<HpoTermData, String> {
        let hpo_term_duplet = HpoTermDuplet::new(label, tid);
        let tid: TermId = hpo_term_duplet.to_term_id()?;
        match self.hpo.term_by_id(&tid) {
            Some(term) =>  { if term.name() != label {
                        return  Err(format!("Malformed HPO label {} for {} (expected: {})", label, tid, term.name()));
                    } 
                },
            None => {
                 return  Err(format!("Could not retrieve HPO term for {}",  tid));
            },
        }
        Ok(HpoTermData::from_duplet(hpo_term_duplet, entry)?)
    }

    /// Set the location of the directory where we will store phenopackets
    /// and will store the CohortDto as a JSON file. The legacy Excel file(s) for the
    /// gene in question are located within existing directories.
    pub fn set_cache_location<P: AsRef<Path>>(&mut self, dir_path: P) -> Result<(), String> {
        match DirManager::new(dir_path) {
            Ok(manager) => {
                self.manager = Some(manager);
            },
            Err(e) => {
                return Err(format!("Could not create directory manager: '{}", e.to_string()));
            },
        }
        Ok(())
    }


    /// Check correctness of a TemplateDto that was sent from the front end.
    /// This operation is performed to see if the edits made in the front end are valid.
    /// If everything is OK, we can go ahead and save the template using another command.
    /// TODO, probably combine in the same command, and add a second command to write to disk
    pub fn validate_template(
        &self, 
        cohort_dto: CohortData) 
    -> Result<(), String> {
        //let builder = CohortDtoBuilder::from_cohort_dto(&cohort_dto, self.hpo.clone())?;
        Err("validate_template- Needs refactor".to_string())
    }

    /// Get path to directory where the cohort is stored.
    pub fn get_cohort_dir(&self) -> Option<PathBuf> {
        self.manager.as_ref().map(|dirman| dirman.get_cohort_dir())
    }

   /*  /// Load an excel file with a table of information that can be
    /// transformed into a collection of phenopackets (e.g. a Supplementary Table)
    /// row_based is true of the data for an individual is arranged in a row,
    /// and it is false if the data is arranged as a column
    pub fn load_external_excel(
        &mut self,
        external_excel_path: &str,
        row_based: bool) -> Result<ColumnTableDto, String> {
        let etl_tools =  excel::read_external_excel_to_dto(external_excel_path, row_based)?;
        Ok(etl_tools)
    }*/

    pub fn analyze_variants(&self, cohort_dto: CohortData) 
    -> Result<Vec<VariantDto>, String> {
       if ! cohort_dto.is_mendelian() {
            return Err(format!("analyze_variants is only implemented for Mendelian"));
       }
       let disease_data = match cohort_dto.disease_list.first() {
            Some(data) => data.clone(),
            None =>  { return Err(format!("Unable to extract DiseaseData")); },
        };
        
       let gt_data: GeneTranscriptData = match disease_data.gene_transcript_list.first() {
            Some(data) => data.clone(),
            None =>  { return Err(format!("Unable to extract GeneTranscriptData")); }
        };
        let vmanager = VariantManager::from_gene_transcript_dto(&gt_data);
        vmanager.analyze_variants(cohort_dto)
    }

    /// Validates all variants in the given [`CohortDto`] that originate from
    /// legacy Excel template files.
    ///
    /// This function is intended for bulk validation of variants that were already
    /// validated in the past. While most should still validate successfully, transient
    /// errors (e.g., network issues) may cause some to fail. In such cases, the
    /// validation can be retried from the front end.
    ///
    /// For cases where a specific variant repeatedly fails validation, use
    /// [`validate_variant`] instead, as it will return the specific error encountered.
    ///
    /// # Arguments
    /// * `cohort_dto` — The current representation of the cohort
    ///
    /// # Returns
    /// The updated [`CohortDto`] on success, or a [`ValidationErrors`] containing
    /// details of all validation failures.
    /// TODO --REMOVE ME
    pub fn validate_all_variants(
        &self,
        cohort_dto: CohortData) 
    -> Result<CohortData, String> {
        
        Err(format!("validate_all_variants -- needs refactor"))
    }
       


}






impl core::fmt::Display for PheTools {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> fmt::Result {
         write!(fmt, "ToDo - Phetools Display")
    }
}
