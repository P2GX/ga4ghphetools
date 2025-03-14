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
}

impl <'a> PyphetoolsTemplateCreator for PheTools<'a> {
    fn create_pyphetools_template (
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

    fn arrange_terms(
        &self, 
        hpo_terms_for_curation: &Vec<TermId>
    ) -> Vec<TermId> {
        let mut term_arrager = HpoTermArranger::new(
            self.hpo
        );
        let arranged_terms = term_arrager.arrange_terms(hpo_terms_for_curation);
        arranged_terms
    }
}






pub fn qc_check(hp_json_path: &str, pyphetools_template_path: &str) {
    let list_of_rows     = excel::read_excel_to_dataframe(pyphetools_template_path);
    if list_of_rows.is_err() {
        eprintln!("[ERROR] could not read excel file: '{}", list_of_rows.err().unwrap());
        return;
    }
   
    let result =  IndividualTemplateFactory::new(hp_json_path, list_of_rows.unwrap().as_ref());
    match result {
        Ok(template_factory) => {
            let result = template_factory. get_templates();
            match result {
                Ok(template_list) => {
                    println!("[INFO] We parsed {} templates successfully.", template_list.len());
                },
                Err(errs) => {
                    eprintln!("[ERROR] We encountered errors");
                    for e in errs.messages {
                        eprintln!("[ERROR] {}", e);
                    }
                }
            }
    },
    Err(error) => {
        eprintln!("Could not create template factory! {}", error);
    }
}

}