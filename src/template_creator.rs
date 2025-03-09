use ontolius::base::term::simple::SimpleMinimalTerm;

use crate::{disease_gene_bundle::DiseaseGeneBundle, rpyphetools_traits::PyphetoolsTemplateCreator};




pub struct TemplateCreator;

impl PyphetoolsTemplateCreator for TemplateCreator {
    fn create_pyphetools_template(disease_gene_bdl:DiseaseGeneBundle, 
                                hpo_terms: Vec<SimpleMinimalTerm>) ->   
        Vec<Vec<String>> {
        todo!()
    }
}