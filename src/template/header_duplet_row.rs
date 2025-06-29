//! HeaderDupletRow: Encapsulate the headers (which we call duplets because each has two fields and which are serialized as the furst two rows of the template)
//! 
//! Each HeaderDuplet determines the meaning of the rows beneath it.
//! We pass a reference (via ARC) of the HeaderDupletRow to each of the rows of the template



use std::collections::HashMap;
use std::fmt::format;
use std::hash::DefaultHasher;
use std::str::FromStr;
use std::sync::Arc;
use std::usize;

use ontolius::ontology::csr::FullCsrOntology;
use ontolius::ontology::OntologyTerms;
use ontolius::term::simple::SimpleTerm;
use ontolius::term::{MinimalTerm, Term};
use ontolius::{Identified, TermId};
use serde::de;

use crate::dto::hpo_term_dto::HpoTermDto;
use crate::dto::template_dto::{HeaderDupletDto};
use crate::dto::validation_errors::ValidationErrors;
use crate::header::disease_header::DiseaseHeader;
use crate::header::duplet_item::DupletItem;
use crate::header::gene_variant_header::GeneVariantHeader;
use crate::header::hpo_term_duplet::HpoTermDuplet;
use crate::header::individual_header::IndividualHeader;
use crate::error::{self, Error, Result};
use crate::hpo::hpo_util::HpoUtil;
use crate::template::disease_bundle::DiseaseBundle;
use crate::template::individual_bundle::IndividualBundle;
use crate::template::pt_template::TemplateType;

const NOT_AVAILABLE: &str = "na";
const EXCLUDED: &str = "excluded";
const OBSERVED: &str = "observed";





const MENDELIAN_INDIVIDUAL_IDX: usize = 0; 
const MENDELIAN_DISEASE_IDX: usize = 4;
const MENDELIAN_GENE_VAR_IDX: usize = 6;
const MENDELIAN_DEMOGRAPHIC_IDX: usize = 12;
const MENDELIAN_SEPARATOR_IDX: usize = 16;
const MENDELIAN_HPO_IDX: usize = 17;


/// Number of columns in the Individual section
const NUMBER_OF_INDIVIDUAL_FIELDS: usize = 4;
/// Number of columns in the Disease/Gene/Variant bundle section
const NUMBER_OF_DISEASE_GENE_BUNDLE_FIELDS: usize = 8;
/// Number of columns in the Demographic section
const NUMBER_OF_DEMOGRAPHIC_FIELDS: usize = 4;
/// Separator field (HPO/na)
const NUMBER_OF_SEPARATOR_FIELDS: usize = 1;

const INDIVIDUAL_IDX: usize = 0;
const DISEASE_IDX: usize = 4;
const GENE_VAR_IDX: usize = 6;
const DEMOGRAPHIC_IDX: usize = 12;
const SEPARATOR_IDX: usize = 16;
const HPO_SECTION_IDX: usize = 17;


/// Total number of constant fields (columns) in the Mendelian template
const N_CONSTANT_FIELDS_MENDELIAN: usize = 
    NUMBER_OF_INDIVIDUAL_FIELDS + NUMBER_OF_DISEASE_GENE_BUNDLE_FIELDS + NUMBER_OF_DEMOGRAPHIC_FIELDS + NUMBER_OF_SEPARATOR_FIELDS;




#[derive(Clone, Debug)]
pub struct HeaderDupletRow {
    individual_header: IndividualHeader,
    disease_header_list: Vec<DiseaseHeader>,
    gene_variant_header_list: Vec<GeneVariantHeader>,
    hpo_duplets: Vec<HpoTermDuplet>,
    template_type: TemplateType,
}


impl HeaderDupletRow {
    pub fn mendelian(
        matrix: &Vec<Vec<String>>,
        hpo: Arc<FullCsrOntology>,
    ) -> std::result::Result<Self, ValidationErrors> {
        Self::qc_matrix_dimensions(matrix)?;
        /// first Q/C the constant part of the Mendelian header
        let iheader = IndividualHeader::from_matrix(matrix, MENDELIAN_DEMOGRAPHIC_IDX)?;
        let dheader = DiseaseHeader::from_matrix(matrix, MENDELIAN_DISEASE_IDX)?;
        let gheader = GeneVariantHeader::from_matrix(matrix, MENDELIAN_GENE_VAR_IDX)?;
        /// If we get here, the constant part is OK and we can check the HPO columns
        let mut hpo_duplet_list: Vec<HpoTermDuplet> = Vec::new();
        let n = matrix[0].len(); // previously checked in qc_matrix_dimensions
        for i in MENDELIAN_HPO_IDX..n {
            let hdup = HpoTermDuplet::new(&matrix[0][i], &matrix[1][i]);
            hpo_duplet_list.push(hdup);
        }
        Self::check_separator(matrix)?;
        let hpo_util = HpoUtil::new(hpo.clone());
        let _ = hpo_util.check_hpo_duplets(&hpo_duplet_list)?;
        
        Ok(Self { 
            individual_header: iheader, 
            disease_header_list: vec![DiseaseHeader::new()], 
            gene_variant_header_list: vec![GeneVariantHeader::new()], 
            hpo_duplets: hpo_duplet_list,
            template_type: TemplateType::Mendelian
        })
    }


    fn check_separator(matrix: &Vec<Vec<String>>) -> std::result::Result<(), ValidationErrors> {
        let mut verror = ValidationErrors::new();
        let h1 = &matrix[0][16];
        let h2 = &matrix[1][16];
        if h1 != "HPO" {
            verror.push_str(format!("Row 0, column 16: Expected 'HPO' but got '{h1}'"));
        } else if h2 != "na" {
            verror.push_str(format!("Row 1, column 16: Expected 'na' but got '{h2}'"));
        } 
        verror.ok()
    }


    fn qc_matrix_dimensions(matrix: &Vec<Vec<String>>) -> std::result::Result<(), ValidationErrors> {
        let n_rows = matrix.len();
        let mut verr = ValidationErrors::new();
        if n_rows < 3 {
            verr.push_str(format!("Empty matrix - must have two header rows and at least one data row but had {}", n_rows));
        }
        let n_cols = matrix[0].len();
        if n_cols < MENDELIAN_HPO_IDX + 1 {
            verr.push_str(format!("Incomplete matrix with {} columns, but at least {} required.", n_cols, MENDELIAN_HPO_IDX+1));
        }
        for (i, row) in matrix.iter().enumerate() {
            let cols = row.len();
            if cols != n_cols {
                verr.push_str(format!("First row has {n_cols} columns but row {i} has {cols}"))
            }
        }
        verr.ok()
    }

       /// We use this function when we add new HPO terms to the cohort; since the previous HeaderRowDuplet does not
    /// have these terms, we take the existing constant fields and append the new HPO term duplets (Note: client
    /// code should have arranged the HPO term list previously). We will then use this to update the existing PpktRow objects
    pub fn update(&self, updated_hpo_duplets: &Vec<HpoTermDuplet>) -> std::result::Result<Self, ValidationErrors> {
        Ok(Self { 
            individual_header: self.individual_header.clone(), 
            disease_header_list: self.disease_header_list.clone(), 
            gene_variant_header_list: self.gene_variant_header_list.clone(), 
            hpo_duplets: updated_hpo_duplets.clone(),
            template_type: self.template_type.clone()
        })
    }

    pub fn hpo_count(&self) -> usize {
        self.hpo_duplets.len()
    }

    pub fn template_type(&self) -> &TemplateType {
        &self.template_type
    }

    pub fn get_hpo_term_dto_list(&self, values: &Vec<String>) 
    -> std::result::Result<Vec<HpoTermDto>, String> {
        let mut hpo_dto_list = Vec::new();
        if self.hpo_count() != values.len() {
            return Err(format!("Expects {} HPO columns but got {}",
                self.hpo_count(), values.len()));
        }
        for (i, cell_contents) in values.iter().enumerate() {
            let hpo_label = self.hpo_duplets[i].row1();
            let hpo_id = self.hpo_duplets[i].row2();
            let dto = HpoTermDto::new(hpo_id, hpo_label, cell_contents);
            hpo_dto_list.push(dto);
        }
        Ok(hpo_dto_list)
    }

    pub fn mendelian_from_dto(dto_list: Vec<HeaderDupletDto>) -> Self {
        let hpo_termduplet_list: Vec<HpoTermDuplet> = dto_list
            .into_iter()
            .map(|dto| dto.to_hpo_duplet())
            .collect();
        Self { 
            individual_header: IndividualHeader::new(), 
            disease_header_list: vec![DiseaseHeader::new()], 
            gene_variant_header_list: vec![GeneVariantHeader::new()], 
            hpo_duplets: hpo_termduplet_list, 
            template_type: TemplateType::Mendelian
        }
    }



    /// Total number of columns in the template, including separator column
    pub fn n_columns(&self) -> usize {
        4 + 2*self.disease_header_list.len() + 6*self.gene_variant_header_list.len() + 4 + self.hpo_duplets.len() +1
    }


    /// We use this function when we add new HPO terms to the cohort; since the previous HeaderRowDuplet does not
    /// have these terms, we take the existing constant fields and append the new HPO term duplets (Note: client
    /// code should have arranged the HPO term list previously). We will then use this to update the existing PpktRow objects
    pub fn update_old(&self, term_list: &Vec<SimpleTerm>) -> Self {
        let updated_hpo_duplets: Vec<HpoTermDuplet> = term_list
            .iter()
            .map(|term| HpoTermDuplet::new(term.name(), &term.identifier().to_string()))
            .collect();
        Self {
            individual_header: self.individual_header.clone(),
            disease_header_list: self.disease_header_list.clone(),
            gene_variant_header_list: self.gene_variant_header_list.clone(),
            hpo_duplets: updated_hpo_duplets,
            template_type: self.template_type,
        }       
    }
    
    pub fn get_hpo_id_list(&self) -> std::result::Result<Vec<TermId>, String> {
        self.hpo_duplets
            .iter()
            .map(|duplet| {
                TermId::from_str(&duplet.row2())
                    .map_err(|_| format!("Could not get HPO TermId from: {}", duplet.row2()))
            })
            .collect()
    }

    pub fn get_hpo_duplets(&self) -> Vec<HpoTermDuplet> {
        self.hpo_duplets.clone()
    }

    pub fn get_hpo_header_dtos(&self) -> Vec<HeaderDupletDto> {
        self.hpo_duplets.iter()
            .map(|hpo_duplet| hpo_duplet.to_header_dto())
            .collect()
    }

    pub fn get_hpo_content_dtos(
        &self,
        cell_content_list: &Vec<String>)
    -> std::result::Result<Vec<HpoTermDto>, String> {
        if cell_content_list.len() != self.hpo_count() {
            return Err(format!("Header has {} HPO columns but cell_content_list has {}.",
            self.hpo_count(), cell_content_list.len()));
        }
        let dto_list: Vec<HpoTermDto> = self.get_hpo_duplets()
            .iter()
            .zip(cell_content_list.iter())
            .map(|(duplet, content)| {
                HpoTermDto::new(duplet.hpo_id(), duplet.hpo_label(), content)
            })
            .collect();
        Ok(dto_list)
    }

    pub fn get_hpo_content_map(
        &self,
        cell_content_list: &[String])
    -> std::result::Result<HashMap<TermId, String>, String> {
        if cell_content_list.len() != self.hpo_count() {
            return Err(format!("Header has {} HPO columns but cell_content_list has {}.",
            self.hpo_count(), cell_content_list.len()));
        }
        self.get_hpo_duplets()
            .iter()
            .zip(cell_content_list.iter())
            .map(|(duplet, content)| {
                duplet.to_term_id().map(|tid| (tid, content.clone()))
            })
            .collect()
    }

    pub fn n_mendelian_contant_fields() -> usize {
        N_CONSTANT_FIELDS_MENDELIAN
    }

  

}



#[cfg(test)]
mod test {
    use super::*;
    use crate::{error::Error, header::{hpo_term_duplet::HpoTermDuplet}};
    use ontolius::{io::OntologyLoaderBuilder, ontology::csr::MinimalCsrOntology, term::simple::SimpleMinimalTerm};
    use rstest::{fixture, rstest};

    #[fixture]
    pub fn one_case_matrix() -> Vec<Vec<String>> {
        let row1: Vec<String> = vec![ 
            "PMID", "title", "individual_id", "comment", "disease_id", "disease_label", "HGNC_id", "gene_symbol", "transcript", "allele_1", "allele_2", "variant.comment", "age_of_onset", "age_at_last_encounter", "deceased", "sex", "HPO", "Failure to thrive", "Seizure"
        ].into_iter().map(|s| s.to_owned()).collect();
        let row2: Vec<String> = vec![
            "CURIE", "str", "str", "optional", "CURIE", "str", "CURIE", "str", "str", "str", "str", "optional", "age", "age", "yes/no/na", "M:F:O:U", "na", "HP:0001508",  "HP:0001250" 
        ].into_iter().map(|s| s.to_owned()).collect();
        let row3: Vec<String> = vec![
            "PMID:29198722", "A Recurrent De Novo Nonsense Variant in ZSWIM6 Results in Severe Intellectual Disability without Frontonasal or Limb Malformations", "p.Arg913Ter Affected Individual 1", "", "OMIM:617865", "Neurodevelopmental disorder with movement abnormalities, abnormal gait, and autistic features", "HGNC:29316", "ZSWIM6", "NM_020928.2", "c.2737C>T", "na", "", "Infantile onset", "P16Y", "na", "M", "na", "observed", "observed"
        ].into_iter().map(|s| s.to_owned()).collect();
        vec![row1, row2, row3]
    }

    #[rstest]
    fn test_n_fields() {
        /// We expect a total of 17 fields before the HPO Term fields start
        assert_eq!(17, HeaderDupletRow::n_mendelian_contant_fields())
    }


   


}