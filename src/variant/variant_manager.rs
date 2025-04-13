//! This class is designed to extract Variant objects input data.
//! Instead, we first map HGVS using VariantValidator, and cache the results (serde JSON), to
//! avoid calling the VariantValidator API repeatedly for the same variant.
//! Additionally, we create chromosomal deletions, duplications, and inversions, but do not 
//! cache them to file because they are created without using an API


use std::{collections::{HashMap, HashSet}, fs::{File, OpenOptions}, path::Path};
use serde::{Serialize, Deserialize};
use crate::error::{self, Error, Result};
use super::{hgvs_variant::HgvsVariant, structural_variant::StructuralVariant, variant_validator::VariantValidator};


impl Error {
    fn cache_error(err: String) -> Self {
        let msg = format!("Could not open variant cache: {err}");
        Error::VariantCacheError { msg }
    }
}

type VariantCache = HashMap<String, HgvsVariant>;
type ChromosomalVarMap = HashMap<String, StructuralVariant>;

fn save_cache(path: &str, cache: &VariantCache) -> Result<()> {
    let file = OpenOptions::new().write(true).create(true).truncate(true).open(path).map_err(|e| Error::cache_error(e.to_string()) )?;
    serde_json::to_writer_pretty(file, cache).map_err(|e| Error::cache_error(e.to_string()))?; 
    Ok(())
}

fn load_cache(path: &str) -> Result<VariantCache> {
    let file = File::open(path).map_err(|e| Error::cache_error(e.to_string()) )?;
    let cache = serde_json::from_reader(file).map_err(|e| Error::cache_error(e.to_string()) )?;
    Ok(cache)
}

/// provide standard filenaming convention. We pickle results from VariantValidator to avoid
/// calling API multiple times in different runs. For instance, the pickled file of variants for
/// the SCL4A1 cohort will be called "variant_validator_cache_SLC4A1.pickle"
fn get_pickle_filename(name: &str) -> String {
    format!("variant_validator_cache_{name}.json")
}
  
    


pub struct VariantManager {
    gene_symbol: String,
    gene_id: String,
    transcript: String,
    cache_file_name: String,
    variant_cache: VariantCache,
    unmapped_alleles: HashSet<String>,
    variant_validator: VariantValidator,
    structural_alleles: ChromosomalVarMap,
}


impl VariantManager {

    pub fn new(
        symbol: &str,
        gene_id: &str,
        transcript: &str
    ) -> Self {
        let symbol = symbol.to_string();
        let cache_name = get_pickle_filename(&symbol);
        let cache_obj: VariantCache = load_cache(&cache_name).unwrap_or_else(|_| HashMap::new());
        VariantManager { 
            gene_symbol: symbol, 
            gene_id: gene_id.to_string(),
            transcript: transcript.into(), 
            cache_file_name: cache_name, 
            variant_cache: cache_obj,
            unmapped_alleles: HashSet::new(),
            variant_validator: VariantValidator::hg38(transcript),
            structural_alleles: HashMap::new(),
        }
    }
    
    pub fn map_hgvs(&mut self, allele: &str) -> Result<HgvsVariant> {
        match self.variant_cache.get(allele) {
            Some(var) => Ok(var.clone()),
            None => {
                match self.variant_validator.encode_hgvs(allele) {
                    Ok(var) => { return Ok(var); },
                    Err(e) => {
                        self.unmapped_alleles.insert(allele.to_string());
                        return Err(Error::VariantCacheError { msg: format!("Could not create variant object for {allele}: {e}") });
                    }
                };
            }
        }
    }


    pub fn code_as_chromosomal_deletion(&mut self, allele: &str) -> Result<StructuralVariant> {
        let var = StructuralVariant::chromosomal_deletion(allele, &self.gene_symbol, &self.gene_id, None);
        self.structural_alleles.insert(allele.to_string(), var.clone());
        Ok(var)
    }

    pub fn code_as_chromosomal_inversion(&mut self, allele: &str) -> Result<StructuralVariant> {
        let var = StructuralVariant::chromosomal_inversion(allele, &self.gene_symbol, &self.gene_id, None);
        self.structural_alleles.insert(allele.to_string(), var.clone());
        Ok(var)
    }

    pub fn code_as_chromosomal_duplication(&mut self, allele: &str) -> Result<StructuralVariant> {
        let var = StructuralVariant::chromosomal_duplication(allele, &self.gene_symbol, &self.gene_id, None);
        self.structural_alleles.insert(allele.to_string(), var.clone());
        Ok(var)
    }

    pub fn code_as_chromosomal_translocation(&mut self, allele: &str) -> Result<StructuralVariant> {
        let var = StructuralVariant::chromosomal_translocation(allele, &self.gene_symbol, &self.gene_id, None);
        self.structural_alleles.insert(allele.to_string(), var.clone());
        Ok(var)
    }

}