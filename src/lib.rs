//! PheTools
//! 
//! Users interact with the library via the PheTools structure.
//! The library does not expose custom datatypes, and errors are translated into strings to 
//! simplify the use of rphetools in applications


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
mod phetools_qc;
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

use std::{fmt::format, str::FromStr, vec};

use disease_gene_bundle::DiseaseGeneBundle;
use hpo::hpo_term_arranger::HpoTermArranger;
use individual_template::IndividualTemplateFactory;
use ontolius::{ontology::csr::FullCsrOntology, TermId};
use ppt_template::PptTemplate;
use rphetools_traits::PyphetoolsTemplateCreator;
use crate::error::Error;


pub struct PheTools<'a> {
    /// Reference to the Ontolius Human Phenotype Ontology Full CSR object
    hpo: &'a FullCsrOntology,
    template: Option<PptTemplate<'a>>
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
        PheTools{
            hpo: hpo,
            template: None,
        } 
    }

    fn set_template(&mut self, template:  PptTemplate<'a>) {
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
    pub fn create_pyphetools_template (
        &mut self,
        disease_id: &str,
        disease_name: &str,
        hgnc_id: &str,
        gene_symbol: &str,
        transcript_id: &str,
        hpo_term_ids: Vec<TermId>
    ) ->  Result<(), String> {
        let dgb_result = DiseaseGeneBundle::new_from_str(disease_id, disease_name, hgnc_id, gene_symbol, transcript_id);
        match dgb_result {
            Ok(dgb) => {
                match template_creator::create_pyphetools_template(
                    dgb,
                    hpo_term_ids,
                    self.hpo) {
                        Ok(template) => {
                            self.set_template(template);
                            Ok(())
                        },
                        Err(e) => {
                            return Err(e.to_string()); // convert to String error for external use
                        }
                }
                 
            }, 
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


    pub fn load_excel_template(&mut self, pyphetools_template_path: &str) 
        -> Result<(), String> 
        {
            let result    = excel::read_excel_to_dataframe(pyphetools_template_path);
            match result {
                Ok(ppt_template) => { 
                    let ppt_res = PptTemplate::from_string_matrix(ppt_template, self.hpo);
                    match ppt_res {
                        Ok(ppt) => {
                            self.template = Some(ppt);
                        },
                        Err(e) => {
                            eprint!("Could not create ppttemplate: {}", e);
                            return Err(e.to_string());}
                    }
                    
                    
                    return Ok(()); },
                Err(e) => {
                    return Err(e.to_string()); }
            }
    }

    pub fn template_qc(&self) -> Vec<String> {
        match &self.template {
            None => {
                let msg = format!("template not initialized");
                let errs = vec![msg];
                return errs;
            },
            Some(template) => {
                vec![]
            }
        }
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
                                Err(err) => {
                                    eprintln!("[ERROR] {err}");
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
        let loader = OntologyLoaderBuilder::new()
        .obographs_parser()
        .build();
    let hpo: FullCsrOntology = loader.load_from_path(hpo_json)
                                                .expect("HPO should be loaded");
        let mut pyphetools = PheTools::new(&hpo);
        pyphetools.load_excel_template(template);
        let errors = pyphetools.template_qc();
        assert!(errors.is_empty());
    
        Ok(())
    }
}

// endregion: --- Tests