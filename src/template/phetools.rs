




use crate::dto::template_dto::{GeneVariantBundleDto, IndividualBundleDto, RowDto, TemplateDto};
use crate::dto::validation_errors::ValidationErrors;
use crate::dto::variant_dto::{VariantDto, VariantListDto};
use crate::error::Error;
use crate::hpo::hpo_util::HpoUtil;
use crate::persistence::dir_manager::DirManager;
use crate::template::disease_gene_bundle::DiseaseGeneBundle;
use crate::hpo::hpo_term_arranger::HpoTermArranger;
use crate::dto::{case_dto::CaseDto, hpo_term_dto::HpoTermDto};
use crate::variant::variant_validator::VariantValidator;

use ontolius::ontology::{MetadataAware, OntologyTerms};
use ontolius::term::MinimalTerm;
use ontolius::{ontology::csr::FullCsrOntology, TermId};
use phenopackets::schema::v2::Phenopacket;
use serde_json::to_string;
use crate::template::pt_template::PheToolsTemplate;
use crate::template::excel;
use crate::phetools_traits::PyphetoolsTemplateCreator;
use core::option::Option::Some;
use std::collections::{HashMap, HashSet};
use std::fmt::{self};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::{fmt::format, str::FromStr, vec};


/// The main struct for interacting with this library
pub struct PheTools {
    /// Reference to the Ontolius Human Phenotype Ontology Full CSR object
    hpo: Arc<FullCsrOntology>,
    /// Template with matrix of all values, quality control methods, and export function to GA4GH Phenopacket Schema
    template: Option<PheToolsTemplate>,
    /// Manager to validate and cache variants
    manager: Option<DirManager>, 
    variant_validator: VariantValidator,
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
            variant_validator: VariantValidator::hg38(),
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
    pub fn create_pyphetools_template(
        &mut self,
        disease_id: &str,
        disease_name: &str,
        hgnc_id: &str,
        gene_symbol: &str,
        transcript_id: &str,
        hpo_term_ids: Vec<TermId>,
    ) -> std::result::Result<PheToolsTemplate, String> {
        let dgb = DiseaseGeneBundle::new_from_str(
            disease_id,
            disease_name,
            hgnc_id,
            gene_symbol,
            transcript_id,
        ).map_err(|e| e.to_string())?;
        let hpo_arc = self.hpo.clone();
        let template = PheToolsTemplate::create_pyphetools_template(
            dgb, 
            hpo_term_ids, 
            hpo_arc
        ).map_err(|e| e.to_string())?;
        Ok(template)
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
        println!("get_template_dto");
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
        matrix: Vec<Vec<String>>
    ) -> Result<(), String> 
    {
        let hpo_arc = self.hpo.clone();
        match PheToolsTemplate::from_mendelian_template(matrix, hpo_arc) {
            Ok(ppt) => {
                self.template = Some(ppt);
                Ok(())
            },
            Err(e) => { Err(e.to_string())}
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




    /// Adds a new row to the template, filling in only the constant fields.
    ///
    /// This method is used to add a new case to the template with minimal information.
    /// It populates the following non-HPO fields `PMID`, `Title`, and `Individual ID`, and
    /// also copies the five constant fields of the disease-gene bundle.
    ///
    /// All other fields will remain empty or set to `"na"`, depending on the template's logic.
    ///
    /// # Arguments
    ///
    /// * `pmid` - The PubMed ID associated with the case.
    /// * `title` - The title of the publication or case.
    /// * `individual_id` - A unique identifier for the individual.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the row is successfully added.
    /// * `Err(String)` if an error occurs during the update.
    pub fn add_empty_row(
        &mut self,
        pmid: &str,
        title: &str,
        individual_id: &str,
    ) -> Result<(), String> {
        todo!()
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
            println!("add_new_row_to_cohort before update n={}", updated_template.phenopacket_count());
        updated_template.add_row_with_hpo_data(individual_dto, hpo_annotations,gene_variant_list,  cohort_dto)
            .map_err(|verr| verr.errors().clone())?;
        let template_dto = updated_template.get_template_dto().map_err(|e| vec![e.to_string()])?;
        println!("add_new_row_to_cohort after update n={}", updated_template.phenopacket_count());
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
            verrs.ok(); // TODO
    }


    pub fn get_variant_list_dto(&self) -> Result<VariantListDto, String> {
        match &self.manager {
            Some(manager) => {
                Ok(manager.get_variant_list_dto())
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
        cohort_dto: TemplateDto) 
    -> Result<PheToolsTemplate, ValidationErrors> {
        let template = PheToolsTemplate::from_template_dto(cohort_dto, self.hpo.clone())?;
        Ok(template)
    }

    pub fn export_phenopackets(&self) -> Result<Vec<Phenopacket>, String> {
        let ppkt_list: Vec<Phenopacket> = Vec::new();
        let template = match &self.template {
            Some(template) => template,
            None => {
                return Err(format!("Phenopacket Template not initialized"));
            },
        };
        let variant_manager = match &self.manager {
            Some(manager) => manager,
            None => {
                return Err(format!("Variant Manager Template not initialized"));
            }
        };
        Ok(template.export_phenopackets())
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

// region:    --- Tests

#[cfg(test)]
mod tests {
    type Error = Box<dyn std::error::Error>;
    type Result<T> = core::result::Result<T, Error>; // For tests.

    use ontolius::io::OntologyLoaderBuilder;

    use super::*;

 
}

// endregion: --- Tests
