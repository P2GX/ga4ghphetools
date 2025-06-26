//! HeaderDupletRow: Encapsulate the headers (which we call duplets because each has two fields and which are serialized as the furst two rows of the template)
//! 
//! Each HeaderDuplet determines the meaning of the rows beneath it.
//! We pass a reference (via ARC) of the HeaderDupletRow to each of the rows of the template



use std::collections::HashMap;
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
use crate::dto::template_dto::{HeaderDto, HeaderDupletDto};
use crate::dto::validation_errors::ValidationErrors;
use crate::header::demographic_header::DemographicHeader;
use crate::header::disease_header::DiseaseHeader;
use crate::header::duplet_item::DupletItem;
use crate::header::gene_variant_header::GeneVariantHeader;
use crate::header::header_duplet::{HeaderDuplet, HeaderDupletItem, HeaderDupletItemFactory};
use crate::header::hpo_separator_duplet::HpoSeparatorDuplet;
use crate::header::hpo_term_duplet::HpoTermDuplet;
use crate::header::individual_header::IndividualHeader;
use crate::header::{individual_id_duplet, variant_comment_duplet};
use crate::header::{age_last_encounter_duplet::AgeLastEncounterDuplet, age_of_onset_duplet::AgeOfOnsetDuplet, allele_1_duplet::Allele1Duplet, allele_2_duplet::Allele2Duplet, comment_duplet::CommentDuplet, deceased_duplet::DeceasedDuplet, disease_id_duplet::DiseaseIdDuplet, disease_label_duplet::DiseaseLabelDuplet, gene_symbol_duplet::GeneSymbolDuplet, hgnc_duplet::HgncDuplet, individual_id_duplet::IndividualIdDuplet, pmid_duplet::PmidDuplet, sex_duplet::SexDuplet, title_duplet::TitleDuplet, transcript_duplet::TranscriptDuplet, variant_comment_duplet::VariantCommentDuplet};
use crate::error::{self, Error, Result};
use crate::hpo::hpo_util::HpoUtil;
use crate::template::disease_bundle::DiseaseBundle;
use crate::template::individual_bundle::IndividualBundle;
use crate::template::pt_template::TemplateType;

use super::header_index::HeaderIndexer;

const NOT_AVAILABLE: &str = "na";
const EXCLUDED: &str = "excluded";
const OBSERVED: &str = "observed";

/// Create macros to get the specific duplet object from the Enum.
macro_rules! impl_duplet_getters {
    ($( $variant:ident => $method:ident : $ty:ty ),*) => {
        impl HeaderDuplet {
            $(
            pub fn $method(&self) -> std::result::Result<&$ty, crate::error::Error> {
                match self {
                    HeaderDuplet::$variant(inner) => Ok(inner),
                    _ => Err(crate::error::Error::TemplateError {
                        msg: format!("Expected {}", stringify!($variant)),
                    }),
                }
            }
            )*
        }
    };
}




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

/// The header duplet has the following sections.
/// Note that Mendelian does not have a DiseaseGeneBundleMelded section
/// which in essence represents the second gene of a pair
/// We know that everything not in this list is an HPO Term Column
#[derive(Clone, Debug)]
enum SectionType {
    Individual,
    DiseaseGeneBundleMendelian,
    DiseaseGeneBundleMelded,
    Demographic,
    Separator,
}

impl SectionType {

    pub fn n_elements(section_type: SectionType) -> usize {
        match section_type {
            SectionType::Individual => 4,
            SectionType::DiseaseGeneBundleMendelian => 7,
            SectionType::DiseaseGeneBundleMelded => 7,
            SectionType::Demographic => 4,
            SectionType::Separator => 1,
        }
    }

    pub fn mendelian() -> Vec<SectionType> {
        let mut stlist: Vec<SectionType> = Vec::new();
        stlist.extend(std::iter::repeat(SectionType::Individual.clone()).take(SectionType::n_elements(SectionType::Individual)));
        stlist.extend(std::iter::repeat(SectionType::DiseaseGeneBundleMendelian.clone()).take(SectionType::n_elements(SectionType::DiseaseGeneBundleMendelian)));
        stlist.extend(std::iter::repeat(SectionType::Demographic.clone()).take(SectionType::n_elements(SectionType::Demographic)));
        stlist.push(SectionType::Separator);
        stlist
    }

}


/// Total number of constant fields (columns) in the Mendelian template
const N_CONSTANT_FIELDS_MENDELIAN: usize = 
    NUMBER_OF_INDIVIDUAL_FIELDS + NUMBER_OF_DISEASE_GENE_BUNDLE_FIELDS + NUMBER_OF_DEMOGRAPHIC_FIELDS + NUMBER_OF_SEPARATOR_FIELDS;

impl_duplet_getters!(
    PmidDuplet => as_pmid_duplet: PmidDuplet,
    TitleDuplet => as_title_duplet: TitleDuplet,
    IndividualIdDuplet => as_individual_id_duplet: IndividualIdDuplet,
    CommentDuplet => as_comment_duplet: CommentDuplet,
    DiseaseIdDuplet => as_disease_id: DiseaseIdDuplet,
    DiseaseLabelDuplet => as_disease_label: DiseaseLabelDuplet,
    HgncDuplet => as_hgnc_duplet: HgncDuplet,
    GeneSymbolDuplet => as_gene_symbol_duplet: GeneSymbolDuplet,
    TranscriptDuplet => as_transcript_duplet: TranscriptDuplet,
    Allele1Duplet => as_allele_1_duplet: Allele1Duplet,
    Allele2Duplet => as_allele_2_duplet: Allele2Duplet,
    VariantCommentDuplet => as_variant_comment_duplet: VariantCommentDuplet,
    AgeOfOnsetDuplet => as_age_of_onset_duplet: AgeOfOnsetDuplet,
    AgeLastEncounterDuplet => as_age_last_encounter_duplet: AgeLastEncounterDuplet,
    DeceasedDuplet => as_deceased_duplet: DeceasedDuplet,
    SexDuplet => as_sex_duplet: SexDuplet,
    HpoSeparatorDuplet => as_separator_duplet: HpoSeparatorDuplet,
    HpoTermDuplet => as_hpo_term_duplet: HpoTermDuplet
);

impl Error {
    fn could_not_extract_duplet(item: &str, i: usize) -> Self {
        Error::TemplateError { msg: format!("Could not extract {item} at index {i}") }
    }

    fn could_not_extract_hpo_duplet(i: usize) -> Self {
        Error::TemplateError { msg: format!("Could not extract HPO Term Column at index {i}") }
    }

    fn template_index_error(actual: usize, maxi: usize, template_name: &str) -> Self {
        Error::TemplateError { msg: format!("Attempt to access item at index {actual} but {template_name} has {maxi} items.") }
    }

    fn no_hpo_column(i: usize, j: usize) -> Self {
        Error::TemplateError { 
            msg: format!("could not retrieve HPO column at index i={}, j={}", i, j)
        }
    }

    fn index_too_large(max_val: usize, n_columns: usize) -> Self {
        Error::TemplateError { 
            msg: format!("Attempt to retrieve from index i={} with a HeaderDupletRow of size {}", 
            max_val, n_columns)
        }
    }

    fn indices_empty() -> Self {
        Error::TemplateError { 
            msg: format!("Attempt to retrieve from HeaderDupletRow with empty indices")
        }
    }

    pub fn invalid_header() -> Self {
        Self::TemplateError { msg: "Invalid HeaderDuplet header".to_string() }
    }
    
}




#[derive(Clone, Debug)]
pub struct HeaderDupletRow {
    individual_header: IndividualHeader,
    disease_header_list: Vec<DiseaseHeader>,
    gene_variant_header_list: Vec<GeneVariantHeader>,
    demographic_header: DemographicHeader,
    hpo_duplets: Vec<HpoTermDuplet>,
    template_type: TemplateType,
}


impl HeaderDupletRow {
    pub fn mendelian(
        matrix: &Vec<Vec<String>>,
        hpo: Arc<FullCsrOntology>,
    ) -> std::result::Result<Self, ValidationErrors> {
        Self::qc_matrix_dimensions(matrix)?;
        let verror = ValidationErrors::new();

        /// first Q/C the constant part of the Mendelian header
        let iheader = IndividualHeader::from_matrix(matrix)?;
        let dheader = DiseaseHeader::from_matrix(matrix, MENDELIAN_DISEASE_IDX)?;
        let gheader = GeneVariantHeader::from_matrix(matrix, MENDELIAN_GENE_VAR_IDX)?;
        let demoheader = DemographicHeader::from_matrix(matrix, MENDELIAN_DEMOGRAPHIC_IDX)?;
        
        /// If we get here, the constant part is OK and we can check the HPO columns
        let mut hpo_duplet_list: Vec<HpoTermDuplet> = Vec::new();
        let index = HeaderIndexer::n_constant_mendelian_columns();
        let n = matrix[0].len(); // previously checked in qc_matrix_dimensions
        for i in MENDELIAN_HPO_IDX..n {
            let hdup = HpoTermDuplet::new(&matrix[0][i], &matrix[1][i]);
            hpo_duplet_list.push(hdup);
        }
        let hpo_util = HpoUtil::new(hpo.clone());
        let _ = hpo_util.check_hpo_duplets(&hpo_duplet_list)?;
        Ok(Self { 
            individual_header: iheader, 
            disease_header_list: vec![DiseaseHeader::new()], 
            gene_variant_header_list: vec![GeneVariantHeader::new()], 
            demographic_header: demoheader,
            hpo_duplets: hpo_duplet_list,
            template_type: TemplateType::Mendelian
        })
    }
    //Err(ValidationErrors::new())




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
            demographic_header: self.demographic_header.clone(),
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
            demographic_header: self.demographic_header.clone(),
            hpo_duplets: updated_hpo_duplets,
            template_type: self.template_type,
        }       
    }
    
    pub fn get_hpo_id_list(&self) -> Result<Vec<TermId>> {
        self.hpo_duplets
            .iter()
            .map(|duplet| {
                TermId::from_str(&duplet.row2())
                    .map_err(|_| Error::termid_parse_error(&duplet.row2()))
            })
            .collect()
    }

    pub fn get_hpo_duplets(&self) -> Vec<HpoTermDuplet> {
        self.hpo_duplets.clone()
    }

    fn get_hpo_header_dtos(&self) -> Vec<HeaderDupletDto> {
        self.hpo_duplets.iter()
            .map(|hpo_duplet| hpo_duplet.to_header_dto())
            .collect()
    }

    fn get_mendelian_header_dto(&self) -> HeaderDto {
        let individual_dto = HeaderDupletDto::new("individual", "na");
        let disease_dto = HeaderDupletDto::new("disease", "na");
        let gene_dto = HeaderDupletDto::new("genetics", "na");
        let demographics_dto = HeaderDupletDto::new("demographics", "na");
        HeaderDto {
            individual_header: individual_dto,
            disease_headers: vec![disease_dto],
            gene_var_headers: vec![gene_dto],
            demographic_header: demographics_dto,
            hpo_headers: self.get_hpo_header_dtos(),
        }
    }

     /// Get the data dtransfer objects for creating a header in the display table
    pub fn get_header_dto(&self) -> HeaderDto {
        match self.template_type {
            TemplateType::Mendelian => self.get_mendelian_header_dto(),
            TemplateType::Melded => todo!(),
        }
    }

    pub fn n_mendelian_contant_fields() -> usize {
        N_CONSTANT_FIELDS_MENDELIAN
    }

  

}




#[derive(Clone, Debug)]
pub struct HeaderDupletRowOLD {
   /// Columns to represent the constant fields
    constant_duplets: Vec<HeaderDuplet>,
    /// Section types of the HeaderDuplets
    section_type_list: Vec<SectionType>,
     /// Variable number of columns with the HPO annotations.
    hpo_duplets: Vec<HpoTermDuplet>,
    indexer: HeaderIndexer,
}

impl HeaderDupletRowOLD {
    /// The first part of the pipeline to go from a matrix of strings to a PptTemplate is to extract HeaderDuplets from the
    /// first two rows. This method checks their validty and creates a Mendelian HeaderDupletRow with constant and HPO columns
    /// The validity of the HPO TermId and label in the DTOs should have been checked before we get to this function.
    

    /// We use this function when we add new HPO terms to the cohort; since the previous HeaderRowDuplet does not
    /// have these terms, we take the existing constant fields and append the new HPO term duplets (Note: client
    /// code should have arranged the HPO term list previously). We will then use this to update the existing PpktRow objects
    pub fn update(&self, updated_hpo_duplets: &Vec<HpoTermDuplet>) -> Result<Self> {
        Ok(Self { 
            constant_duplets: self.constant_duplets.clone(), 
            section_type_list: self.section_type_list.clone(), 
            hpo_duplets: updated_hpo_duplets.clone(), 
            indexer: self.indexer.clone() 
        })       
    }

   




    fn check_hpo_term(header_duplets: &Vec<HeaderDuplet>, i: usize) -> Result<HpoTermDuplet> {
        let hpo_term_dup = header_duplets
            .get(i)
            .ok_or_else(|| Error::could_not_extract_hpo_duplet(i))?
            .as_hpo_term_duplet()?;
        Ok(hpo_term_dup.clone())
    }

    

    pub fn qc_check(&self, i: usize, cell_contents: &str) -> Result<()> {
        if i < self.constant_duplets.len() {
            match self.constant_duplets.get(i) {
                Some(hdup) => {
                    hdup.qc_cell(cell_contents)?;
                    Ok(())
                },
                None => {
                    return Err(Error::EmptyLabel)
                }
            }
        } else {
            let j = i - self.constant_duplets.len();
            match self.hpo_duplets.get(j) {
                Some(hdup) => {
                    hdup.qc_cell(cell_contents)?;
                    Ok(())
                },
                None => {
                    return Err(Error::EmptyLabel)
                }
            }
        }
    }




    /// Get the name of the i'th column
    pub fn get_column_name(&self, i: usize) -> Result<String> {
        /*if self.is_hpo_column(i) {
            let j = i - self.indexer.n_constant();
            match self.hpo_duplets.get(j) {
                Some(hpo_col) => { Ok(hpo_col.row1())  },
                None => {  Err(Error::no_hpo_column(i, j))  }
            }
        } else {
            return self.indexer.get_column_name(i);
        }*/
        Ok("todo getcolumnname".to_ascii_lowercase())
    }




   

    pub fn get_hpo_row(&self, hpo_dto_list: &Vec<HpoTermDto>) -> Vec<String> {
        let hpo_map: HashMap<String, HpoTermDto> = hpo_dto_list
            .into_iter()
            .map(|dto| (dto.term_id().to_string(), dto.clone()))
            .collect();
        let mut values: Vec<String> = Vec::new();
        for hdup in &self.hpo_duplets {
            let tid = hdup.row2();
            if let Some(dto) = hpo_map.get(&tid) {
                if dto.is_not_ascertained() {
                    values.push(NOT_AVAILABLE.to_string());
                } else if dto.is_excluded() {
                    values.push(EXCLUDED.to_string());
                } else if dto.is_observed() {
                    values.push(OBSERVED.to_string());
                } else {
                    values.push(dto.label());
                }
            } else {
                values.push(NOT_AVAILABLE.to_string());
            }
        }
        values
    }


}


#[cfg(test)]
mod test {
    use super::*;
    use crate::{error::Error, header::{header_duplet::HeaderDupletItem, hpo_term_duplet::HpoTermDuplet}};
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


    #[rstest]
    fn test_adding_terms(
        one_case_matrix: Vec<Vec<String>>
    ) -> Result<()> {
        let hdup_list = match HeaderDuplet::extract_from_string_matrix(&one_case_matrix) {
            Ok(val) => val,
            Err(e) => {
                return Err(e); 
            }
        };
       /*  let header_duplet_row = HeaderDupletRow::mendelian_from_duplets(hdup_list)?;
        let hpo_duplete_list = header_duplet_row.get_hpo_duplets();
        assert_eq!(2, hpo_duplete_list.len());
        // Add one term
        let xerostomia: TermId = ("HP", "0000217").into();
        let hpo_term_dup = HpoTermDuplet::new("Xerostomia", "HP:0000217");
        let mut terms_new = Vec::new();
        terms_new.extend(hpo_duplete_list);
        terms_new.push(hpo_term_dup);
        /// In client code, we would check and arrange the HPO terms here.
        let updated_row = header_duplet_row.update(&terms_new)?;
        let hpo_duplete_list = updated_row.get_hpo_duplets();
        assert_eq!(3, hpo_duplete_list.len());
        */
        eprint!("TODO REFACOT TEST");
        Ok(())
    }
   


}