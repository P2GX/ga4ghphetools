//! PpktExporter -- One row together with Header information, all that is needed to export a GA4GH phenopacket
//!
//! Each table cell is modelled as having the ability to return a datatype and the contents as a String
//! If a PpktExporter instance has no error, then we are ready to create a phenopacket.

use std::collections::HashMap;
use std::fmt::{self};
use std::ops::Deref;
use std::sync::Arc;
use std::time::Instant;

use ontolius::ontology::csr::FullCsrOntology;
use ontolius::term::simple::SimpleTerm;
use ontolius::TermId;
use polars::prelude::default_arrays;

use crate::dto::case_dto::CaseDto;
use crate::dto::hpo_term_dto::HpoTermDto;
use crate::dto::template_dto::{CellDto, DiseaseDto, GeneVariantBundleDto, IndividualBundleDto, RowDto};
use crate::dto::validation_errors::ValidationErrors;
use crate::hpo::age_util::{self, check_hpo_table_cell};
use crate::template::curie::Curie;
use crate::error::{self, Error, Result};
use crate::phetools_traits::TableCell;
use crate::template::disease_bundle::DiseaseBundle;
use crate::template::gene_variant_bundle::{self, GeneVariantBundle};
use crate::template::individual_bundle::IndividualBundle;
use crate::template::simple_label::SimpleLabel;
use crate::template::disease_gene_bundle::DiseaseGeneBundle;
use crate::template::header_duplet_row::{self, HeaderDupletRow};


impl Error {
    fn unrecognized_value(val: &str, field_name: &str) -> Self {
        Error::UnrecognizedValue {
            value: val.to_string(),
            column_name: field_name.to_string(),
        }
    }

    fn malformed_title(title: &str) -> Self {
        Error::TemplateError { msg: format!("Malformed template header '{}'", title) }
    }

    fn no_content(i: usize) -> Self {
        Error::TemplateError { msg: format!("No content and index '{i}'") }
    }
}



#[derive(Clone, Debug)]
pub struct PpktRow {
    header: Arc<HeaderDupletRow>,
    individual_bundle: IndividualBundle,
    disease_bundle_list: Vec<DiseaseBundle>,
    gene_var_bundle_list: Vec<GeneVariantBundle>,
    hpo_content: Vec<String>
}



impl PpktRow {
    pub fn from_row(
        header: Arc<HeaderDupletRow>,
        content: Vec<String>,
    ) -> std::result::Result<Self, ValidationErrors> {
        match header.template_type() {
            crate::template::pt_template::TemplateType::Mendelian => Self::from_mendelian_row(header, content),
            crate::template::pt_template::TemplateType::Melded => todo!(),
        }
    }

    pub fn from_mendelian_row(
        header: Arc<HeaderDupletRow>,
        content: Vec<String>
    ) -> std::result::Result<Self, ValidationErrors> {
        let DEMOGRAPHIC_IDX:usize = 12;
        let ibundle = IndividualBundle::from_row(&content, DEMOGRAPHIC_IDX)?;
        let disease_bundle = DiseaseBundle::from_row(&content, 4)?; // todo -- put index contents in same place
        let gene_variant_bundle = GeneVariantBundle::from_row(&content, 6)?;
        let mut verrs = ValidationErrors::new();
        let mut hpo_content: Vec<String> = Vec::new();
        for item in content.iter().skip(17) {
            verrs.push_result(age_util::check_hpo_table_cell(&item));
            hpo_content.push(item.clone());
        }
        if verrs.has_error() {
            return Err(verrs);
        }
        Ok(Self { header: header.clone(), 
            individual_bundle: ibundle, 
            disease_bundle_list: vec![disease_bundle], 
            gene_var_bundle_list: vec![gene_variant_bundle],
            hpo_content 
        })
    }


    pub fn from_dto(dto: RowDto, header: Arc<HeaderDupletRow>) -> Self {
        let hpo_content = dto.hpo_data.into_iter()
            .map(|c|c.value)
            .collect();
        Self { 
            header, 
            individual_bundle: IndividualBundle::from_dto(dto.individual_dto), 
            disease_bundle_list: DiseaseBundle::from_dto_list(dto.disease_dto_list), 
            gene_var_bundle_list: GeneVariantBundle::from_dto_list(dto.gene_var_dto_list), 
            hpo_content
        }
    }

    pub fn get_individual_dto(&self) -> IndividualBundleDto {
        let ibdl = &self.individual_bundle;
        IndividualBundleDto::new(ibdl.pmid(), ibdl.title(), ibdl.individual_id(), ibdl.comment(),
            ibdl.age_of_onset(), ibdl.age_at_last_encounter(), ibdl.deceased(), ibdl.sex())
    }

    pub fn get_disease_dto_list(&self) -> Vec<DiseaseDto> {
        let mut dto_list: Vec<DiseaseDto> = Vec::new();
        for disease in &self.disease_bundle_list {
            dto_list.push(disease.to_dto())
        }
        dto_list
    }

    pub fn get_gene_var_dto_list(&self) -> Vec<GeneVariantBundleDto> {
        let mut gbdto_list: Vec<GeneVariantBundleDto> = Vec::new();
        for gvb in &self.gene_var_bundle_list{
            gbdto_list.push(gvb.to_dto());
        }
        gbdto_list
    }

    pub fn get_hpo_value_list(&self) -> Vec<CellDto> {
        let mut cell_dto_list: Vec<CellDto> = Vec::new();
        for hpo_val in &self.hpo_content {
            cell_dto_list.push(CellDto::new(hpo_val));
        }
        cell_dto_list
    }

    pub fn get_hpo_term_dto_list(&self) -> std::result::Result<Vec<HpoTermDto>, String> {
        self.header.get_hpo_term_dto_list(&self.hpo_content).map_err(|e| e.to_string())
    }


    pub fn mendelian_from(
        header_duplet_row: Arc<HeaderDupletRow>,
        case: CaseDto, 
        dgb: DiseaseGeneBundle, 
        hpo_values: Vec<String>, ) -> Result<Self> 
    {
        let mut values: Vec<String> = case.individual_values();
        values.extend(dgb.values());
        values.extend(case.variant_values());
        values.push("na".to_string()); // separator
        values.extend(hpo_values);
       /*  Ok(Self {
            header_duplet_row: header_duplet_row,
            content: values
        })*/
        Err(Error::custom("mendelian_from-refacot"))
    }

    /// This function checks the current PpktRow for syntactical errors
    pub fn check_for_errors(&self) -> std::result::Result<(), ValidationErrors> {
        let mut verrs = ValidationErrors::new();
        verrs.push_verr_result(self.individual_bundle.do_qc());
        for db in &self.disease_bundle_list {
            verrs.push_verr_result(db.do_qc());
        }
        for gvb in &self.gene_var_bundle_list {
            verrs.push_verr_result(gvb.do_qc());
        }
        for item in &self.hpo_content {
            verrs.push_result(check_hpo_table_cell(item));
        }

        verrs.ok()
    }
}

/*
    
    fn get_item(&self, title: &str) -> Result<String> {
        self.header_duplet_row
        .get_idx(title)
        .and_then(|i| {
            self.content
                .get(i)
                .cloned()
                .ok_or_else(|| Error::no_content(i))
        })
    }

   

    pub fn get_value_at(&self, i: usize) -> Result<String> {
            if i >= self.content.len() {
                Err(Error::TemplateError { msg: format!("Invalid index {i}") })
            } else {
                Ok(self.content[i].clone())
            }
    } 

    /// Return the data transfer object for displaying information about the individual (id, PMID, title, comment) in a GUI
    pub fn get_individual_dto(&self) -> Result<IndividualBundleDto> {
        Ok(IndividualBundleDto::new(&self.pmid()?, &self.title()?, &self.individual_id()?, &self.get_comment()?))
    }

    /// Return the data (outside of IndividualDto) as a vector of CellDtos-
    /// TODO -- we will want to update this as we create more DTOs
    pub fn get_cell_dtos(&self) -> Result<Vec<CellDto>> {
        let mut dtos: Vec<CellDto> = Vec::new();
        for val in self.content.iter().skip(4) {
            dtos.push(CellDto::new(val));
        }
        Ok(dtos)
    }

    pub fn get_row_dto(&self) -> Result<RowDto> {
        let individual_dto = self.get_individual_dto()?;
        let cell_dto_list = self.get_cell_dtos()?;
        

        //Ok(RowDto::new(individual_dto, cell_dto_list))
        Err(Error::TemplateError { msg: format!("refactoring error") })
    }
*/

    /// the tid_map has TermId to label
    /*
    pub fn update(
        &self, 
        tid_map: &mut HashMap<TermId, String>, 
        updated_hdr: Arc<HeaderDupletRowOLD>) 
    -> Result<Self> {
        // update the tid map with the existing  values
        let previous_hpo_id_list = self.header_duplet_row.get_hpo_id_list()?;
        let offset = self.header_duplet_row.get_hpo_offset();
        for (i, term_id) in previous_hpo_id_list.iter().enumerate() {
            let j = i + offset;
            match self.content.get(j) {
                Some(value) => {
                    tid_map.insert(term_id.clone(), value.clone());
                },
                None => {
                    return Err(Error::TemplateError { msg: format!("Could not retrieve value in update i={i}, j={j}") } );
                }
            }
        }
        let updated_hpo_id_list = updated_hdr.get_hpo_id_list()?;


        let updated_hpo: Result<Vec<String>> = updated_hpo_id_list
                .into_iter()
                .map(|term_id| {
                    tid_map.get(&term_id)
                        .cloned()
                        .ok_or_else(|| Error::TemplateError {
                            msg: format!("Could not retrieve updated value for '{}'", &term_id)
                        })
                })
                .collect();
        let updated_hpo = updated_hpo?;
        let n_const = self.n_constant_columns();
        let mut updated_content: Vec<String> = self.content.iter().take(n_const).cloned().collect();
        updated_content.extend(updated_hpo);
        Ok(Self {
            header_duplet_row: updated_hdr,
            content: updated_content,
        })
    }


    pub fn get_items(&self, indices: &Vec<usize>) -> Result<Vec<String>> {
        match indices.iter().copied().max() {
            Some(max_i) => {
                if max_i > self.content.len() {
                    return Err(Error::TemplateError { msg: format!("Index {max_i} out of bounds") });
                }
                let selected: Vec<String> = indices
                    .iter()
                    .filter_map(|&idx| self.content.get(idx).cloned())
                    .collect();
                Ok(selected)
            },
            None => {
                return Err(Error::TemplateError { msg: format!("Could not extract from from indices") });
            }
        }
    } 
    pub fn remove_whitespace(&mut self, col: usize) -> Result<()> {
        if col > self.content.len() {
            return Err(Error::TemplateError { msg: format!("row index error {col}") })
        }
        if let Some(s) = self.content.get_mut(col) {
            *s = s.chars().filter(|c| !c.is_whitespace()).collect();
        }
        Ok(())
    }

    pub fn trim(&mut self, col: usize) {
        if let Some(s) = self.content.get_mut(col) {
            *s = s.trim().to_string();
        }
    }

    
    /// Set the indicated cell to value or return an Error if the value is not valid
    pub fn set_value(&mut self, col: usize, value: &str) -> Result<()> {
        let duplet = self.header_duplet_row.get_duplet_at_index(col)?;
        duplet.qc_cell(value)?;
        if let Some(s) = self.content.get_mut(col) {
            *s = value.to_string();
        }
        Ok(())
    }



    

    pub fn get_hpo_term_dto_list(&self) -> Result<Vec<HpoTermDto>> {
        let mut dto_list: Vec<HpoTermDto> = Vec::new();
        let hpo_duplets = self.header_duplet_row.get_hpo_duplets();
        let offset = self.header_duplet_row.get_hpo_offset();
        for j in 0..hpo_duplets.len() {
            let i = j + offset;
            if let Some(hpo_duplet ) = hpo_duplets.get(j) {
                if let Some(entry) = self.content.get(i) {
                    let dto = HpoTermDto::new(hpo_duplet.row2(), hpo_duplet.row1(), entry);
                    dto_list.push(dto);  
                } else {
                    return Err(Error::TemplateError { msg: format!("Could not extract entry from PpktRow") });
                }
            } else {
                return Err(Error::TemplateError { msg: format!("Could not HpoTermDuplet from PpktRow") });
            }
        }
        Ok(dto_list)
    }
   
}
*/


#[cfg(test)]
mod test {
    use crate::{error::Error, header::{hpo_term_duplet::HpoTermDuplet}, hpo::hpo_util::{self, HpoUtil}};
    use lazy_static::lazy_static;
    use ontolius::{io::OntologyLoaderBuilder, ontology::{csr::MinimalCsrOntology, OntologyTerms}, term};
    use polars::io::SerReader;
    use super::*;
    use std::{fs::File, io::BufReader, str::FromStr};
    use rstest::{fixture, rstest};
    use flate2::bufread::GzDecoder;

    #[fixture]
    fn hpo() -> Arc<FullCsrOntology> {
        let path = "resources/hp.v2025-03-03.json.gz";
        let reader = GzDecoder::new(BufReader::new(File::open(path).unwrap()));
        let loader = OntologyLoaderBuilder::new().obographs_parser().build();
        let hpo = loader.load_from_read(reader).unwrap();
        Arc::new(hpo)
    }



    #[fixture]
    fn row1() -> Vec<String> 
    {
        let row: Vec<&str> = vec![
            "PMID", "title", "individual_id", "comment", "disease_id", "disease_label", "HGNC_id",	"gene_symbol", 
            "transcript", "allele_1", "allele_2", "variant.comment", "age_of_onset", "age_at_last_encounter", 
            "deceased", "sex", "HPO",	"Clinodactyly of the 5th finger", "Hallux valgus",	"Short 1st metacarpal", 
            "Ectopic ossification in muscle tissue", "Long hallux", "Pain", "Short thumb"
        ];
        row.into_iter().map(|s| s.to_owned()).collect()
    }

    #[fixture]
    fn row2() -> Vec<String> 
    {
        let row: Vec<&str> = vec![
            "CURIE", "str", "str", "optional", "CURIE", "str", "CURIE", "str", "str", "str", "str", "optional", "age", "age", "yes/no/na", "M:F:O:U", "na",
            "HP:0004209", "HP:0001822", "HP:0010034", "HP:0011987", "HP:0001847", "HP:0012531", "HP:0009778"];
        row.into_iter().map(|s| s.to_owned()).collect()
    }

    #[fixture]
    fn row3() -> Vec<String> {
        let row: Vec<&str> =  vec![
            "PMID:29482508", "Difficult diagnosis and genetic analysis of fibrodysplasia ossificans progressiva: a case report", "current case", "", 
            "OMIM:135100", "Fibrodysplasia ossificans progressiva", "HGNC:171", "ACVR1", 
            "NM_001111067.4", "c.617G>A", "na", "NP_001104537.1:p.(Arg206His)", 
            "P9Y", "P16Y", "no", "M", "na", "na", "P16Y", "na", "P16Y", "P16Y", "P16Y", "na"];
        row.into_iter().map(|s| s.to_owned()).collect()
    }


    #[fixture]
    fn original_matrix(row1: Vec<String>, row2: Vec<String>, row3: Vec<String>)  -> Vec<Vec<String>> {
        let mut rows = Vec::with_capacity(3);
        rows.push(row1);
        rows.push(row2);
        rows.push(row3);
        rows
    }


    #[fixture]
    pub fn case_a_dto() -> CaseDto {
        CaseDto::new(
            "PMID:123", 
            "A Recurrent De Novo Nonsense Variant", 
            "Individual 7", 
            "",  // comment
            "c.2737C>T",  // allele_1
            "na", // allele_2
            "",  // variant.comment
            "Infantile onset", // age_at_onset
            "P32Y", //  age_at_last_encounter
            "na", // deceased
            "M" //sex
        )
    }

    #[fixture]
    pub fn hpo_dtos() -> Vec<HpoTermDto> {
        vec![HpoTermDto::new("HP:0001382", "Joint hypermobility", "observed"),
        HpoTermDto::new("HP:0000574", "Thick eyebrow", "observed") ]
    }


}

