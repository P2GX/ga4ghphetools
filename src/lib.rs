mod allele;
mod curie;
mod disease_gene_bundle;
mod error;
mod excel;
mod simple_hpo;
mod hpo_term_template;
mod individual_template;
mod onset;
mod hpo {
    pub mod hpo_term_arranger;
}
mod pptcolumn {
    pub mod age;
    pub mod deceased;
    pub mod header_duplet;
    pub mod ppt_column;
}
mod ppt_template;
mod simple_label;
mod simple_term;
mod template_creator;
mod transcript;
mod rphetools_traits;

use std::{str::FromStr, vec};

use disease_gene_bundle::DiseaseGeneBundle;
use hpo::hpo_term_arranger::HpoTermArranger;
use individual_template::IndividualTemplateFactory;
use ontolius::{ontology::csr::FullCsrOntology, TermId};
use rphetools_traits::PyphetoolsTemplateCreator;
use crate::error::Result;

pub struct PheTools<'a> {
    /// Reference to the Ontolius Human Phenotype Ontology Full CSR object
    hpo: &'a FullCsrOntology
}

impl<'a> PheTools<'a> {
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
    pub fn new(hpo: &'a FullCsrOntology) -> Self {
        PheTools{hpo}
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
    /// - `Ok(Vec<Vec<String>>)` - A nested vector of strings representing the generated template.
    /// - `Err(ErrorType)` - An error if template generation fails.
    ///
    pub fn create_pyphetools_template (
        &self,
        disease_id: &str,
        disease_name: &str,
        hgnc_id: &str,
        gene_symbol: &str,
        transcript_id: &str,
        hpo_term_ids: Vec<TermId>
    ) ->  Result<Vec<Vec<String>>> {
        let dgb_result = DiseaseGeneBundle::new_from_str(disease_id, disease_name, hgnc_id, gene_symbol, transcript_id);
        match dgb_result {
            Ok(dgb) => {
                let tmplt_res = template_creator::create_pyphetools_template(
                    dgb,
                    hpo_term_ids,
                    self.hpo
                );
                return tmplt_res;
            }, 
            Err(e) => {
                Err(e)
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
    pub fn arrange_terms(
        &self, 
        hpo_terms_for_curation: &Vec<TermId>
    ) -> Vec<TermId> {
        let mut term_arrager = HpoTermArranger::new(
            self.hpo
        );
        let arranged_terms = term_arrager.arrange_terms(hpo_terms_for_curation);
        arranged_terms
    }


    pub fn template_qc(&self, matrix: Vec<Vec<String>>) -> Vec<String> {
        let mut err_list = Vec::new();
       


        err_list
    }


    pub fn template_qc_excel_file(&self, pyphetools_template_path: &str) -> Vec<String> {
        let mut err_list = Vec::new();
        let row_result     = excel::read_excel_to_dataframe(pyphetools_template_path);
        match row_result {
            Ok(list_of_rows) => {
                        let result =  IndividualTemplateFactory::new(self.hpo, list_of_rows.as_ref());
                        match result {
                            Ok(template_factory) => {
                                                let result = template_factory. get_templates();
                                                match result {
                                                    Ok(template_list) => {
                                                        println!("[INFO] We parsed {} templates successfully.", template_list.len());
                                                        vec![]
                                                    },
                                                    Err(errs) => {
                                                        eprintln!("[ERROR] We encountered errors");
                                                        vec![]
                                                    }
                                                }
                                            }
                            Err(e) =>  {
                                err_list.push(e);
                                return  vec![];
                            },
                    }  
                }
                Err(e) =>  {
                    
                    return  vec![];
                },
        }
    }


}