
use std::str::FromStr;
use ontolius::ontology::csr::FullCsrOntology;
use ontolius::ontology::OntologyTerms;
use ontolius::term::simple::SimpleMinimalTerm;
use ontolius::term::MinimalTerm;
use ontolius::{Identified, TermId};

use crate::disease_gene_bundle::DiseaseGeneBundle;

use crate::ppt_template::PptTemplate;
use crate::error::{self, Error, Result};



/// Create the initial pyphetools template (Table) with empty values so the curator can start to make
/// a template with cases for a specific cohort
/// Todo: Figure out the desired function signature.
pub fn create_pyphetools_template<'a>(
    dg_bundle: DiseaseGeneBundle,
    hpo_term_ids: Vec<TermId>,
    hpo: &'a FullCsrOntology,
    ) ->  Result<Vec<Vec<String>>> {
        let mut smt_list: Vec<SimpleMinimalTerm> = Vec::new();
        for hpo_id in &hpo_term_ids {
            match hpo.term_by_id(hpo_id) {
                Some(term) => { 
                    let smt = SimpleMinimalTerm::new(term.identifier().clone(), term.name(), vec![], false);
                    smt_list.push(smt);},
                None => { return Err(Error::HpIdNotFound { id: hpo_id.to_string() }); }
            }
        }
       
        let result = PptTemplate::create_pyphetools_template_mendelian(dg_bundle, hpo_term_ids, hpo)?;
        return result.get_string_matrix();
}
