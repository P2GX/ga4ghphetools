
use std::str::FromStr;
use ontolius::ontology::csr::FullCsrOntology;
use ontolius::ontology::OntologyTerms;
use ontolius::term::simple::{SimpleMinimalTerm};
use ontolius::term::MinimalTerm;
use ontolius::{Identified, TermId};

use crate::disease_gene_bundle::DiseaseGeneBundle;


use crate::pptcolumn::pptheaders::PptHeader;




/// Create the initial pyphetools template (Table) with empty values so the curator can start to make
/// a template with cases for a specific cohort
/// Todo: Figure out the desired function signature.
pub fn create_pyphetools_template<'a>(
    disease_id: &str,
    disease_name: &str,
    hgnc_id: &str,
    gene_symbol: &str,
    transcript_id: &str, 
    hpo_term_ids: Vec<TermId>,
    hpo: &'a FullCsrOntology,
    ) ->  Result<Vec<Vec<String>>, String> {
        let disease_term_id = TermId::from_str(disease_id);
        if disease_term_id.is_err() {
            return Err(format!("Could not create TermId for disease identifier {}", disease_id));
        }
        let disease_term_id = disease_term_id.unwrap();
        let hgnc_term_id = TermId::from_str(hgnc_id);
        if hgnc_term_id.is_err() {
            return Err(format!("Could not create TermId for HGNC identifier {}", hgnc_id));
        }
        let hgnc_term_id = hgnc_term_id.unwrap();
        let dg_bundle = DiseaseGeneBundle::new(
            &disease_term_id,
            disease_name,
            &hgnc_term_id, 
            gene_symbol, 
            transcript_id)?;

        let mut smt_list: Vec<SimpleMinimalTerm> = Vec::new();
        for hpo_id in hpo_term_ids {
            match hpo.term_by_id(&hpo_id) {
                Some(term) => { 
                    let smt = SimpleMinimalTerm::new(term.identifier().clone(), term.name(), vec![], false);
                    smt_list.push(smt);},
                None => { return Err(format!("Could not retrieve term for HPO id {}", hpo_id)); }
            }
        }
        let result = PptHeader::get_initialized_matrix(dg_bundle, 
                &smt_list,
                hpo);
        match result {
            Ok(matrix) => Ok(matrix),
            Err(err_list) => {
                Err(err_list.join("; "))
        }        
    }
}
