//! HeaderDupletRow: Encapsulate the headers (which we call duplets because each has two fields and which are serialized as the furst two rows of the template)
//! 
//! Each HeaderDuplet determines the meaning of the rows beneath it.
//! We pass a reference (via ARC) of the HeaderDupletRow to each of the rows of the template



use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use std::usize;

use ontolius::ontology::csr::FullCsrOntology;
use ontolius::ontology::OntologyTerms;
use ontolius::term::simple::SimpleTerm;
use ontolius::term::{MinimalTerm, Term};
use ontolius::{Identified, TermId};

use crate::dto::hpo_term_dto::HpoTermDto;
use crate::header::header_duplet::{HeaderDuplet, HeaderDupletItem, HeaderDupletItemFactory};
use crate::header::hpo_separator_duplet::HpoSeparatorDuplet;
use crate::header::hpo_term_duplet::HpoTermDuplet;
use crate::header::{individual_id_duplet, variant_comment_duplet};
use crate::header::{age_last_encounter_duplet::AgeLastEncounterDuplet, age_of_onset_duplet::AgeOfOnsetDuplet, allele_1_duplet::Allele1Duplet, allele_2_duplet::Allele2Duplet, comment_duplet::CommentDuplet, deceased_duplet::DeceasedDuplet, disease_id_duplet::DiseaseIdDuplet, disease_label_duplet::DiseaseLabelDuplet, gene_symbol_duplet::GeneSymbolDuplet, hgnc_duplet::HgncDuplet, individual_id_duplet::IndividualIdDuplet, pmid_duplet::PmidDuplet, sex_duplet::SexDuplet, title_duplet::TitleDuplet, transcript_duplet::TranscriptDuplet, variant_comment_duplet::VariantCommentDuplet};
use crate::error::{self, Error, Result};

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

/// Number of columns in the Individual section
const NUMBER_OF_INDIVIDUAL_FIELDS: usize = 4;
/// Number of columns in the Disease/Gene/Variant bundle section
const NUMBER_OF_DISEASE_GENE_BUNDLE_FIELDS: usize = 8;
/// Number of columns in the Demographic section
const NUMBER_OF_DEMOGRAPHIC_FIELDS: usize = 4;
/// Separator field (HPO/na)
const NUMBER_OF_SEPARATOR_FIELDS: usize = 1;

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
   /// Columns to represent the constant fields
    constant_duplets: Vec<HeaderDuplet>,
    /// Section types of the HeaderDuplets
    section_type_list: Vec<SectionType>,
     /// Variable number of columns with the HPO annotations.
    hpo_duplets: Vec<HpoTermDuplet>,
    indexer: HeaderIndexer,
}

impl HeaderDupletRow {
    /// The first part of the pipeline to go from a matrix of strings to a PptTemplate is to extract HeaderDuplets from the
    /// first two rows. This method checks their validty and creates a Mendelian HeaderDupletRow with constant and HPO columns
    /// The validity of the HPO TermId and label in the DTOs should have been checked before we get to this function.
    pub fn mendelian_from_duplets(
        header_duplets: Vec<HeaderDuplet>,
    ) -> Result<Self> {
        let constant_duplets = HeaderIndexer::extract_mendelian_constant_duplets(&header_duplets)?;
         /// Now get the HPO columns - if we get here, we could extract the Mendelian headers
         let mut hpo_duplet_vec = Vec::new();
         let index = HeaderIndexer::n_constant_mendelian_columns();
         for hdup in header_duplets.iter().skip(index) {
            let hpo_dup = hdup.as_hpo_term_duplet()?;
            hpo_duplet_vec.push(hpo_dup.clone()); 
         }
        Ok(Self {
            constant_duplets:constant_duplets,
            hpo_duplets: hpo_duplet_vec,
            section_type_list: SectionType::mendelian(),
            indexer: HeaderIndexer::mendelian(),
        })
    }



    pub fn mendelian(hpo_duplets: Vec<HpoTermDuplet>) -> Self {
        let mut constant_fields: Vec<HeaderDuplet> = Vec::new();
        /// Individual
        constant_fields.push(PmidDuplet::new().into_enum());
        constant_fields.push(TitleDuplet::new().into_enum());
        constant_fields.push(IndividualIdDuplet::new().into_enum());
        constant_fields.push(CommentDuplet::new().into_enum());
        /// DiseaseGeneBundle
        constant_fields.push(DiseaseIdDuplet::new().into_enum());
        constant_fields.push(DiseaseLabelDuplet::new().into_enum());
        constant_fields.push(HgncDuplet::new().into_enum());
        constant_fields.push(GeneSymbolDuplet::new().into_enum());
        constant_fields.push(TranscriptDuplet::new().into_enum());
        constant_fields.push(Allele1Duplet::new().into_enum());
        constant_fields.push(Allele2Duplet::new().into_enum());
        constant_fields.push(VariantCommentDuplet::new().into_enum());
        /// Demographic
        constant_fields.push(AgeOfOnsetDuplet::new().into_enum());
        constant_fields.push(AgeLastEncounterDuplet::new().into_enum());
        constant_fields.push(DeceasedDuplet::new().into_enum());
        constant_fields.push(SexDuplet::new().into_enum());
        /// HPO Separator column
        constant_fields.push(HpoSeparatorDuplet::new().into_enum());
        ///

        Self {
            constant_duplets: constant_fields,
            section_type_list: SectionType::mendelian(),
            hpo_duplets: hpo_duplets,
            indexer: HeaderIndexer::mendelian()
        }
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
            constant_duplets: self.constant_duplets.clone(), 
            section_type_list: self.section_type_list.clone(), 
            hpo_duplets: updated_hpo_duplets, 
            indexer: self.indexer.clone() 
        }          
    }

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

    fn qc_header(&self) -> Result<()> {
        todo!()
    }

    pub fn get_idx(&self, column_name: &str) -> Result<usize> {
        self.indexer.get_idx(column_name)
    }


    fn check_hpo_term(header_duplets: &Vec<HeaderDuplet>, i: usize) -> Result<HpoTermDuplet> {
        let hpo_term_dup = header_duplets
            .get(i)
            .ok_or_else(|| Error::could_not_extract_hpo_duplet(i))?
            .as_hpo_term_duplet()?;
        Ok(hpo_term_dup.clone())
    }

    pub fn n_mendelian_contant_fields() -> usize {
        N_CONSTANT_FIELDS_MENDELIAN
    }

    pub fn hpo_count(&self) -> usize {
        self.hpo_duplets.len()
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



    /// return a String matrix (two rows) representing the header
    pub fn get_string_matrix(&self) -> Vec<Vec<String>> {
        let mut row1: Vec<String> = Vec::with_capacity(self.n_columns());
        let mut row2: Vec<String> = Vec::with_capacity(self.n_columns());
        for hdup in &self.constant_duplets {
            row1.push(hdup.row1());
            row2.push(hdup.row2());
        }
        for hdup in &self.hpo_duplets {
            row1.push(hdup.row1());
            row2.push(hdup.row2());
        }
        let rows: Vec<Vec<String>> = vec![row1, row2];
        rows
    }

    pub fn n_columns(&self) -> usize {
        if self.indexer.is_mendelian() {
            return N_CONSTANT_FIELDS_MENDELIAN + self.hpo_duplets.len();
        } else {
            panic!("Need to implement n_columns for melded");
        }
    }

    pub fn is_hpo_column(&self, i: usize) -> bool {
        if self.indexer.is_constant_idx(i) {
            return false;
        } else if i >= self.n_columns() {
            return false;
        } else {
            return true;
        }
    }

      /// Get the name of the i'th column
      pub fn get_column_name(&self, i: usize) -> Result<String> {
        if self.is_hpo_column(i) {
            let j = i - self.indexer.n_constant();
            match self.hpo_duplets.get(j) {
                Some(hpo_col) => { Ok(hpo_col.row1())  },
                None => {  Err(Error::no_hpo_column(i, j))  }
            }
        } else {
            return self.indexer.get_column_name(i);
        }
    }


    
    pub fn n_constant(&self) -> usize {
        self.constant_duplets.len()
    }


    pub fn get_duplet_at_index(&self, i: usize) -> Result<HeaderDuplet> {
       if i > self.n_columns() {
            return Err(Error::TemplateError { msg: format!("index out of bounds") })
       } else if i < self.n_constant() {
            Ok(self.constant_duplets[i].clone())
        } else {
            let j = i - self.n_constant();
            Ok(self.hpo_duplets[j].clone().into_enum())
        }
    }

    pub fn get_selected_columns(&self, indices: &Vec<usize>) -> Result<Vec<Vec<String>>> {
        if let Some(&max_val) = indices.iter().max() {
            if max_val > self.n_columns() {
                return Err(Error::index_too_large(max_val, self.n_columns()));
            }
        } else {
            return Err(Error::indices_empty());
        }
        let rows: Vec<Vec<String>> = Vec::new();



        Ok(rows)
    }

    /// Get the index of the first HPO term
    pub fn get_hpo_offset(&self) -> usize {
        self.constant_duplets.len()
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
        let header_duplet_row = HeaderDupletRow::mendelian_from_duplets(hdup_list)?;
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
        Ok(())
    }
   


}