use std::fs::{File, OpenOptions};
use std::{collections::HashMap, path::PathBuf};



use crate::dto::validation_errors::ValidationErrors;
use crate::dto::variant_dto::VariantListDto;
use crate::variant::hgvs_variant::HgvsVariant;
use crate::variant::structural_validator::StructuralValidator;
use crate::{dto::variant_dto::VariantDto, variant::variant_validator::VariantValidator};


use crate::variant::structural_variant::StructuralVariant;

type VariantCache = HashMap<String, HgvsVariant>;
type StructuralCache = HashMap<String, StructuralVariant>;

pub struct VariantManager {
    hgvs_cache_file_path: PathBuf,
    hgvs_cache: VariantCache,
    structural_cache_file_path: PathBuf,
    structural_cache: StructuralCache,
    variant_map: HashMap<String, VariantDto>,
    validator: VariantValidator,
    structural_validator: StructuralValidator
}




impl VariantManager {
    pub fn new(path_buf: &PathBuf) -> Self {
        let hgvs_cache_file_path = path_buf.join("hgvs_cache.txt");
        let cache_obj: VariantCache = 
            Self::load_hgvs(&hgvs_cache_file_path).unwrap_or_else(|_| HashMap::new());
        let structural_cache_file_path = path_buf.join("structural_cache.txt");
        let structural_cache_obj = 
            Self::load_structural(&structural_cache_file_path).unwrap_or_else(|_| HashMap::new());
        Self {
            hgvs_cache_file_path,
            hgvs_cache: cache_obj,
            structural_cache_file_path,
            structural_cache: structural_cache_obj,
            variant_map: HashMap::new(),
            validator: VariantValidator::hg38(),
            structural_validator: StructuralValidator::hg38()
        }
    }

    pub fn add_variant(&mut self, variant_dto: &VariantDto) {
        self.variant_map.insert(variant_dto.variant_string().to_string(), variant_dto.clone());
    }

    pub fn add_variant_list(&mut self, variants: &[VariantDto] ) {
        for dto in variants {
            self.variant_map.insert(dto.variant_string().to_string(), dto.clone());
        }
    }

    fn save_hgvs(&self) -> Result<(), String> {
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&self.hgvs_cache_file_path)
            .map_err(|e| e.to_string())?;
        serde_json::to_writer_pretty(file, &self.hgvs_cache)
            .map_err(|e| e.to_string())?; 
        Ok(())
    }
    
    fn load_hgvs(path: &PathBuf) -> Result<VariantCache, String> {
        let file = File::open(path).map_err(|e| e.to_string())?;
        let cache = serde_json::from_reader(file)
            .map_err(|e| e.to_string())?;
        Ok(cache)
    }

    fn save_structural(&self) -> Result<(), String> {
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&self.structural_cache_file_path)
            .map_err(|e| e.to_string())?;
        serde_json::to_writer_pretty(file, &self.structural_cache)
            .map_err(|e| e.to_string())?; 
        Ok(())
    }

    fn load_structural(structural_cache_file_path: &PathBuf) -> Result<StructuralCache, String> {
        let file = File::open(structural_cache_file_path).map_err(|e| e.to_string())?;
        let cache = serde_json::from_reader(file).map_err(|e| e.to_string())?;
        Ok(cache)
    }

    pub fn n_hgvs(&self) -> usize {
        self.hgvs_cache.len()
    }

    pub fn n_sv(&self) -> usize {
        self.structural_cache.len()
    }

    pub fn clear_cache(&mut self) {
        self.hgvs_cache.clear();
        self.structural_cache.clear();
    }

    pub fn get_hgvs_variant(&self, var_str: &str) -> Option<HgvsVariant> {
        self.hgvs_cache.get(var_str).cloned()
    }

    pub fn get_sv(&self, var_str: &str) -> Option<StructuralVariant> {
        self.structural_cache.get(var_str).cloned()
    }

    /// Extract a list of the variant DTOs sorted such that the HGVS variants come first and are sorted
    /// by gene symbol and then alphanumerbetically by HGVS nomenclature
    pub fn sorted_variant_dtos(&self) -> Vec<VariantDto> {
        let mut variant_list: Vec<VariantDto> = self.variant_map.values().cloned().collect();
        variant_list.sort_by(|a, b| {
            (
                a.is_structural(), // false < true
                a.gene_symbol(),
                a.numerical_key(),
                a.variant_string(),
            )
            .cmp(&(
                b.is_structural(),
                b.gene_symbol(),
                b.numerical_key(),
                b.variant_string(),
            ))
        });
        variant_list
    }

    /// Check if a variant is valid and if so add it to the cache. If not, return an Error.
    pub fn validate_variant(&mut self, dto: &VariantDto) -> Result<VariantDto, String> {
        let key = dto.variant_string();

        if dto.is_structural() {
            if self.structural_cache.contains_key(key) {
                Ok(dto.clone_validated())
            } else {
                let sv = self.structural_validator.validate_sv(dto)?;
                self.structural_cache.insert(key.to_string(), sv);
                self.save_structural();
                Ok(dto.clone_validated())
            }
        } else if self.hgvs_cache.contains_key(key) {
            Ok(dto.clone_validated())
        } else {
            let hgvs = self.validator.validate_hgvs(dto)?;
            self.hgvs_cache.insert(key.to_string(), hgvs);
            self.save_hgvs();
            Ok(dto.clone_validated())
        }
    }

    pub fn validate_variants(&mut self, dto_list: &Vec<VariantDto>) -> Result<(), ValidationErrors> {
        let mut verrs = ValidationErrors::new();
        for dto in dto_list {
            match self.validate_variant(dto) {
                Ok(dto) => {
                    return Ok(()); },
                Err(e) => {
                    verrs.push_str(e);
                }
            }
        }
        verrs.ok()
    }


    pub fn get_variant_list_dto(&self) 
    -> VariantListDto {
        let verrs = ValidationErrors::new();
        let mut evaluated_dto_list: Vec<VariantDto> = Vec::with_capacity(self.variant_map.len());
        for (variant, dto) in self.variant_map.iter() {
            if dto.is_structural() {
                if self.structural_cache.contains_key(variant ) {
                    evaluated_dto_list.push(dto.clone_validated());
                } else {
                    evaluated_dto_list.push(dto.clone_unvalidated());
                }
            } else if self.hgvs_cache.contains_key(variant) {
                evaluated_dto_list.push(dto.clone_validated());
            } else {
                evaluated_dto_list.push(dto.clone_unvalidated());
            }
        }
        VariantListDto::new(evaluated_dto_list)
    }

    pub fn validate_variant_dto_list(&mut self, variant_dto_list: Vec<VariantDto>) -> Result<Vec<VariantDto>, String> {
        let mut evaluated_dto_list: Vec<VariantDto> = Vec::with_capacity(variant_dto_list.len());
        for dto in variant_dto_list {
            let variant = dto.variant_string();
            if dto.is_structural() {
                if self.structural_cache.contains_key(variant ) {
                    evaluated_dto_list.push(dto.clone_validated());
                } else {
                    match self.structural_validator.validate_sv(&dto) {
                        Ok(sv) => {
                            self.structural_cache.insert(variant.to_string(), sv);
                            evaluated_dto_list.push(dto.clone_validated());
                        },
                        Err(e) => {
                            evaluated_dto_list.push(dto.clone_unvalidated());
                        },
                    }
                }
            } else if self.hgvs_cache.contains_key(variant) {
                evaluated_dto_list.push(dto.clone_validated());
            } else {
                match self.validator.validate_hgvs(&dto) {
                    Ok(hgvs) => {
                        self.hgvs_cache.insert(variant.to_string(), hgvs);
                        evaluated_dto_list.push(dto.clone_validated());
                    },
                    Err(e) => {
                        evaluated_dto_list.push(dto.clone_unvalidated());
                    },
                }
            }
        }
        // write variants to cache.
        self.save_hgvs()?;
        self.save_structural()?; 
        VariantDto::sort_variant_dtos(&mut evaluated_dto_list);
        Ok(evaluated_dto_list)
    }


    pub fn get_hgvs_dict(&self) -> &HashMap<String, HgvsVariant> {
        &self.hgvs_cache
    }

    pub fn get_structural_dict(&self) -> &HashMap<String, StructuralVariant> {
        &self.structural_cache
    }


}