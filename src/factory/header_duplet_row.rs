//! HeaderDupletRow: Encapsulate the headers (which we call duplets because each has two fields and which are serialized as the furst two rows of the template)
//! 
//! Each HeaderDuplet determines the meaning of the rows beneath it.
//! We pass a reference (via ARC) of the HeaderDupletRow to each of the rows of the template.
//! This only applies to the legacy Excel templates. Once we have updated Phenopacket Store to use the new JSON format, we will no longer need this struct.



use std::collections::HashMap;
use std::str::FromStr;
use ontolius::term::simple::SimpleTerm;
use ontolius::term::{MinimalTerm};
use ontolius::{Identified, TermId};

use crate::dto::hpo_term_dto::{CellValue, HpoTermData, HpoTermDuplet};
use crate::dto::cohort_dto::CohortType;
use crate::dto::validation_errors::ValidationErrors;
use crate::error::PheToolsError;
use crate::error::ontology_error::OntologyError;
use crate::header::disease_header::DiseaseHeader;
use crate::header::gene_variant_header::GeneVariantHeader;
use crate::header::individual_header::IndividualHeader;




/// Number of columns in the Individual section
const NUMBER_OF_INDIVIDUAL_FIELDS: usize = 4;
/// Number of columns in the Disease/Gene/Variant bundle section
const NUMBER_OF_DISEASE_GENE_BUNDLE_FIELDS: usize = 8;
/// Number of columns in the Demographic section
const NUMBER_OF_DEMOGRAPHIC_FIELDS: usize = 4;
/// Separator field (HPO/na)
const NUMBER_OF_SEPARATOR_FIELDS: usize = 1;




/// Total number of constant fields (columns) in the Mendelian template
const N_CONSTANT_FIELDS_MENDELIAN: usize = 
    NUMBER_OF_INDIVIDUAL_FIELDS + NUMBER_OF_DISEASE_GENE_BUNDLE_FIELDS + NUMBER_OF_DEMOGRAPHIC_FIELDS + NUMBER_OF_SEPARATOR_FIELDS;




#[derive(Clone, Debug)]
pub struct HeaderDupletRow {
    individual_header: IndividualHeader,
    disease_header_list: Vec<DiseaseHeader>,
    gene_variant_header_list: Vec<GeneVariantHeader>,
    hpo_duplets: Vec<HpoTermDuplet>,
    template_type: CohortType,
}


impl HeaderDupletRow {
   
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


    fn mendelian_from_hpo_duplets(hpo_duplets: Vec<HpoTermDuplet>) -> Self {
        Self { 
            individual_header: IndividualHeader::new(), 
            disease_header_list: vec![DiseaseHeader::new()], 
            gene_variant_header_list: vec![GeneVariantHeader::new()], 
            hpo_duplets, 
            template_type: CohortType::Mendelian 
        }
    }

    pub fn from_hpo_duplets(
        hpo_duplets: Vec<HpoTermDuplet>, 
        template_type: CohortType)
    -> Self {
            match template_type {
                CohortType::Mendelian => Self::mendelian_from_hpo_duplets(hpo_duplets),
                CohortType::Melded => todo!(),
                CohortType::Digenic => todo!()
            }
        }


    pub fn hpo_count(&self) -> usize {
        self.hpo_duplets.len()
    }

    pub fn template_type(&self) -> &CohortType {
        &self.template_type
    }

    pub fn get_hpo_term_dto_list(&self, values: &Vec<String>) 
    -> std::result::Result<Vec<HpoTermData>, PheToolsError> {
        let mut hpo_dto_list = Vec::new();
        if self.hpo_count() != values.len() {
            return Err(OntologyError::header_length_mismatch_err(self.hpo_count(), values.len()).into());
        }
        for (i, cell_contents) in values.iter().enumerate() {
            let dto = HpoTermData::new(self.hpo_duplets[i].clone(), CellValue::from_str(cell_contents)?)?;
            hpo_dto_list.push(dto);
        }
        Ok(hpo_dto_list)
    }

    pub fn new_mendelian_ppkt_from_dto(duplet_list: &[HpoTermDuplet]) -> Self {
        Self { 
            individual_header: IndividualHeader::new(), 
            disease_header_list: vec![DiseaseHeader::new()], 
            gene_variant_header_list: vec![GeneVariantHeader::new()], 
            hpo_duplets: duplet_list.to_vec(), 
            template_type: CohortType::Mendelian
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
    
    pub fn get_hpo_id_list(&self) -> std::result::Result<Vec<TermId>, OntologyError> {
        let mut term_id_list: Vec<TermId> = Vec::with_capacity(self.hpo_duplets.len());
        for duplet in &self.hpo_duplets {
            let tid = duplet.to_term_id()?;
            term_id_list.push(tid);
        }
        Ok(term_id_list)
    }

    pub fn get_hpo_duplets(&self) -> Vec<HpoTermDuplet> {
        self.hpo_duplets.clone()
    }

    pub fn hpo_duplets(&self) -> &[HpoTermDuplet] {
        self.hpo_duplets.as_ref()
    }

    pub fn get_hpo_header_dtos(&self) -> Vec<HpoTermDuplet> {
        self.hpo_duplets.clone()
    }

    pub fn get_hpo_content_dtos(
        &self,
        cell_content_list: &Vec<String>)
    -> std::result::Result<Vec<HpoTermData>, PheToolsError> {
        if cell_content_list.len() != self.hpo_count() {
            let msg = format!("Header has {} HPO columns but cell_content_list has {}.",
            self.hpo_count(), cell_content_list.len());
            return Err(OntologyError::manipulation_err(msg).into());
        }
        let mut dto_list: Vec<HpoTermData> = Vec::new();
        for (duplet, content) in self.get_hpo_duplets().iter().zip(cell_content_list.iter()) {
            let htd = HpoTermData::new(duplet.clone(),  CellValue::from_str(&content)?)?;
            dto_list.push(htd);
        }
        Ok(dto_list)
    }

    /// Get a map with tid (term ID of the HPO column in question) to value (contents of the cell, e.g. observed, P32Y)
    pub fn get_hpo_content_map(
        &self,
        cell_content_list: &[String])
    -> std::result::Result<HashMap<TermId, String>, OntologyError> {
        if cell_content_list.len() != self.hpo_count() {
            let msg = format!("Header has {} HPO columns but cell_content_list has {}.",
            self.hpo_count(), cell_content_list.len());
            return Err(OntologyError::manipulation_err(msg));
        }
        let mut dto_map: HashMap<TermId, String> = HashMap::new();
        for (duplet, content) in self.get_hpo_duplets().iter().zip(cell_content_list.iter()) {
            let tid = duplet.to_term_id()?;
            dto_map.insert(tid, content.to_string());
        };
        Ok(dto_map)
    }

    pub fn n_mendelian_contant_fields() -> usize {
        N_CONSTANT_FIELDS_MENDELIAN
    }

  

}



#[cfg(test)]
mod test {
    use super::*;
    use rstest::{fixture, rstest};

    #[fixture]
    fn hpo_duplets() -> Vec<HpoTermDuplet> {
        vec![
            HpoTermDuplet::new("Seizure", "HP:0001250"),
            HpoTermDuplet::new("Nystagmus", "HP:0000639"),
        ]
    }

    #[rstest]
    fn test_hpo_count(hpo_duplets: Vec<HpoTermDuplet>) {
        let header = HeaderDupletRow::from_hpo_duplets(hpo_duplets, CohortType::Mendelian);
        assert_eq!(header.hpo_count(), 2);
    }

    #[rstest]
    fn test_template_type(hpo_duplets: Vec<HpoTermDuplet>) {
        let header = HeaderDupletRow::from_hpo_duplets(hpo_duplets, CohortType::Mendelian);
        assert_eq!(*header.template_type(), CohortType::Mendelian);
    }

    #[rstest]
    fn test_n_columns(hpo_duplets: Vec<HpoTermDuplet>) {
        let header = HeaderDupletRow::from_hpo_duplets(hpo_duplets, CohortType::Mendelian);
        // 4 (individual) + 2*1 (disease) + 6*1 (gene/variant) + 4 (demographic) + 2 (hpo) + 1 (separator)
        assert_eq!(header.n_columns(), 19);
    }

    #[rstest]
    fn test_get_hpo_id_list(hpo_duplets: Vec<HpoTermDuplet>) {
        let header = HeaderDupletRow::from_hpo_duplets(hpo_duplets, CohortType::Mendelian);
        let ids = header.get_hpo_id_list().unwrap();
        assert_eq!(ids.len(), 2);
        assert_eq!(ids[0].to_string(), "HP:0001250");
        assert_eq!(ids[1].to_string(), "HP:0000639");
    }

    #[rstest]
    fn test_get_hpo_content_map(hpo_duplets: Vec<HpoTermDuplet>) {
        let header = HeaderDupletRow::from_hpo_duplets(hpo_duplets, CohortType::Mendelian);
        let values = vec!["observed".to_string(), "excluded".to_string()];
        let map = header.get_hpo_content_map(&values).unwrap();
        assert_eq!(map.len(), 2);
    }

    #[rstest]
    fn test_get_hpo_content_map_length_mismatch(hpo_duplets: Vec<HpoTermDuplet>) {
        let header = HeaderDupletRow::from_hpo_duplets(hpo_duplets, CohortType::Mendelian);
        let values = vec!["observed".to_string()]; // wrong length: 1 vs 2
        assert!(header.get_hpo_content_map(&values).is_err());
    }

    #[rstest]
    fn test_get_hpo_term_dto_list_length_mismatch(hpo_duplets: Vec<HpoTermDuplet>) {
        let header = HeaderDupletRow::from_hpo_duplets(hpo_duplets, CohortType::Mendelian);
        let values = vec!["observed".to_string()]; // wrong length
        assert!(header.get_hpo_term_dto_list(&values).is_err());
    }

    #[rstest]
    fn test_update_replaces_hpo_duplets(hpo_duplets: Vec<HpoTermDuplet>) {
        let header = HeaderDupletRow::from_hpo_duplets(hpo_duplets, CohortType::Mendelian);
        let new_duplets = vec![HpoTermDuplet::new("Ataxia", "HP:0001251")];
        let updated = header.update(&new_duplets).unwrap();
        assert_eq!(updated.hpo_count(), 1);
        assert_eq!(updated.get_hpo_id_list().unwrap()[0].to_string(), "HP:0001251");
    }

}