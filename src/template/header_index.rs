//! A convenience module to get the index of specific columns

use std::collections::{HashMap, HashSet};
use lazy_static::lazy_static;

use crate::{error::{self, Error, Result}, header::{header_duplet::*, hpo_term_duplet::HpoTermDuplet, pmid_duplet::PmidDuplet, title_duplet::TitleDuplet}};

#[derive(Clone, Debug, PartialEq)]
enum IndexerType {
    Mendelian,
    Blended,
}

const MENDELIAN_HEADER_FIELDS: [&str; 17] = [
    "PMID", "title", "individual_id", "comment", "disease_id", "disease_label",
    "HGNC_id", "gene_symbol", "transcript", "allele_1", "allele_2",
    "variant.comment", "age_of_onset", "age_at_last_encounter",
    "deceased", "sex", "HPO",
];


lazy_static! {
    pub static ref MENDELIAN_DUPLETS: Vec<HeaderDuplet> = vec![
        crate::header::pmid_duplet::PmidDuplet::new().into_enum(),
        crate::header::title_duplet::TitleDuplet::new().into_enum(),
        crate::header::individual_id_duplet::IndividualIdDuplet::new().into_enum(),
        crate::header::comment_duplet::CommentDuplet::new().into_enum(),
        crate::header::disease_id_duplet::DiseaseIdDuplet::new().into_enum(),
        crate::header::disease_label_duplet::DiseaseLabelDuplet::new().into_enum(),
        crate::header::hgnc_duplet::HgncDuplet::new().into_enum(),
        crate::header::gene_symbol_duplet::GeneSymbolDuplet::new().into_enum(),
        crate::header::transcript_duplet::TranscriptDuplet::new().into_enum(),
        crate::header::allele_1_duplet::Allele1Duplet::new().into_enum(),
        crate::header::allele_2_duplet::Allele2Duplet::new().into_enum(),
        crate::header::variant_comment_duplet::VariantCommentDuplet::new().into_enum(),
        crate::header::age_of_onset_duplet::AgeOfOnsetDuplet::new().into_enum(),
        crate::header::age_last_encounter_duplet::AgeLastEncounterDuplet::new().into_enum(),
        crate::header::deceased_duplet::DeceasedDuplet::new().into_enum(),
        crate::header::sex_duplet::SexDuplet::new().into_enum(),
        crate::header::hpo_separator_duplet::HpoSeparatorDuplet::new().into_enum()
    ];
}





lazy_static! {
    pub static ref MENDELIAN_INDEX_MAP: HashMap<String, usize> =  {
        let mut imap = HashMap::new();
        for (idx, key) in MENDELIAN_DUPLETS.iter().enumerate() {
            imap.insert(key.row1(), idx);
        }
        imap
    };
}

#[derive(Clone, Debug)]
pub struct HeaderIndexer {
    title_to_index_map: HashMap<String, usize>,
    indexer_type: IndexerType,
}

impl HeaderIndexer {


    pub fn mendelian() -> Self {
        Self {
            title_to_index_map: MENDELIAN_INDEX_MAP.clone(),
            indexer_type: IndexerType::Mendelian,
        }
    }



    pub fn is_valid_mendelian_header_row(header_list: &Vec<HeaderDuplet>) -> bool {
        if header_list.len() > MENDELIAN_DUPLETS.len() {
            return false;
        }
        for (idx, key) in MENDELIAN_DUPLETS.iter().enumerate() {
            if key.row1() != header_list[idx].row1() {
                return false;
            } else if key.row2() != header_list[idx].row2() {
                return false;
            }
        }
        true
    }

    pub fn extract_mendelian_constant_duplets(header_list: &Vec<HeaderDuplet>) -> Result<Vec<HeaderDuplet>> {
        let constant_duplets: Vec<HeaderDuplet> = Vec::with_capacity(MENDELIAN_DUPLETS.len());
        for (idx, key) in MENDELIAN_DUPLETS.iter().enumerate() {
            if key.row1() != header_list[idx].row1() {
                return Err(Error::HeaderError { msg: format!("HeaderDuplet: Expected {} but got {}", &header_list[idx].row1() , &key.row1() ) });
            } else if key.row2() != header_list[idx].row2() {
                return Err(Error::HeaderError { msg: format!("HeaderDuplet: Expected {} but got {}", &header_list[idx].row2() , &key.row2() ) });
            }
        }
        // if we get here, the template has the correct values for a Mendelian header (which are constant)
        // so we can just clone the following slice
        Ok(MENDELIAN_DUPLETS.to_vec())
    }

   

    pub fn get_idx(&self, column_name: &str) -> Result<usize> {
        match self.title_to_index_map.get(column_name).copied() {
            Some(i) => Ok(i),
            None => Err(Error::HeaderError { msg: format!("Could not find index for '{}'", column_name) })
        }
    }

    /// This method can be use to help decide if an index (usize) 
    /// refers to an HPO Term column or one of the constant columns
    /// * Returns
    ///  - `true` if the index falls within the range of the constant (non-HPO term) columns
    pub fn is_constant_idx(&self, i: usize) -> bool {
        return i < self.title_to_index_map.len();
    }

    pub fn n_constant(&self) -> usize {
        self.title_to_index_map.len()
    }

    pub fn is_mendelian(&self) -> bool {
        return self.indexer_type == IndexerType::Mendelian;
    }

    pub fn n_constant_mendelian_columns() -> usize {
        MENDELIAN_DUPLETS.len()
    }

    pub fn get_column_name(&self, i: usize) -> Result<String> {
        if i < MENDELIAN_HEADER_FIELDS.len() {
            Ok(MENDELIAN_HEADER_FIELDS[i].to_string())
        } else {
            Err(Error::HeaderError { msg: format!("header index '{}' out of bounds", i)})
        }
    }

}



