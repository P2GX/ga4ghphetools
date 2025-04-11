use crate::{
    error::{self, Error, Result},
    ppt_template::PptTemplate,
};

pub(crate) trait TemplateRowAdder {
    fn add_row(
        &self,
        pmid: impl Into<String>,
        title: impl Into<String>,
        individual_id: impl Into<String>,
        ppt_template: &mut PptTemplate,
    ) -> Result<()>;
}

pub struct MendelianRowAdder {}
/// Add one new row that is empty except for the patient information
impl TemplateRowAdder for MendelianRowAdder {
    fn add_row(
        &self,
        pmid: impl Into<String>,
        title: impl Into<String>,
        individual_id: impl Into<String>,
        ppt_template: &mut PptTemplate,
    ) -> Result<()> {
        /// Add an empty string to all fields (columns) of the template
        /// except for the few where we know the values
        const EMPTY: &str = "";
        let row_idx = ppt_template.add_blank_row()?;
        ppt_template.set_value(row_idx, 0, pmid)?;
        ppt_template.set_value(row_idx, 1, title)?;
        ppt_template.set_value(row_idx, 2, individual_id)?;
        ppt_template.set_value(row_idx, 3, EMPTY)?; // comment
        ppt_template.set_value(row_idx, 4, ppt_template.disease())?;
        ppt_template.set_value(row_idx, 5, ppt_template.disease_id())?;
        ppt_template.set_value(row_idx, 6, ppt_template.hgnc())?;
        ppt_template.set_value(row_idx, 7, ppt_template.gene_symbol())?;
        ppt_template.set_value(row_idx, 8, ppt_template.transcript())?;
        for i in 9..=12 {
            // age onset, age observation, sex, deceased
            ppt_template.set_value(row_idx, i, EMPTY)?;
        }
        ppt_template.set_value(row_idx, 13, "na")?; // separator
                                                    // all remaining columns are HPO columns. Initialize to na
        for i in 14..ppt_template.column_count() {
            ppt_template.set_value(row_idx, i, "na")?;
        }
        Ok(())
    }
}
