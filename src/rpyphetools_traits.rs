use ontolius::base::term::simple::SimpleMinimalTerm;
use crate::disease_gene_bundle::DiseaseGeneBundle;




/// Create a table that will contain the fields we need to fill out.
/// We create a rows with string fields, and from this we will create the rows
/// of individual templates that will be quality-checked.
pub trait PyphetoolsTemplateCreator {
    fn create_pyphetools_template(disease_gene_bdl: DiseaseGeneBundle, hpo_terms: Vec<SimpleMinimalTerm>) ->   
        Vec<Vec<String>>;
}


pub trait TableCell {
    fn value(&self) -> String;
    
}