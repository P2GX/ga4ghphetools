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
use crate::dto::template_dto::{CellDto, IndividualDto, RowDto};
use crate::header::header_duplet::{HeaderDuplet, HeaderDupletItem};
use crate::template::curie::Curie;
use crate::error::{self, Error, Result};
use crate::phetools_traits::TableCell;
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

#[derive(Clone)]
pub struct PpktRow {
    /// Reference to the header, which is the same for all rows in the template
    header_duplet_row: Arc<HeaderDupletRow>,
    /// Contents of the row, represented as Strings.
    content: Vec<String>,
}

impl PpktRow {
    pub fn new(
        header_duplet_row: Arc<HeaderDupletRow>,
        content: Vec<String>,
    ) -> Self {
        Self {
            header_duplet_row,
            content: content,

        }
    }

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

    pub fn individual_id(&self) -> Result<String> {
        self.get_item("individual_id")
    }

    pub fn pmid(&self) -> Result<String> {
        self.get_item("PMID")
    }

    pub fn title(&self) -> Result<String> {
        self.get_item("title")
    }

    pub fn get_comment(&self) -> Result<String> {
        self.get_item("comment")
    }

    pub fn disease_id(&self) -> Result<String> {
        self.get_item("disease_id")
    }

    pub fn disease_label(&self) -> Result<String> {
        self.get_item("disease_label")
    }

    pub fn hgnc_id(&self) -> Result<String> {
        self.get_item("hgnc_id")
    }

    pub fn gene_symbol(&self) -> Result<String> {
        self.get_item("gene_symbol")
    }

    pub fn transcript_id(&self) -> Result<String> {
        self.get_item("transcript_id")
    }

    pub fn allele_1(&self) -> Result<String> {
        self.get_item("allele_1")
    }

    pub fn allele_2(&self) -> Result<String> {
        self.get_item("allele_2")
    }

    pub fn age_of_onset(&self) -> Result<String> {
        self.get_item("age_of_onset")
    }

    pub fn age_at_last_encounter(&self) -> Result<String> {
        self.get_item("age_at_last_encounter")
    }

    pub fn deceased(&self) -> Result<String> {
        self.get_item("deceased")
    }

    pub fn sex(&self) -> Result<String> {
        self.get_item("sex")
    }

    /// Return the first error or OK
    pub fn qc_check(&self) -> Result<()> {
        let ncols = self.content.len();
        for i in 0..ncols {
            let cell_contents = self.content[i].as_str();
            self.header_duplet_row.qc_check(i, cell_contents)?;
        }

        Ok(())
    }

    pub fn n_constant_columns(&self) -> usize {
        self.header_duplet_row.n_constant()
    }

    /// Get a (potentially empty) list of Errors for this template
    pub fn get_errors(&self) -> Vec<Error> {
        self.content
            .iter()
            .enumerate()
            .filter_map(|(i, cell)| self.header_duplet_row.qc_check(i, cell).err())
            .collect()
    }

    pub fn get_string_row(&self) -> Vec<String> {
            self.content.clone()
    }

    pub fn get_value_at(&self, i: usize) -> Result<String> {
            if i >= self.content.len() {
                Err(Error::TemplateError { msg: format!("Invalid index {i}") })
            } else {
                Ok(self.content[i].clone())
            }
    }

    /// Return the data transfer object for displaying information about the individual (id, PMID, title, comment) in a GUI
    pub fn get_individual_dto(&self) -> Result<IndividualDto> {
        Ok(IndividualDto::new(self.pmid()?, self.title()?, self.individual_id()?, self.get_comment()?))
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

        Ok(RowDto::new(individual_dto, cell_dto_list))
    }


    /// the tid_map has TermId to label
    pub fn update(
        &self, 
        tid_map: &mut HashMap<TermId, String>, 
        updated_hdr: Arc<HeaderDupletRow>) 
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
        Ok(Self {
            header_duplet_row: header_duplet_row,
            content: values
        })
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



#[cfg(test)]
mod test {
    use crate::{error::Error, header::{header_duplet::HeaderDupletItem, hpo_term_duplet::HpoTermDuplet}, hpo::hpo_util::{self, HpoUtil}};
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

    #[rstest]
    fn test_update_ppkt_row(
        mut original_matrix: Vec<Vec<String>>, 
        hpo: Arc<FullCsrOntology>,
        case_a_dto: CaseDto,
        hpo_dtos: Vec<HpoTermDto>
    ) -> Result<()> {
        let hpo_arc = hpo.clone();
        let hpo_util = HpoUtil::new(hpo_arc);
        hpo_util.check_hpo_dto(&hpo_dtos)?;
        let hdup_list = match HeaderDuplet::extract_from_string_matrix(&original_matrix) {
            Ok(val) => val,
            Err(e) => {
                return Err(e);
            }
        };
        let content = &original_matrix[2].clone();
        
        let header_duplet_row = HeaderDupletRow::mendelian_from_duplets(hdup_list).unwrap();
        let hdr_arc = Arc::new(header_duplet_row);
        let hdr_arc2 = hdr_arc.clone();
        let ppkt_row = PpktRow::new(hdr_arc, content.to_vec());
        
        assert_eq!(ppkt_row.pmid()?, "PMID:29482508");
        let hpo_util = HpoUtil::new(hpo.clone());
        let mut simple_terms = hpo_util.simple_terms_from_dto(&hpo_dtos)?;
        let mut hpo_term_id_to_label_map = hpo_util.term_label_map_from_dto_list(&hpo_dtos)?;
        assert_eq!(2, simple_terms.len());
        let updated_hdr = hdr_arc2.update_old(&simple_terms);
        let updated_arc = Arc::new(updated_hdr);
        let updated_ppkt = ppkt_row.update(&mut hpo_term_id_to_label_map, updated_arc);

        Ok(())
    }



}

