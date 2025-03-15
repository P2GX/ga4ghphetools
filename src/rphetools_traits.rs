use ontolius::TermId;









pub trait PyphetoolsTemplateCreator {
    /// Create a table that will contain the fields we need to fill out.
    /// We create a rows with string fields, and from this we will create the rows
    /// of individual templates that will be quality-checked.
    fn create_pyphetools_template<'a>(
        &self,
        disease_id: &str,
        disease_name: &str,
        hgnc_id: &str,
        gene_symbol: &str,
        transcript_id: &str,
        hpo_term_ids: Vec<TermId>
    ) ->  Result<Vec<Vec<String>>, String>;

    fn arrange_terms(
        &self, 
        hpo_terms_for_curation: &Vec<TermId>
    ) -> Vec<TermId>;

    fn template_qc(&self, pyphetools_template_path: &str) -> Vec<String>;

}


pub(crate) trait TableCell {
    fn value(&self) -> String;
}