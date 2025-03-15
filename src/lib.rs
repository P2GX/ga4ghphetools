mod allele;
mod curie;
mod disease_gene_bundle;
mod excel;
mod header;
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
    pub mod pptheaders;
}
mod simple_label;
mod simple_term;
mod template_creator;
mod transcript;
mod rphetools_traits;

use std::vec;

use hpo::hpo_term_arranger::HpoTermArranger;
use individual_template::IndividualTemplateFactory;
use ontolius::{ontology::csr::FullCsrOntology, TermId};
use rphetools_traits::PyphetoolsTemplateCreator;


pub struct PheTools<'a> {
    hpo: &'a FullCsrOntology
}

impl<'a> PheTools<'a> {
    pub fn new(hpo: &'a FullCsrOntology) -> Self {
        PheTools{hpo}
    }

    pub fn create_pyphetools_template (
        &self,
        disease_id: &str,
        disease_name: &str,
        hgnc_id: &str,
        gene_symbol: &str,
        transcript_id: &str,
        hpo_term_ids: Vec<TermId>
    ) ->  Result<Vec<Vec<String>>, String> {
        return template_creator::create_pyphetools_template(
            disease_id,
            disease_name,
            hgnc_id,
            gene_symbol,
            transcript_id,
            hpo_term_ids,
            self.hpo
        );
    }

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

    pub fn template_qc(&self, pyphetools_template_path: &str) -> Vec<String> {
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
                                                        return  errs.messages;
                                                    }
                                                }
                                            }
                            Err(e) =>  {
                                err_list.push(e);
                                return err_list;
                            },
                    }  
                }
                Err(e) =>  {
                    err_list.push(e.to_string());
                    return err_list;
                },
        }
    }


}