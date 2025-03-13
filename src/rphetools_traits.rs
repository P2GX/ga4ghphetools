use ontolius::{ontology::csr::MinimalCsrOntology, term::simple::SimpleMinimalTerm};



/// Create a table that will contain the fields we need to fill out.
/// We create a rows with string fields, and from this we will create the rows
/// of individual templates that will be quality-checked.
pub trait PyphetoolsTemplateCreator {
    fn create_pyphetools_template<'a>(
        disease_id: &str,
        disease_name: &str,
        hgnc_id: &str,
        gene_symbol: &str,
        transcript_id: &str,
        hpo_terms: Vec<SimpleMinimalTerm>,
        hpo:&'a MinimalCsrOntology) ->  Result<Vec<Vec<String>>, String>;
}


pub trait TableCell {
    fn value(&self) -> String;
}