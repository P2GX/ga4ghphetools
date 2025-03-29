//! Pyphetools Template
//! 
//! The struct that contains all data needed to create or edit a cohort of phenopackets
//! in "pyphetools" format, and to export GA4GH Phenopackets.

use std::{collections::HashMap, fmt::format, str::FromStr};

use ontolius::{ontology::{csr::FullCsrOntology, OntologyTerms}, term::{simple::SimpleMinimalTerm, MinimalTerm}, Identified, TermId};


use crate::{disease_gene_bundle::DiseaseGeneBundle, hpo::hpo_term_arranger::HpoTermArranger, pptcolumn::ppt_column::PptColumn};
use crate::error::{self, Error, Result};

pub enum TemplateType {
    Mendelian,
    Melded
}

/// All data needed to edit a cohort of phenopackets or export as GA4GH Phenopackets
pub struct PptTemplate {
    disease_gene_bundle: DiseaseGeneBundle,
    columns: Vec<PptColumn>,
    nrows: usize,
    templateType: TemplateType
}


impl PptTemplate {



    /// Create the initial pyphetools template (Table) with empty values so the curator can start to make
    /// a template with cases for a specific cohort
    /// Todo: Figure out the desired function signature.
    pub fn create_pyphetools_template_mendelian<'a>(
        dg_bundle: DiseaseGeneBundle,
        hpo_term_ids: Vec<TermId>,
        hpo: &'a FullCsrOntology,
        ) ->  Result<Self> {
            

            let mut smt_list: Vec<SimpleMinimalTerm> = Vec::new();
            for hpo_id in hpo_term_ids {
                match hpo.term_by_id(&hpo_id) {
                    Some(term) => { 
                        let smt = SimpleMinimalTerm::new(term.identifier().clone(), term.name(), vec![], false);
                        smt_list.push(smt);},
                    None => { return Err(Error::HpIdNotFound { id: hpo_id.to_string() } ); }
                }
            }
            let column_result = Self::get_ppt_columns(&smt_list, hpo);
            match column_result {
                 // nrows is 2 at this point - we have initialized the two header rows
                Ok(columns) => {
                     Ok(Self {
                        disease_gene_bundle: dg_bundle,
                        columns: columns,
                        nrows: 2 as usize,
                        templateType: TemplateType::Mendelian
                    })
                },
                Err(e) => Err(e)
            }
        }


    pub fn get_ppt_columns<'a>(
        hpo_terms: &Vec<SimpleMinimalTerm>, 
        hpo:&'a FullCsrOntology
    ) -> Result<Vec<PptColumn>> {
        let mut column_list: Vec<PptColumn> = vec![];
        column_list.push(PptColumn::pmid());
        column_list.push(PptColumn::title());
        column_list.push(PptColumn::individual_id());
        column_list.push(PptColumn::individual_comment());
        column_list.push(PptColumn::disease_id());
        column_list.push(PptColumn::disease_label());
        column_list.push(PptColumn::hgnc());
        column_list.push(PptColumn::gene_symbol());
        column_list.push(PptColumn::transcript());
        column_list.push(PptColumn::allele_1());
        column_list.push(PptColumn::allele_2());
        column_list.push(PptColumn::variant_comment());
        column_list.push(PptColumn::age_of_onset());
        column_list.push(PptColumn::age_at_last_encounter());
        column_list.push(PptColumn::deceased());
        column_list.push(PptColumn::sex());
        column_list.push(PptColumn::separator());

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
                None => return Err(Error::HpIdNotFound { id: tid.to_string() }),
            }
        }
        /* todo QC headers */
        return Ok(column_list);
    }

    /// A function to export a Vec<Vec<String>> matrix from the data
    /// 
    /// # Returns
    ///     
    /// - `Ok(Vec<Vec<String>>)`: A 2d matrix of owned strings representing the data in the template.
    /// - `Err(std::io::Error)`: If an error occurs while transforming the data into a String matrix.
    pub fn get_string_matrix(&self) -> Result<Vec<Vec<String>>> {
        let mut rows: Vec<Vec<String>> = Vec::new();
        for idx in 0..self.nrows {
            let mut row: Vec<String> = Vec::new();
            for col in &self.columns {
                match col.get(idx) {
                    Ok(data) => row.push(data),
                    Err(e) => {
                        return Err(Error::Custom(format!("Could not retrieve column at index {idx}")));
                    }
                }
            }
            rows.push(row);
        }
        Ok(rows)
    }
    
}
