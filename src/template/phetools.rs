




use crate::dto::template_dto::{RowDto, TemplateDto};
use crate::dto::validation_errors::ValidationErrors;
use crate::dto::variant_dto::VariantDto;
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
use std::collections::{HashMap, HashSet};
use std::fmt::{self};
use std::path::Path;
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
    ) -> Result<(), String> {
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
        let mut term_arrager = HpoTermArranger::new(hpo_arc);
        
        term_arrager.arrange_term_ids(hpo_terms_for_curation)
    }

    /// Get a table of values as Strings for display/export
    /// 
    /// # Returns
    ///
    /// - A `Vec<Vec<String>>` containing all of the data in the template as Strings
    /// # Notes
    ///
    /// - The matrix contains two header rows and then one row for each phenopacket in the template
    pub fn get_string_matrix(&self) -> Result<Vec<Vec<String>>, String> {
        match &self.template {
            Some(template) => {
                let matrix = vec![vec!["todo".to_ascii_lowercase()]];
                Ok(matrix)
            }
            None => {
                Err("Template is not initialized".to_string())
            }
        }
    }

    /// Return a Data Transfer Object to display the entire phenopacket cohort (template)
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


    /// Get a focused table of values as Strings for display/export
    /// The table contains the PMID, title, individual_id, and one HPO column (only)
    /// 
    /// # Returns 
    /// 
    /// - `Vec<Vec<String>>` containing data for the four columns mentioned above as Strings
    pub fn get_hpo_col_with_context(&mut self, col: usize) -> Result<Vec<Vec<String>>, String> {
        match &mut self.template {
            Some(template) => {
                let matrix = template
                    .get_hpo_col_with_context(col)
                    .map_err(|e| e.to_string())?;
                Ok(matrix)
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

    /// Return true if this column contains data about an HPO term
    pub fn is_hpo_col(&self, col: usize) -> bool {
        match &self.template {
            Some(tplt) => {
                tplt.is_hpo_column(col)
            },
            None => false,
        }
    }

    /// todo - is this function needed?
    pub fn col_type_at(&self, col: usize) -> Result<String, String> {
        match &self.template {
            Some(tplt) => {
                if col >= tplt.n_columns() {
                    return Err(Error::column_index_error(col, tplt.n_columns()).to_string());
                }
                match tplt.get_column_name(col) {
                    Ok(ctype) => {
                        Ok(format!("{:?}", ctype))
                    },
                    Err(e) => {
                        Err(format!("{}", e))
                    }
                }
            }
            None => {
                Err("col_type_at: template not initialized".to_string())
            }
        }
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
    /// - Client code should retrieve HpoTermDto objects using the function [`Self::get_hpo_term_dto`]. 
    ///   This function will additionally rearrange the order of the HPO columns to keep them in "ideal" (DFS) order. 
    ///   Cells for HPO terms (columns) not included
    ///   in the list of items but present in the columns of the previous matrix will be set to "na"
    pub fn add_row_with_hpo_data(
        &mut self,
        case_dto: CaseDto,
        hpo_dto_items: Vec<HpoTermDto>
    ) -> Result<(), String> {
        let hpo_util = HpoUtil::new(self.hpo.clone());
        let _ = hpo_util.check_hpo_dto(&hpo_dto_items);
        match &mut self.template {
            Some(template) => {
                template.add_row_with_hpo_data(case_dto, hpo_dto_items).map_err(|e|e.to_string())?;
                Ok(())
            }
            None => Err("Template not initialized".to_string())
        }
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
                Err("template not initialized".to_string())
            }
        }
    }

    /// Get a vector of options that apply for the selected table cell 
    /// (row 0 is header 1, row 1 is header 2, row 2.. are the phenopacket rows)
    ///  # Arguments
    ///
    /// * `row` - row index
    /// * `col` - column index
    /// * `addtl` - List of additional options to show (if present, these are added to the standard options for each column type)
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
                Err("template not initialized".to_string())
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
                Ok(())
            },
            None => {
                Err("template not initialized".to_string())
            }
        } 
    }

    pub fn delete_row(&mut self, row: usize) -> Result<(), String> {
        match &mut self.template {
            Some(template) => {
                template.delete_row(row);
                Ok(())
            }
            None => Err("template not initialized".to_string()),
        }
    }

     /// Total number of columns
    ///
    /// # Returns
    ///
    /// total number of columns (HPO and non-HPO)
    pub fn ncols(&self) -> usize {
        match &self.template {
            Some(template) => template.n_columns(),
            None => 0,
        }
    }


    /// get number of rows including header
    pub fn nrows(&self) -> usize {
        match &self.template {
            Some(template) => template.phenopacket_count() + template.header_row_count(),
            None => 0,
        }
    }

   

    pub fn get_template_summary(&self) -> Result<HashMap<String, String>, String> {
        match &self.template {
            Some(template) => {
                let summary = template.get_summary();
                if summary.is_empty() {
                    Err("Empty template".to_string())
                } else {
                    Ok(summary)
                }
            },
            None => Err("Phetools template not initialized".to_string())
        }
    }

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
        match self.hpo.term_by_id(&tid) {
            Some(term) => {
                if term.name() != label {
                    Err(format!("Malformed HPO label {} for {} (expected: {})", label, tid, term.name()))
                } else {
                    Ok(dto)
                }
            },
            None => {
                Err(format!("Could not find HPO term identifier {} in the ontology", tid))
            }
        }
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
    pub fn validate_variant(
        &mut self,
        variant_dto: VariantDto
    ) -> Result<(), String> {
        match &mut self.manager {
            Some(manager) => {
                let dto = variant_dto;
                if dto.variant_string().starts_with("c.") || dto.variant_string().starts_with("n.") {
                    manager.validate_hgvs(dto.variant_string(), dto.transcript()).map_err(|e|e.to_string())?;
                } else {
                    manager.validate_sv(dto.variant_string(), dto.hgnc_id(), dto.gene_symbol()).map_err(|e|e.to_string())?;
                }
            },
            None => {
                Err("Variant Manager not initialized".to_string())
            },
        }
        Ok(())
    }

    /// Validate all variants are are in the current template
    pub fn validate_all_variants(&mut self) -> Result<(), String> {
       // for x in self.
        match &self.template {
            Some(template) => {
            },
            None => {
                return Err(format!(""));
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
        cohort_dto: TemplateDto) 
    -> Result<(), ValidationErrors> {
        let mut verrs = ValidationErrors::new();
        match cohort_dto.cohort_type.as_str() {
            "mendelian" => Ok(()),
            _ => todo!()
        };
        /*
         pub cohort_type: String,
    pub hpo_headers: Vec<HeaderDupletDto>,
    pub rows: Vec<RowDto>
     */

        verrs.ok()
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
