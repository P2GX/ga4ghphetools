use std::str::FromStr;

use ontolius::{base::{term::simple::SimpleMinimalTerm, TermId}, ontology::csr::MinimalCsrOntology};

use crate::{disease_gene_bundle::DiseaseGeneBundle, pptcolumn::pptheaders::PptHeader, rphetools_traits::PyphetoolsTemplateCreator};




pub struct TemplateCreator;

impl PyphetoolsTemplateCreator for TemplateCreator {
    /// Create the initial pyphetools template (Table) with empty values so the curator can start to make
    /// a template with cases for a specific cohort
    /// Todo: Figure out the desired function signature.
    fn create_pyphetools_template<'a>(
        disease_id: &str,
        disease_name: &str,
        hgnc_id: &str,
        gene_symbol: &str,
        transcript_id: &str, 
        hpo_terms: Vec<SimpleMinimalTerm>,
        hpo:&'a MinimalCsrOntology) ->  Result<Vec<Vec<String>>, String> {
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
            let ppt_header = PptHeader{};
            let result = ppt_header.get_initialized_matrix(dg_bundle, 
                    &hpo_terms,
                    hpo);
            match result {
                Ok(matrix) => Ok(matrix),
                Err(err_list) => {
                    Err(err_list.join("; "))
            }        
        }
    }
}