




use crate::dto::etl_dto::ColumnTableDto;
use crate::dto::template_dto::{DiseaseGeneDto, GeneVariantBundleDto, IndividualBundleDto,TemplateDto};
use crate::dto::validation_errors::ValidationErrors;
use crate::dto::variant_dto::{VariantDto, VariantListDto};
use crate::etl::etl_tools::EtlTools;
use crate::persistence::dir_manager::DirManager;
use crate::hpo::hpo_term_arranger::HpoTermArranger;
use crate::dto::{ hpo_term_dto::HpoTermDto};

use ontolius::ontology::{MetadataAware, OntologyTerms};
use ontolius::term::MinimalTerm;
use ontolius::{ontology::csr::FullCsrOntology, TermId};
use phenopackets::schema::v2::Phenopacket;
use crate::template::pt_template::{PheToolsTemplate, TemplateType};
use crate::template::excel;
use core::option::Option::Some;
use std::collections::{HashMap};
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
    template: Option<PheToolsTemplate>,
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
            template: None,
            manager: None,
            etl_tools: None,
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
    /// - `Ok(())` - success.
    /// - `Err(String)` - An error if template generation fails.
    ///
    /// # TODO - implemented Melded/Digenic
    pub fn create_pyphetools_template_from_seeds(
        &mut self,
        dto: DiseaseGeneDto,
        hpo_term_ids: Vec<TermId>,
    ) -> std::result::Result<TemplateDto, String> {
        if dto.template_type != TemplateType::Mendelian {
            return Err("TemplateDto generation for non-Mendelian not implemented yet".to_string());
        }
        let hpo_arc = self.hpo.clone();
        let template = PheToolsTemplate::create_pyphetools_template(
            dto, 
            hpo_term_ids, 
            hpo_arc
        ).map_err(|e| e.to_string())?;
        let dto = template.get_template_dto().map_err(|e| e.to_string())?;
        self.template = Some(template);
        Ok(dto)
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

    /// Return a Data Transfer Object to display the entire phenopacket cohort (template)
    /// This function is called when the user opens a new template. It
    /// opens the file, creates a DTO, and sets up the directory/variant managers
    pub fn get_template_dto(&self) -> Result<TemplateDto, String> {
        match &self.template {
            Some(template) => {
                let dto = template.get_template_dto().map_err(|e| e.to_string())?;
                Ok(dto)
            }
            None => {
                Err("Template is not initialized".to_string())
            }
        }
    }




    /// Load a two dimensional String matrix representing the entire PheTools template
    pub fn load_matrix(
        &mut self, 
        matrix: Vec<Vec<String>>,
        fix_errors: bool
    ) -> Result<TemplateDto, Vec<String>> 
    {
        let hpo_arc = self.hpo.clone();
        match PheToolsTemplate::from_mendelian_template(matrix, hpo_arc, fix_errors) {
            Ok(ppt) => {
                match ppt.get_template_dto() {
                    Ok(dto) => {
                        self.template = Some(ppt);
                        Ok(dto)
                    } 
                    Err(e) => Err(vec![e.to_string()]),
                }
            },
            Err(verrs) => { Err(verrs.errors())}
        }
    }

    /// Transform an excel file (representing a PheTools template) into a matrix of Strings
    fn excel_template_to_matrix(
        phetools_template_path: &str,
    ) -> Result<Vec<Vec<String>>, Vec<String>> 
    {
        excel::read_excel_to_dataframe(phetools_template_path)
            .map_err(|e| vec![e.to_string()])
    }

    /// Load an Excel file representing the entire PheTools template
    /// Arguments
    /// - `template_path` - path to excel file with Phetools cohort template
    /// - `fix_errors` - if true, atempt to fix easily fixable errors
    pub fn load_excel_template(
        &mut self,
        phetools_template_path: &str,
        fix_errors: bool
    ) -> Result<TemplateDto, Vec<String>> {
        let matrix = Self::excel_template_to_matrix( phetools_template_path)?;
        self.load_matrix(matrix, fix_errors)
    }

    /// Here we load a JSON file that represents a partially finished
    /// transformation of an external template file
    pub fn set_external_template_dto(
        &mut self,
        dto: &ColumnTableDto) -> Result<(), String> {
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
        cohort_dto: TemplateDto) 
    -> std::result::Result<TemplateDto, Vec<String>> {
        let mut updated_template = 
            PheToolsTemplate::from_dto( self.hpo.clone(), &cohort_dto)
                .map_err(|e|vec![e])?;
        updated_template.add_hpo_term_to_cohort(hpo_id, hpo_label)
            .map_err(|verrs| verrs.errors().clone())?;
        let template_dto = updated_template.get_template_dto().map_err(|e| vec![e.to_string()])?;
        self.template = Some(updated_template);
        
        Ok(template_dto)
    }


    /// This function is called if the user enters information about a new phenopacket to
    /// be added to an existing cohort. The function will need to merge this with the
    /// existing cohort - this means mainly that we need to add na to terms used in this
    /// cohort but not in the existing phenopacket, and vice verssa
    /// # Arguments
    ///
    /// * `individual_dto` - Information about the PMID, individual, demographivds
    /// * `hpo_annotations` - list of observed/excluded HPO terms
    /// 
    /// # Returns Ok if successful, otherwise list of strings representing errors
    pub fn add_new_row_to_cohort(
        &mut self,
        individual_dto: IndividualBundleDto, 
        hpo_annotations: Vec<HpoTermDto>,
        gene_variant_list: Vec<GeneVariantBundleDto>,
        cohort_dto: TemplateDto) 
    -> Result<TemplateDto, Vec<String>> {
        let mut updated_template: PheToolsTemplate = 
            PheToolsTemplate::from_dto( self.hpo.clone(), &cohort_dto)
                .map_err(|e|vec![e])?;
        updated_template.add_row_with_hpo_data(individual_dto, hpo_annotations,gene_variant_list,  cohort_dto)
            .map_err(|verr| verr.errors().clone())?;
        let template_dto = updated_template.get_template_dto().map_err(|e| vec![e.to_string()])?;
        self.template = Some(updated_template);
        Ok(template_dto)
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

    
    /// Validate a variant sent by the front-end using a Data Transfer Object.
    /// If the variant starts with "c." or "n.", we validate it as HGVS,
    /// otherwise we validate it as a candidate Structural Variant.
    /// The method has the side effect of adding successfully validated variants to a file cache.
    /// If the variant was successfully validated, we return the same dto but with the validated flag set to true
    pub fn validate_variant(
        &mut self,
        variant_dto: VariantDto
    ) -> Result<VariantDto, String> {
        match &mut self.manager {
            Some(manager) => {
                manager.validate_variant(&variant_dto)
            },
            None => {
                Err("validate_variant: Variant Manager not initialized".to_string())
            },
        }
    }

    pub fn validate_all_variants(&mut self) -> Result<VariantListDto, ValidationErrors> {
            let verrs = ValidationErrors::new();
            todo!();
            //verrs.ok(); // TODO
    }


    pub fn validate_variant_dto_list(&mut self, variant_dto_list: Vec<VariantDto>) -> Result<Vec<VariantDto>, String> {
        match self.manager.as_mut() {
            Some(manager) => {
                Ok(manager.validate_variant_dto_list(variant_dto_list))
            },
            None => {
                Err("Variant manager not initialized".to_string())
            },
        }
    }



    /// Check correctness of a TemplateDto that was sent from the front end.
    /// This operation is performed to see if the edits made in the front end are valid.
    /// If everything is OK, we can go ahead and save the template using another command.
    /// TODO, probably combine in the same command, and add a second command to write to disk
    pub fn validate_template(
        &self, 
        cohort_dto: &TemplateDto) 
    -> Result<PheToolsTemplate, Vec<String>> {
        let template = PheToolsTemplate::from_template_dto(cohort_dto, self.hpo.clone())
            .map_err(|verrs| verrs.errors())?;
        Ok(template)
    }


    pub fn get_default_cohort_dir(&self) -> Option<PathBuf> {
        self.manager.as_ref().map(|dirman| dirman.get_cohort_dir())
    }

      /// Check correctness of a TemplateDto that was sent from the front end.
    /// This operation is performed to see if the edits made in the front end are valid.
    /// If everything is OK, we can go ahead and save the template using another command.
    /// TODO, probably combine in the same command, and add a second command to write to disk
    pub fn save_template(
        &mut self, 
        cohort_dto: &TemplateDto) 
    -> Result<(), Vec<String>> {
        let template = self.validate_template(cohort_dto)?;
        self.template = Some(template);
        Ok(())
    }

    pub fn export_ppkt(
        &mut self,
        cohort_dto: &TemplateDto) -> Result<Vec<Phenopacket>, String> {
            let template = self.validate_template(cohort_dto)
                .map_err(|_| "Could not validate template. Try again".to_string())?;
            self.template = Some(template);
            let template = match &self.template {
            Some(template) => template,
                None => {
                    return Err("Phenopacket Template not initialized".to_string());
                },
            };
            let dir_manager = match self.manager.as_mut() {
                Some(manager) => manager,
                None => {
                    return Err("Variant Manager Template not initialized".to_string());
                }
            };
            let hgvs_dict = dir_manager.get_hgvs_dict();
            let structural_dict = dir_manager.get_structural_dict();
            template.extract_phenopackets(hgvs_dict, structural_dict)
    }

    
    fn write_ppkt(ppkt: &Phenopacket, file_path: PathBuf) -> Result<(), String> {
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&file_path)
            .map_err(|e| e.to_string())?;
        serde_json::to_writer_pretty(file, &ppkt)
            .map_err(|e| e.to_string())?; 
        Ok(())
    }
    
    pub fn write_ppkt_list(&mut self,  cohort_dto: TemplateDto, dir: PathBuf) -> Result<(), String> {
        let ppkt_list: Vec<Phenopacket> = self.export_ppkt(&cohort_dto)?;
        for ppkt in ppkt_list {
            let title = ppkt.id.clone() + ".json";
            let mut file_path = dir.clone();
            file_path.push(title);
            Self::write_ppkt(&ppkt, file_path)?;
        }
        Ok(())
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
        match &self.template {
            Some(tplt) => {
                let gene_sym = "todo".to_string();
                let hgnc = "todo".to_string();
                let dis = "todo".to_string();
                let ds_id = "todo".to_string();
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
