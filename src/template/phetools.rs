




use crate::dto::etl_dto::ColumnTableDto;
use crate::dto::cohort_dto::{DiseaseGeneDto, IndividualDto,CohortDto};
use crate::dto::hgvs_variant::HgvsVariant;
use crate::dto::structural_variant::{StructuralVariant, SvType};
use crate::dto::variant_dto::VariantValidationDto;
use crate::etl::etl_tools::EtlTools;
use crate::persistence::dir_manager::DirManager;
use crate::hpo::hpo_term_arranger::HpoTermArranger;
use crate::dto::{ hpo_term_dto::HpoTermDto};
use crate::variant::hgvs_variant_validator::HgvsVariantValidator;
use crate::variant::structural_validator::StructuralValidator;

use ontolius::ontology::{MetadataAware, OntologyTerms};
use ontolius::term::MinimalTerm;
use ontolius::{ontology::csr::FullCsrOntology, TermId};
use phenopackets::schema::v2::Phenopacket;
use crate::template::cohort_dto_builder::{CohortDtoBuilder, CohortType};
use crate::template::excel;
use core::option::Option::Some;
use std::collections::HashMap;
use std::fmt::{self};
use std::fs::OpenOptions;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::{ vec};


/// The main struct for interacting with this library
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
    cohort: Option<CohortDto>,
    /// Manager to validate and cache variants
    manager: Option<DirManager>, 
    etl_tools: Option<EtlTools>,
    hgvs_validator: HgvsVariantValidator,
    sv_validator: StructuralValidator
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
            hgvs_validator: HgvsVariantValidator::hg38(),
            sv_validator: StructuralValidator::hg38()
        }
    }


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
        disease_gene_dto: DiseaseGeneDto,
        dir_path: PathBuf,
        hpo_term_ids: Vec<TermId>,
    ) -> std::result::Result<CohortDto, String> {
        if template_type != CohortType::Mendelian {
            return Err("TemplateDto generation for non-Mendelian not implemented yet".to_string());
        }
        let dirman = DirManager::new(dir_path)?;
        let hpo_arc = self.hpo.clone();
        let cohort_dto = CohortDtoBuilder::create_pyphetools_template(
            template_type, 
            disease_gene_dto,
            hpo_term_ids, 
            hpo_arc
        ).map_err(|e| e.to_string())?;
        self.manager = Some(dirman);
        Ok(cohort_dto)
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
        let mut term_arrager = HpoTermArranger::new(hpo_arc);
        
        term_arrager.arrange_term_ids(hpo_terms_for_curation)
    }

    pub fn initialize_project_dir(&mut self, project_dir: PathBuf) -> Result<(), String> {
        self.manager = Some(DirManager::new(project_dir)?);
        Ok(())
    }

    /// Load a two dimensional String matrix representing the entire PheTools template
    /// # Arguments
    ///
    /// * `matrix` - A 2D vector of strings representing the Mendelian template (extracted from Excel template file).
    /// * `fix_errors` - Whether to update HPO labels automatically.
    pub fn load_matrix<F>(
        &mut self, 
        matrix: Vec<Vec<String>>,
        fix_errors: bool,
        progress_cb: F
    ) -> Result<CohortDto, String> 
        where F: FnMut(u32,u32),{
        let hpo_arc = self.hpo.clone();
        CohortDtoBuilder::dto_from_mendelian_template(matrix, hpo_arc, fix_errors, progress_cb)
    }

    /// Transform an excel file (representing a PheTools template) into a matrix of Strings
    /// This is used to create the CohortDto object
    /// TODO delete this method once we have converted all of the existing Excel templates.
    fn excel_template_to_matrix(
        phetools_template_path: &str,
    ) -> Result<Vec<Vec<String>>, String> 
    {
        excel::read_excel_to_dataframe(phetools_template_path)
    }

    /// Load an Excel file representing a legacy Pyphentools template (Mendelian).
    /// This function can be removed once we have transformed all legacy templates.
    /// # Arguments
    ///
    /// * `template_path` - path to excel file with Phetools cohort template
    /// * `fix_errors` - Whether to update HPO labels automatically.
    pub fn load_excel_template<F>(
        &mut self,
        phetools_template_path: &str,
        fix_errors: bool,
        progress_cb: F
    ) -> Result<CohortDto, String> 
    where F: FnMut(u32,u32) {
        let matrix = Self::excel_template_to_matrix( phetools_template_path)?;
        self.load_matrix(matrix, fix_errors, progress_cb)
    }

    /// Todo: update documentation
    pub fn set_external_template_dto(
        &mut self,
        dto: &ColumnTableDto
    ) -> Result<(), String> {
        let etl = EtlTools::from_dto(self.hpo.clone(), dto);
        self.etl_tools = Some(etl);
        Ok(())
    }



    /// Add a new HPO term to the template with initial value "na". Client code can edit the new column
    ///
    /// # Arguments
    ///
    /// * `hpo_id` - HPO identifier
    /// * `hpo_label` - Corresponding HPO label
    ///
    /// # Returns
    ///
    /// ``Ok(())`` if successful, otherwise ``Err(String)``
    /// # Notes
    /// 
    /// The method returns an error if an attempt is made to add an existing HPO term. The method rearranged terms in DFS order
    pub fn add_hpo_term_to_cohort(
        &mut self,
        hpo_id: &str,
        hpo_label: &str,
        cohort_dto: CohortDto) 
    -> std::result::Result<CohortDto, String> {
        /*
        let mut updated_template = 
            CohortDtoBuilder::from_cohort_dto(  &cohort_dto, self.hpo.clone())?;
        updated_template.add_hpo_term_to_cohort(hpo_id, hpo_label)
            .map_err(|verrs| format!("{:?}", verrs.errors()))?;
        let template_dto = updated_template.get_template_dto()?;
         */
        Err("add_hpo_term_to_cohort- Needs refactor".to_string())
        
    }


    /// This function is called if the user enters information about a new phenopacket to
    /// be added to an existing cohort. The existing cohort is represented as cohort_dto
    /// (the source of truth about the cohort data comes from the frontend, is updated here, and then
    /// passed back to the frontend)
    /// Note that disease_gene_dto should match with the genes/diseases information if previous rows
    /// are present, otherwise it will seed a new template
    /// # Arguments
    ///
    /// * `individual_dto` - Information about the PMID, individual, demographics for the new row
    /// * `hpo_annotations` - list of observed/excluded HPO terms for the new row
    /// * `gene_variant_list` - list of genes/variants for the new row
    /// * `disease_gene_dto` - diseases and genes/transcripts required for the new row 
    /// * `cohort_dto` - previous cohort (source of truth), to which the new data will be added
    /// 
    /// # Returns updated cohort DTO if successful, otherwise list of strings representing errors
    pub fn add_new_row_to_cohort(
        &mut self,
        individual_dto: IndividualDto, 
        hpo_annotations: Vec<HpoTermDto>,
        variant_key_list: Vec<String>,
        cohort_dto: CohortDto) 
    -> Result<CohortDto, String> {
        let disease_gene_dto = cohort_dto.disease_gene_dto.clone();
        let mut builder = CohortDtoBuilder::new(CohortType::Mendelian, disease_gene_dto, self.hpo.clone());
        builder.add_new_row_to_cohort(individual_dto, hpo_annotations, variant_key_list, cohort_dto)
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
        tid: impl Into<String>,
        label: impl Into<String>,
        entry: impl Into<String>
    )  -> Result<HpoTermDto, String> {
        let dto = HpoTermDto::new(tid, label, entry);
        let tid: TermId = dto.ontolius_term_id().map_err(|e| e.to_string())?;
        let label = dto.label();
        self.hpo.term_by_id(&tid)
            .ok_or_else(|| format!("Could not find HPO term identifier {} in the ontology", tid))
            .and_then(|term| {
                if term.name() != label {
                    Err(format!("Malformed HPO label {} for {} (expected: {})", label, tid, term.name()))
                } else {
                    Ok(dto)
                }
            })
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


    /// Validates an HGVS variant using the VariantValidator API
    /// First we check if we already have information about the variant in 
    /// our CohortDto, which contains a HashMap of previously validated variants.
    /// # Arguments
    /// * `vv_dto` — Data transfer object containing the variant to validate
    /// # Returns
    /// - corresponding full HgvsVariant object, with information derived from VariantValidator
    pub fn validate_hgvs_variant(
        &self,
        vv_dto: VariantValidationDto,
        cohort_dto: CohortDto
    ) -> Result<HgvsVariant, String> {
        if ! vv_dto.is_hgvs() {
            return Err(format!("Attempt to HGVS-validate non-HGVS variant: {:?}", vv_dto));
        }
        let hgvs_key = HgvsVariant::generate_variant_key(&vv_dto.variant_string, &vv_dto.gene_symbol, &vv_dto.transcript);
        if let Some(hgvs_var) = cohort_dto.hgvs_variants.get(&hgvs_key) {
            return Ok(hgvs_var.clone());
        }
        self.hgvs_validator.validate(vv_dto)
    }

    /// Validates a structural variant
    /// First we check if we already have information about the variant in 
    /// our CohortDto, which contains a HashMap of previously validated variants.
    /// If we do not find it, we use VariantValidator to check the 
    /// chromosome of the variant (we need this information to determine the proper genotype)
     pub fn validate_structural_variant(
        &self,
        vv_dto: VariantValidationDto,
        cohort_dto: CohortDto
    ) -> Result<StructuralVariant, String> {
        if ! vv_dto.is_sv() {
            return Err(format!("Attempt to SV-validate non-SV variant: {:?}", vv_dto));
        }
        let sv_type: SvType = SvType::try_from(vv_dto.validation_type)?;
        let sv_key = StructuralVariant::generate_variant_key(&vv_dto.variant_string, &vv_dto.gene_symbol, sv_type);
        if let Some(sv) = cohort_dto.structural_variants.get(&sv_key) {
            return Ok(sv.clone())
        }
        self.sv_validator.validate(vv_dto)
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
        cohort_dto: CohortDto) 
    -> Result<CohortDto, String> {
        
        Err(format!("validate_all_variants -- needs refactor"))
    }
       



    /// Check correctness of a TemplateDto that was sent from the front end.
    /// This operation is performed to see if the edits made in the front end are valid.
    /// If everything is OK, we can go ahead and save the template using another command.
    /// TODO, probably combine in the same command, and add a second command to write to disk
    pub fn validate_template(
        &self, 
        cohort_dto: CohortDto) 
    -> Result<(), String> {
        //let builder = CohortDtoBuilder::from_cohort_dto(&cohort_dto, self.hpo.clone())?;
        Err("validate_template- Needs refactor".to_string())
    }

    /// Get path to directory where the cohort is stored.
    pub fn get_cohort_dir(&self) -> Option<PathBuf> {
        self.manager.as_ref().map(|dirman| dirman.get_cohort_dir())
    }

    /** Export phenopackets contained in the TemplateDto object passed from the front end (we consider
     * that the frontend possesses the single source of truth, and always update the TemplateDto object in the
     * backend). We first create a new PheToolsTemplate from the CohortDto object (because the single
     * source of truth is regard to come from the front end).
    ) 
    pub fn export_ppkt(
        &mut self,
        cohort_dto: CohortDto,
        orcid: &str) 
    -> Result<Vec<Phenopacket>, String> {
        // 1. Update PheToolsTemplate object according to DTO
       let builder = CohortDtoBuilder::from_cohort_dto(&cohort_dto, self.hpo.clone())?;
        let ppkt_list = builder.extract_phenopackets(cohort_dto, orcid)?;
        Ok(ppkt_list)
    }*/

    
    fn write_ppkt(ppkt: &Phenopacket, file_path: PathBuf) -> Result<(), String> {
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&file_path)
            .map_err(|e| e.to_string())?;
        println!("ga4ghphenotools - {}: l.{}", file!(), line!());
        serde_json::to_writer_pretty(file, &ppkt)
            .map_err(|e| e.to_string())?; 
        Ok(())
    }
    

    /// Write phenopackets to file that correspond to the current TemplateDto
    pub fn write_ppkt_list(
        &mut self,  
        cohort_dto: CohortDto, 
        dir: PathBuf,
        orcid: String) -> Result<(), String> {
        let ppkt_list: Vec<Phenopacket> = vec![];// self.export_ppkt(cohort_dto, &orcid)?;
        for ppkt in ppkt_list {
            let title = ppkt.id.clone() + ".json";
            let mut file_path = dir.clone();
            file_path.push(title);
            Self::write_ppkt(&ppkt, file_path)?;
        }
        Err("write_ppkt_list -- needs refactor".to_ascii_lowercase())
    }

    /// Load an excel file with a table of information that can be
    /// transformed into a collection of phenopackets (e.g. a Supplementary Table)
    /// row_based is true of the data for an individual is arranged in a row,
    /// and it is false if the data is arranged as a column
    pub fn load_external_excel(
        &mut self,
        external_excel_path: &str,
        row_based: bool) -> Result<ColumnTableDto, String> {
        let etl_tools = EtlTools::new(self.hpo.clone(), external_excel_path, row_based)?;
        let dto = etl_tools.raw_table().clone();
        self.etl_tools = Some(etl_tools);
        Ok(dto)
    }

}

impl core::fmt::Display for PheTools {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> fmt::Result {
         write!(fmt, "ToDo - Phetools Display")
    }
}
