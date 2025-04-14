//! PheTools
//!
//! Users interact with the library via the PheTools structure.
//! The library does not expose custom datatypes, and errors are translated
//! into strings to simplify the use of rphetools in applications


mod error;
mod hpo;
mod pptcolumn;
mod template;
mod rphetools_traits;
pub mod variant;


use crate::error::Error;
use crate::template::template_row_adder::TemplateRowAdder;
use pptcolumn::disease_gene_bundle::DiseaseGeneBundle;
use hpo::hpo_term_arranger::HpoTermArranger;
use template::individual_template::IndividualTemplateFactory;
use ontolius::ontology::MetadataAware;
use ontolius::{ontology::csr::FullCsrOntology, TermId};
use template::ppt_template::PptTemplate;
use pptcolumn::ppt_column::ColumnType;
use rphetools_traits::PyphetoolsTemplateCreator;
use std::collections::HashMap;
use std::fmt::{self};
use std::sync::Arc;
use std::{fmt::format, str::FromStr, vec};
use template::template_row_adder::MendelianRowAdder;
pub struct PheTools {
    /// Reference to the Ontolius Human Phenotype Ontology Full CSR object
    hpo: Arc<FullCsrOntology>,
    template: Option<PptTemplate>,
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

    fn set_template(&mut self, template: PptTemplate) {
        self.template = Some(template)
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
        let dgb_result = DiseaseGeneBundle::new_from_str(
            disease_id,
            disease_name,
            hgnc_id,
            gene_symbol,
            transcript_id,
        );
        match dgb_result {
            Ok(dgb) => {
                match template::template_creator::create_pyphetools_template(dgb, hpo_term_ids, &self.hpo) {
                    Ok(template) => {
                        self.set_template(template);
                        Ok(())
                    }
                    Err(e) => {
                        return Err(e.to_string()); // convert to String error for external use
                    }
                }
            }
            Err(e) => {
                return Err(e.to_string()); // convert to String error for external use
            }
        }
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

    pub fn load_excel_template(
        &mut self,
        pyphetools_template_path: &str,
    ) -> Result<(), Vec<String>> {
        let result = template::excel::read_excel_to_dataframe(pyphetools_template_path);
        match result {
            Ok(ppt_template) => {
                let ppt_res = PptTemplate::from_string_matrix(ppt_template, &self.hpo);
                match ppt_res {
                    Ok(ppt) => {
                        self.template = Some(ppt);
                    }
                    Err(e) => {
                        eprint!("Could not create ppttemplate");
                        let err_string = e.iter().map(|e| e.to_string()).collect();
                        return Err(err_string);
                    }
                }
                return Ok(());
            }
            Err(e) => {
                let err_string = vec![e.to_string()];
                return Err(err_string);
            }
        }
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

    pub fn is_hpo_col(&self, col: usize) -> bool {
        match &self.template {
            Some(tplt) => {
                if col >= tplt.column_count() {
                    return false;
                }
                match tplt.col_type_at(col) {
                    Ok(ctype) => {
                        return ctype == ColumnType::HpoTermColumn;
                    }
                    Err(_) => {
                        return false;
                    }
                }
            }
            None => false,
        }
    }

    pub fn col_type_at(&self, col: usize) -> String {
        match &self.template {
            Some(tplt) => {
                if col >= tplt.column_count() {
                    return Error::column_index_error(col, tplt.column_count()).to_string();
                }
                match tplt.col_type_at(col) {
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

    pub fn add_row(
        &mut self,
        pmid: impl Into<String>,
        title: impl Into<String>,
        individual_id: impl Into<String>,
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
                    return Err(format!("Non mendelian not implemenet"));
                }
            }
            None => Err(format!("Attempt to add row to null template!")),
        }
    }

    pub fn set_value(
        &mut self,
        row: usize,
        col: usize,
        value: impl Into<String>,
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

    pub fn get_options(
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
        let row_result = template::excel::read_excel_to_dataframe(pyphetools_template_path);
        match row_result {
            Ok(list_of_rows) => {
                let result = IndividualTemplateFactory::new(&self.hpo, list_of_rows.as_ref());
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
