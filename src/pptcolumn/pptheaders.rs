///! The pyphetools template has two header lines. For the static fields, the information is only needed from the
/// first header. For the HPO columns, the label is shown in the first header and the HPO id is
/// shown in the second field. The purpose of this struct is simply to record the strings in
/// both rows so that we can do some Q/C prior to starting to create the DataFrame object.
/// 
use std::{collections::HashMap, fmt};


use ontolius::term::simple::SimpleMinimalTerm;
use ontolius::term::MinimalTerm;
use ontolius::{ontology::csr::FullCsrOntology, Identified, TermId};
use regex::Regex;
use crate::{disease_gene_bundle::DiseaseGeneBundle, hpo::hpo_term_arranger::HpoTermArranger};

use super::header_duplet::HeaderDuplet;
use super::ppt_column::PptColumn;



 /// PptHeader: Pyphetools Header - manage the generation of the first two rows of our template.
pub struct PptHeader;


impl PptHeader {
    /* 
    pub fn get_header_duplets<'a>(hpo_terms: &Vec<SimpleMinimalTerm>, 
                                        hpo:&'a FullCsrOntology) -> Result<Vec<HeaderDuplet>, Vec<String>> {
        let mut header_duplets: Vec<HeaderDuplet> = vec![];
        let mut errors: Vec<String> = Vec::new();
        for i in 0..NUMBER_OF_CONSTANT_HEADER_FIELDS {
            header_duplets.push(HeaderDuplet::new(EXPECTED_H1_FIELDS[i], EXPECTED_H2_FIELDS[i]));
        }
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
                Some(name) => header_duplets.push(HeaderDuplet::new(name, tid.to_string())),
                None => errors.push(format!("Could not get HPO term label for {}", tid)),
            }
        }
        if ! errors.is_empty() {
            return Err(errors);
        }
       /*  if let Err(res) = Self::qc_list_of_header_items(&header_duplets) {
            return Err(errors);
        } else {
            return Ok(header_duplets);
        }*/
        Ok(header_duplets);
    } 


   
    
   
    /// When we first create the pyphetools template, we create the first two (header) lines
    /// and then we create 5 additional lines that are empty except for the constant parts
    /// i.e., information about the disease and disease gene that are constant in all lines
    fn get_empty_row(dg_bundle: &DiseaseGeneBundle, row_len: usize) -> Vec<String> {
        let mut row = vec![];
        for _ in 0..4 {
            row.push(String::new()); // empty PMID, title, individual_id, comment
        }
        row.push(dg_bundle.disease_id_as_string());
        row.push(dg_bundle.disease_name());
        row.push(dg_bundle.hgnc_id_as_string());
        row.push(dg_bundle.gene_symbol());
        row.push(dg_bundle.transcript());
        for _ in 0..7 {
             // allele_1,  allele_2, variant.comment, age_of_onset, age_at_last_encounter, deceased,sex	
            row.push(String::new());  
        }
        row.push("na".to_string()); // the HPO column has "na" as a demarker
        // calculate numer of remaining columns (where the HPO terms go)
        let remaining_columns = row_len - NUMBER_OF_CONSTANT_HEADER_FIELDS;
        for _ in 0..remaining_columns {
            row.push(String::new());  
        }


        row
    }*/
/* 
    pub fn get_initialized_matrix<'a>(dg_bundle: DiseaseGeneBundle, 
                                      hpo_terms: &Vec<SimpleMinimalTerm>,
                                      hpo:&'a FullCsrOntology) -> 
                    Result<Vec<Vec<String>>, Vec<String>>   {
        let header_duplets = Self::get_header_duplets(hpo_terms, hpo)?;
       // Self::qc_list_of_header_items(&header_duplets)?;
        let mut errors = vec![];
        let mut matrix: Vec<Vec<String>> = vec![];
        // initialize the first two rows
        let n_columns = header_duplets.len();
        let mut row1 = vec![];
        let mut row2 = vec![];
        for i in 0..n_columns {
            match header_duplets.get(i) {
                Some(hd) => {
                    row1.push(hd.row1());
                    row2.push(hd.row2());
                },
                None => {
                    errors.push(format!("Could not retrieve header for index {}", i));
                }
            }
        }
        matrix.push(row1);
        matrix.push(row2);
        // We add a default value of 5 additional rows
        for _ in 0..5 {
            matrix.push(Self::get_empty_row(&dg_bundle, n_columns));
        }


        Ok(matrix)
    }
*/

}


