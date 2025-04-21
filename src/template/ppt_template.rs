//! PptTemplate
//!
//! The struct that contains all data needed to create or edit a cohort of phenopackets
//! in "pyphetools" format, and to export GA4GH Phenopackets.

use std::{collections::HashMap, fmt::format, str::FromStr, vec};

use ontolius::{
    ontology::{csr::FullCsrOntology, OntologyTerms},
    term::{simple::SimpleMinimalTerm, MinimalTerm},
    Identified, TermId,
};

use crate::{error::{self, Error, Result}, header::header_duplet::{HeaderDuplet, HeaderDupletItem}};
use crate::{
    pptcolumn::disease_gene_bundle::DiseaseGeneBundle,
    hpo::hpo_term_arranger::HpoTermArranger,
    template::phetools_qc::PheToolsQc,
    pptcolumn::ppt_column::PptColumn,
};

/// Phetools can be used to curate cases with Mendelian disease or with melded phenotypes
#[derive(PartialEq)]
pub enum TemplateType {
    Mendelian,
    Melded,
}

/// All data needed to edit a cohort of phenopackets or export as GA4GH Phenopackets
pub struct PptTemplate {
    disease_gene_bundle: DiseaseGeneBundle,
    columns: Vec<PptColumn>,
    template_type: TemplateType,
    ptools_qc: PheToolsQc,
}

const PMID_COL: usize = 0;
const TITLE_COL: usize = 1;
const INDIVIDUAL_ID_COL: usize = 2;
const INDIVIDUAL_COMMENT: usize = 3;
const EMPTY_STRING: &str = "";

impl Error {
    fn empty_template(nlines: usize) -> Self {
        let msg = format!("Valid template must have at least three rows (at least one data row) but template has only {nlines} rows");
        Error::TemplateError { msg }
    }
    fn unequal_row_lengths() -> Self {
        let msg = format!("Not all rows of template have the same number of fields");
        Error::TemplateError { msg }
    }

    fn could_not_find_column(colname: &str) -> Self {
        let msg = format!("Could not retrieve {colname} column");
        Error::TemplateError { msg }
    }
}

impl PptTemplate {
    /// Create the initial pyphetools template (Table) with empty values so the curator can start to make
    /// a template with cases for a specific cohort
    /// Todo: Figure out the desired function signature.
    pub fn create_pyphetools_template_mendelian(
        dg_bundle: DiseaseGeneBundle,
        hpo_term_ids: Vec<TermId>,
        hpo: &FullCsrOntology,
    ) -> Result<Self> {
        let mut smt_list: Vec<SimpleMinimalTerm> = Vec::new();
        for hpo_id in hpo_term_ids {
            match hpo.term_by_id(&hpo_id) {
                Some(term) => {
                    let smt = SimpleMinimalTerm::new(
                        term.identifier().clone(),
                        term.name(),
                        vec![],
                        false,
                    );
                    smt_list.push(smt);
                }
                None => {
                    return Err(Error::HpIdNotFound {
                        id: hpo_id.to_string(),
                    });
                }
            }
        }
        let column_result = Self::get_ppt_columns(&smt_list, hpo);
        match column_result {
            // nrows is 2 at this point - we have initialized the two header rows
            Ok(columns) => Ok(Self {
                disease_gene_bundle: dg_bundle,
                columns: columns,
                template_type: TemplateType::Mendelian,
                ptools_qc: PheToolsQc::new(),
            }),
            Err(e) => Err(e),
        }
    }

    pub fn get_ppt_columns(
        hpo_terms: &Vec<SimpleMinimalTerm>,
        hpo: &FullCsrOntology,
    ) -> Result<Vec<PptColumn>> {
        let empty_col: Vec<String> = vec![]; // initialize to empty column
        let mut column_list: Vec<PptColumn> = vec![];
        column_list.push(PptColumn::pmid(&empty_col));
        column_list.push(PptColumn::title(&empty_col));
        column_list.push(PptColumn::individual_id(&empty_col));
        column_list.push(PptColumn::individual_comment(&empty_col));
        column_list.push(PptColumn::disease_id(&empty_col));
        column_list.push(PptColumn::disease_label(&empty_col));
        column_list.push(PptColumn::hgnc(&empty_col));
        column_list.push(PptColumn::gene_symbol(&empty_col));
        column_list.push(PptColumn::transcript(&empty_col));
        column_list.push(PptColumn::allele_1(&empty_col));
        column_list.push(PptColumn::allele_2(&empty_col));
        column_list.push(PptColumn::variant_comment(&empty_col));
        column_list.push(PptColumn::age_of_onset(&empty_col));
        column_list.push(PptColumn::age_at_last_encounter(&empty_col));
        column_list.push(PptColumn::deceased(&empty_col));
        column_list.push(PptColumn::sex(&empty_col));
        column_list.push(PptColumn::separator(&empty_col));

        // Arrange the HPO terms in a sensible order.
        let mut hpo_arranger = HpoTermArranger::new(hpo);
        let term_id_to_label_d: HashMap<TermId, String> = hpo_terms
            .iter()
            .map(|term| (term.identifier().clone(), term.name().to_string()))
            .collect();
        let term_ids: Vec<TermId> = term_id_to_label_d.keys().cloned().collect();
        let arranged_term_ids = hpo_arranger.arrange_terms(&term_ids);

        for tid in arranged_term_ids {
            let result = term_id_to_label_d.get(&tid);
            match result {
                Some(name) => column_list.push(PptColumn::hpo_term(name, &tid)),
                None => {
                    return Err(Error::HpIdNotFound {
                        id: tid.to_string(),
                    })
                }
            }
        }
        /* todo QC headers */
        return Ok(column_list);
    }

    /// get the total number of rows (which is 2 for the header plus the number of phenopacket rows)
    fn nrows(&self) -> Result<usize> {
        match self.columns.get(0) {
            Some(col0) => Ok(col0.phenopacket_count() + 2),
            None => Err(Error::TemplateError {
                msg: format!("Could not extract column zero"),
            }),
        }
    }

    /// A function to export a ``Vec<Vec<String>>`` matrix from the data
    ///
    /// # Returns
    ///     
    /// - `Ok(Vec<Vec<String>>)`: A 2d matrix of owned strings representing the data in the template.
    /// - `Err(std::io::Error)`: If an error occurs while transforming the data into a String matrix.
    pub fn get_string_matrix(&self) -> Result<Vec<Vec<String>>> {
        let mut rows: Vec<Vec<String>> = Vec::new();
        let nrows = self.nrows()?;
        println!("ppt_template n-columns= {}", self.column_count());
        for idx in 0..nrows {
            let mut row: Vec<String> = Vec::new();
            let mut i = 0 as usize;

            for col in &self.columns {
                //println!("Column {} row {}\n{}",i, idx,  col);
                i += 1;
                match col.get(idx) {
                    Ok(data) => row.push(data),
                    Err(e) => {
                        return Err(Error::Custom(format!(
                            "Could not retrieve column at index {idx}"
                        )));
                    }
                }
            }
            rows.push(row);
        }
        Ok(rows)
    }

    /// Generate a PptTemplate from a String matrix (e.g., from an Excel file)
    ///
    /// In most cases, we expect only one disease, but for melded genetic diagnoses we expect two
    /// We inspect the first two header rows to determine if a template has one or two diseases.
    pub fn from_string_matrix(
        matrix: Vec<Vec<String>>,
        hpo: &FullCsrOntology,
    ) -> std::result::Result<Self, Vec<Error>> {
        let mut error_list: Vec<Error> = Vec::new();
        if matrix.len() < 3 {
            error_list.push(Error::empty_template(matrix.len()));
            return Err(error_list);
        }
        // check equal length of all rows
        let row_len = matrix[0].len();
        if !matrix.iter().all(|v| v.len() == row_len) {
            error_list.push(Error::unequal_row_lengths());
            return Err(error_list);
        }
        let hdup_list = match HeaderDuplet::extract_from_string_matrix(&matrix) {
            Ok(val) => val,
            Err(e) => {
                error_list.push(e);
                return Err(error_list); // not recoverable
            }
        };
        /// TODO separate for Mendelian and Melded here
        let ptools_qc = PheToolsQc::new();
        if let Err(e) = ptools_qc.is_valid_mendelian_header(&hdup_list) {
            error_list.push(e);
        };
        // transpose the String matrix so we can create PptColumns
        let mut columns = vec![Vec::with_capacity(matrix.len()); row_len];
        const HEADER_ROWS: usize = 2;
        for row in matrix.into_iter().skip(HEADER_ROWS) {
            for (col_idx, value) in row.into_iter().enumerate() {
                columns[col_idx].push(value);
            }
        }
        let mut column_list: Vec<PptColumn> = vec![];
        let disease_id_col = PptColumn::disease_id(&columns[4]);
        let disease_id_str = match disease_id_col.get_unique() {
            Ok(val) => val,
            Err(e) => {
                error_list.push(e);
                String::default()
            }
        };
        let disease_id_str = match disease_id_col.get_unique() {
            Ok(val) => val,
            Err(e) => {
                error_list.push(e); // Capture the actual error
                String::new() // Placeholder to allow further processing
            }
        };
        let disease_id_tid = match TermId::from_str(&disease_id_str) {
            Ok(tid) => tid,
            Err(e) => {
                error_list.push(Error::termid_parse_error(&disease_id_str));
                return Err(error_list); // not recoverable
            }
        };
        let disease_label_col = PptColumn::disease_label(&columns[5]);
        let disease_label_str = match disease_label_col.get_unique() {
            Ok(val) => val,
            Err(e) => {
                error_list.push(e);
                String::default()
            }
        };
        let hgnc_col = PptColumn::hgnc(&columns[6]);
        let hgnc_str = match hgnc_col.get_unique() {
            Ok(val) => val,
            Err(e) => {
                error_list.push(e);
                String::default()
            }
        };
        let hgnc_tid = match TermId::from_str(&hgnc_str) {
            Ok(tid) => tid,
            Err(e) => {
                error_list.push(Error::termid_parse_error(&hgnc_str));
                return Err(error_list); // not recoverable
            }
        };
        let gene_symbol_col = PptColumn::gene_symbol(&columns[7]);
        let gene_symbol_str = match gene_symbol_col.get_unique() {
            Ok(val) => val,
            Err(e) => {
                error_list.push(e);
                String::default()
            }
        };
        let transcript_col = PptColumn::transcript(&columns[8]);
        let transcript_str = match transcript_col.get_unique() {
            Ok(val) => val,
            Err(e) => {
                error_list.push(e);
                String::default()
            }
        };
        let dg_bundle = match DiseaseGeneBundle::new(
            &disease_id_tid,
            disease_label_str,
            &hgnc_tid,
            gene_symbol_str,
            transcript_str,
        ) {
            Ok(val) => val,
            Err(e) => {
                error_list.push(e);
                return Err(error_list);
            }
        };
        column_list.push(PptColumn::pmid(&columns[0]));
        column_list.push(PptColumn::title(&columns[1]));
        column_list.push(PptColumn::individual_id(&columns[2]));
        column_list.push(PptColumn::individual_comment(&columns[3]));
        column_list.push(disease_id_col);
        column_list.push(disease_label_col);
        column_list.push(hgnc_col);
        column_list.push(gene_symbol_col);
        column_list.push(transcript_col);
        column_list.push(PptColumn::allele_1(&columns[9]));
        column_list.push(PptColumn::allele_2(&columns[10]));
        column_list.push(PptColumn::variant_comment(&columns[11]));
        column_list.push(PptColumn::age_of_onset(&columns[12]));
        column_list.push(PptColumn::age_at_last_encounter(&columns[13]));
        column_list.push(PptColumn::deceased(&columns[14]));
        column_list.push(PptColumn::sex(&columns[15]));
        column_list.push(PptColumn::separator(&columns[16]));
        // Every column after this must be an HPO column
        // We must have at least one HPO column for the template to be valid
        if row_len < 18 {
            error_list.push(Error::TemplateError {
                msg: format!("No HPO column found (number of columns: {})", row_len),
            });
        }
        for i in 17..row_len {
            let hp_column = PptColumn::hpo_term_from_column(&hdup_list[i], &columns[i]);
            column_list.push(hp_column);
        }
        if error_list.is_empty() {
            Ok(Self {
                disease_gene_bundle: dg_bundle,
                columns: column_list,
                template_type: TemplateType::Mendelian,
                ptools_qc: ptools_qc,
            })
        } else {
            Err(error_list)
        }
    }

    /// Validate the current template
    ///
    ///  * Returns
    ///
    /// - a vector of errors (can be empty)
    ///
    pub fn validate(&self) -> Vec<Error> {
        // for now, validate Mendelian only TODO extend for Melded
        let mut error_list: Vec<Error> = Vec::new();
        if let Err(e) = self.qc_headers() {
            error_list.push(e);
        }
        error_list
    }

    pub fn is_mendelian(&self) -> bool {
        return self.template_type == TemplateType::Mendelian;
    }

    pub fn is_hpo_column(&self, i: usize) -> bool {
        match &self.columns.get(i) {
            Some(col) => col.is_hpo_column(),
            None => false
        }
    }

    /// Get the name of the i'th column
    pub fn get_column_name(&self, i: usize) -> Result<String> {
        match &self.columns.get(i) {
            Some(column) => Ok(column.get_header_duplet().row1()),
            None => Err(Error::TemplateError {
                msg: format!("Could not get column at {i}"),
            }),
        }
    }

    pub fn get_hpo_col_with_context(&mut self, i: usize) -> Result<Vec<Vec<String>>> {
        let mut focused_cols: Vec<&PptColumn> = Vec::new();
        let pmid_col = self
            .columns
            .get(0)
            .ok_or(Error::could_not_find_column("pmid"))?;
        let title_col = self
            .columns
            .get(1)
            .ok_or(Error::could_not_find_column("title"))?;
        let ind_id_col = self
            .columns
            .get(2)
            .ok_or(Error::could_not_find_column("individual_id"))?;
        let hpo_col = self
            .columns
            .get(i)
            .ok_or(Error::could_not_find_column("hpo"))?; // should never happen
        focused_cols.push(pmid_col);
        focused_cols.push(title_col);
        focused_cols.push(ind_id_col);
        focused_cols.push(hpo_col);

        let mut rows: Vec<Vec<String>> = Vec::new();
        let nrows = self.nrows()?;

        for idx in 0..nrows {
            let mut row: Vec<String> = Vec::new();
            let mut i = 0 as usize;
            for col in &focused_cols {
                i += 1;
                match col.get(idx) {
                    Ok(data) => row.push(data),
                    Err(e) => {
                        return Err(Error::Custom(format!(
                            "Could not retrieve column at index {idx}"
                        )));
                    }
                }
            }
            rows.push(row);
        }
        Ok(rows)
    }

    fn qc_headers(&self) -> Result<()> {
        let mut headers = Vec::new();
        for c in &self.columns {
            headers.push(c.get_header_duplet());
        }
        if self.template_type == TemplateType::Mendelian {
            self.ptools_qc.is_valid_mendelian_header(&headers)?;
        }
        // TODO MELDED

        Ok(())
    }

    /// TODO we probably do not want to keep this, instead return the table
    pub fn get_string_column(&self, idx: usize) -> Result<Vec<String>> {
        if idx >= self.column_count() {
            Err(Error::column_index_error(idx, self.column_count()))
        } else {
            match self.columns.get(idx) {
                Some(col) => {
                    return Ok(col.get_string_column());
                }
                None => {
                    return Err(Error::column_index_error(idx, self.column_count()));
                }
            }
        }
    }

    pub fn columns_iter(&self) -> impl Iterator<Item = &PptColumn> {
        self.columns.iter()
    }

    pub fn phenopacket_count(&self) -> usize {
        match self.columns.get(0) {
            Some(col0) => col0.phenopacket_count(),
            None => 0,
        }
    }

    pub fn header_row_count(&self) -> usize {
        2
    }

    pub fn disease(&self) -> String {
        self.disease_gene_bundle.disease_name()
    }

    pub fn disease_id(&self) -> String {
        self.disease_gene_bundle.disease_id_as_string()
    }

    pub fn hgnc(&self) -> String {
        self.disease_gene_bundle.hgnc_id_as_string()
    }

    pub fn gene_symbol(&self) -> String {
        self.disease_gene_bundle.gene_symbol()
    }

    pub fn transcript(&self) -> String {
        self.disease_gene_bundle.transcript()
    }

    pub fn column_count(&self) -> usize {
        self.columns.len()
    }

    /// Intended to be used as part of the process to add a new
    /// row to a template.
    ///
    /// * Returns row index of the new row
    ///
    ///
    pub fn add_blank_row(&mut self) -> Result<usize> {
        for col in &mut self.columns {
            col.add_blank_field();
        }
        // check equal length of all rows
        let row_len = self.columns[0].phenopacket_count();
        if !self
            .columns
            .iter()
            .all(|v| v.phenopacket_count() == row_len)
        {
            return Err(Error::unequal_row_lengths());
        }
        // The last index is one less than the row len
        Ok(row_len - 1)
    }

    /// Delete a row. We expect this to come from a GUI where the rows include
    /// the headers (two rows) and adjust here. TODO - Consider
    /// adjusting the count in the GUI
    pub fn delete_row(&mut self, row: usize) -> Result<()> {
        let row_len = self.columns[0].phenopacket_count() + 2;
        if row < 2 {
            Err(Error::cannot_delete_header(row))
        } else if row >= row_len {
            Err(Error::delete_beyond_max_row(row, row_len))
        } else {
            for col in &mut self.columns {
                let row = row - 2; // adjust for header
                col.delete_entry_at_row(row);
            }
            Ok(())
        }
    }

    pub fn set_value(&mut self, row: usize, col: usize, value: &str) -> Result<()> {
        if row >= self.columns[0].phenopacket_count() {
            return Err(Error::row_index_error(
                row,
                self.columns[0].phenopacket_count(),
            ));
        }
        if col >= self.columns.len() {
            return Err(Error::column_index_error(col, self.columns.len()));
        }
        let mut col = &mut self.columns[col];
        col.set_value(row, value)?;
        Ok(())
    }

    pub fn get_options(&self, row: usize, col: usize, addtl: Vec<String>) -> Result<Vec<String>> {
        if col >= self.columns.len() {
            return Err(Error::column_index_error(col, self.columns.len()));
        }
        match self.columns.get(col) {
            Some(column) => {
                return Ok(column.get_options(row, col, addtl));
            }
            None => {
                // should never happen
                return Err(Error::TemplateError {
                    msg: format!("could not retrieve column"),
                });
            }
        }
        Ok(vec![])
    }

    pub fn get_summary(&self) -> HashMap<String, String> {
        let mut summary: HashMap<String, String> = HashMap::new();
        let dgb = &self.disease_gene_bundle;
        summary.insert("disease".to_string(), dgb.disease_name());
        summary.insert("disease_id".to_string(), dgb.disease_id_as_string());
        summary.insert("hgnc_id".to_string(), dgb.hgnc_id_as_string());
        summary.insert("gene_symbol".to_string(), dgb.gene_symbol());
        summary.insert("transcript".to_string(), dgb.transcript());
        let hpo_terms = self.column_count();
        summary.insert("hpo_term_count".to_string(), format!("{}", hpo_terms));
        let ppkt_n = self.phenopacket_count();
        summary.insert("n_ppkt".to_string(), format!("{}", ppkt_n));
        summary
    }
}
