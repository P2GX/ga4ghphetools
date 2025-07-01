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
use crate::hpo::hpo_util;
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
        println!("from_mendelian_row");
        let DEMOGRAPHIC_IDX:usize = 12;
        let ibundle = IndividualBundle::from_row(&content, DEMOGRAPHIC_IDX)?;
        let disease_bundle = DiseaseBundle::from_row(&content, 4)?; // todo -- put index contents in same place
        let gene_variant_bundle = GeneVariantBundle::from_row(&content, 6)?;
        let mut verrs = ValidationErrors::new();
        let mut hpo_content: Vec<String> = Vec::new();
        for item in content.iter().skip(17) {
            let cell = if item.trim().is_empty() { "na" } else { item }; // TODO -- remove once old templates have been restructured
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


    pub fn mendelian_from_dto(
        &self,
        header_duplet_row: Arc<HeaderDupletRow>,
        individual_dto: IndividualBundleDto,
        annotations: Vec<HpoTermDto>,
        existing_annotation_map:HashMap<TermId, String>) 
    -> std::result::Result<Self, ValidationErrors> 
    {
        let verrs = ValidationErrors::new();
        //let existing_header = self.header:
        
       /*  Ok(Self {
            header_duplet_row: header_duplet_row,
            content: values
        })*/
        if verrs.has_error() {
            Err(verrs)
        } else {
            Ok(Self{ 
                header: header_duplet_row, 
                individual_bundle: todo!(), 
                disease_bundle_list: todo!(), 
                gene_var_bundle_list: todo!(), 
                hpo_content: todo!() })
        }
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


    pub fn update_header(
        &self, 
        updated_hdr: Arc<HeaderDupletRow>
    ) -> std::result::Result<Self, ValidationErrors> {
        let mut verrs = ValidationErrors::new();
        let updated_hpo_id_list = updated_hdr.get_hpo_id_list()?;
        let previous_header = &self.header;
        let hpo_map = previous_header.get_hpo_content_map(&self.hpo_content);
        let hpo_map = hpo_map.map_err(|e|{
            verrs.push_str(e);
            verrs // only propagated if error occurs
        })?;
        let mut content = Vec::new();
        for tid in updated_hpo_id_list {
            let item: String = hpo_map
                .get(&tid)
                .cloned() // converts Option<&String> to Option<String>
                .unwrap_or_else(|| "na".to_string()); // this will pertain to the new HPO term we are adding
            content.push(item);
        }

        Ok(Self { 
            header: updated_hdr, 
            individual_bundle: self.individual_bundle.clone(), 
            disease_bundle_list: self.disease_bundle_list.clone(), 
            gene_var_bundle_list: self.gene_var_bundle_list.clone(), 
            hpo_content: content 
        })
    }

    pub fn update(
        &self, 
        tid_map: &mut HashMap<TermId, String>, 
        updated_hdr: Arc<HeaderDupletRow>) 
    -> std::result::Result<Self, ValidationErrors> {
        // update the tid map with the existing  values
        let mut verr = ValidationErrors::new();
        let previous_hpo_id_list = self.header.get_hpo_id_list()?;
        let hpo_cell_content_list = self.hpo_content.clone();
        if previous_hpo_id_list.len() != hpo_cell_content_list.len() {
            verr.push_str("Mismatched lengths between HPO ID list and HPO content");
            return Err(verr); // not recoverable
        }
        for (hpo_id, cell_content) in previous_hpo_id_list.iter().zip(hpo_cell_content_list.iter()) {
            tid_map.insert(hpo_id.clone(), cell_content.clone());
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
        let updated_hpo = updated_hpo.map_err(|e| {
                verr.push_str(&e.to_string());
                verr
            })?;
        Ok(Self {
            header: updated_hdr,
            individual_bundle: self.individual_bundle.clone(),
            disease_bundle_list: self.disease_bundle_list.clone(),
            gene_var_bundle_list: self.gene_var_bundle_list.clone(),
            hpo_content: updated_hpo,
        })
    }


}



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

