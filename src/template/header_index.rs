//! A convenience module to get the index of specific columns

use std::collections::{HashMap, HashSet};

use lazy_static::lazy_static;


#[derive(Clone, Debug)]
enum IndexerType {
    Mendelian,
    Blended,
}



lazy_static! {
    pub static ref MENDELIAN_INDICES: HashMap<String, usize> =  {
        let mut imap = HashMap::new();
        imap.insert("PMID".to_string(), 0);
        imap.insert("title".to_string(), 1);
        imap.insert("individual_id".to_string(), 2);
        imap.insert("comment".to_string(), 3);
        imap.insert("disease_id".to_string(), 4);
        imap.insert("disease_label".to_string(), 5);
        imap.insert("HGNC_id".to_string(), 6);
        imap.insert("gene_symbol".to_string(), 7);
        imap.insert("transcript".to_string(), 8);
        imap.insert("allele_1".to_string(), 9);
        imap.insert("allele_2".to_string(), 10);
        imap.insert("variant.comment".to_string(), 11);
        imap.insert("age_of_onset".to_string(), 12);
        imap.insert("age_at_last_encounter".to_string(), 13);
        imap.insert("deceased".to_string(), 14);
        imap.insert("sex".to_string(), 15);
        imap.insert("HPO".to_string(), 16);
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
            title_to_index_map: MENDELIAN_INDICES.clone(),
            indexer_type: IndexerType::Mendelian,
        }
    }

    pub fn get_idx(&self, column_name: &str) -> Option<usize> {
        if self.title_to_index_map.contains_key(column_name) {
            self.title_to_index_map.get(column_name).copied()
        } else {
            None
        }
    }
}



