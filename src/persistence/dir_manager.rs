
use std::{collections::HashMap, fs::{self, File, OpenOptions}, path::{Path, PathBuf}};

use crate::variant::{hgvs_variant::HgvsVariant, structural_variant::StructuralVariant, variant_validator::VariantValidator};

use super::ValidatorOfVariants;

use crate::variant::structural_variant::DELETION as DEL;
use crate::variant::structural_variant::DUPLICATION as DUP;
use crate::variant::structural_variant::INVSERSION as INV;
use crate::variant::structural_variant::TRANSLOCATION as TRANSL;

type VariantCache = HashMap<String, HgvsVariant>;
type StructuralCache = HashMap<String, StructuralVariant>;

pub struct DirManager {
    cache_dir_path: PathBuf,
    hgvs_cache_file_path: PathBuf,
    hgvs_cache: VariantCache,
    structural_cache_file_path: PathBuf,
    structural_cache: StructuralCache,
    variant_validator: VariantValidator
}


impl DirManager {
    /// Open the directory at the indicated location; if it does not exist, create it.
    /// Once we have opened the directory, open or create the HGVS cache.
    pub fn new<P: AsRef<Path>>(dir_path: P) -> Result<Self, String> {
        let path_buf = dir_path.as_ref().to_path_buf();
        if !path_buf.exists() {
            fs::create_dir_all(&path_buf).map_err(|e| e.to_string())?;
        }
        if !path_buf.is_dir() {
            return Err(format!("Path exists but is not a directory: {:?}", path_buf));
        }
        let var_cache_path = path_buf.join("hgvs_cache.txt");
        let cache_obj: VariantCache = Self::load_hgvs(&var_cache_path).unwrap_or_else(|_| HashMap::new());
        let struct_cache_path = path_buf.join("structural_cache.txt");
        let structural_cache_obj = Self::load_structural(&struct_cache_path).unwrap_or_else(|_| HashMap::new());
        let vvalidator = VariantValidator::hg38();
        Ok(Self {
            cache_dir_path: path_buf,
            hgvs_cache_file_path: var_cache_path,
            hgvs_cache: cache_obj,
            structural_cache_file_path: struct_cache_path,
            structural_cache: structural_cache_obj,
            variant_validator: vvalidator,
        })
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
}


impl ValidatorOfVariants for DirManager {
    fn validate_variant(&mut self, variant: &str, transcript: &str) -> Result<(), String> {
        let full_hgvs = format!("{transcript}:{variant}");
        if self.hgvs_cache.contains_key(&full_hgvs) {
            return Ok(());
        } else {
            let var = self.variant_validator.encode_hgvs(variant, transcript)
                .map_err(|e| e.to_string())?;
            self.hgvs_cache.insert(full_hgvs, var);
            self.save_hgvs()?;
            return Ok(());
        }
    }

    fn validate_sv(&mut self, variant: &str, hgnc_id: &str, gene_symbol: &str) -> Result<(), String> {
        if self.structural_cache.contains_key(variant) {
            return Ok(());
        }
        let mut parts = variant.splitn(2, ':');
        if let (Some(prefix), Some(rest)) = (parts.next().map(str::trim), parts.next()) {
            let sv_string: &str = rest.trim();
            match prefix {
                DEL => {
                    let sv= StructuralVariant::code_as_chromosomal_deletion(variant, hgnc_id, gene_symbol)
                        .map_err(|e| e.to_string())?;
                    self.structural_cache.insert(variant.to_string(), sv);
                },
                DUP => {
                    let sv= StructuralVariant::code_as_chromosomal_duplication(variant, hgnc_id, gene_symbol)
                    .map_err(|e| e.to_string())?;
                    self.structural_cache.insert(variant.to_string(), sv);
                },
                INV => {
                    let sv= StructuralVariant::code_as_chromosomal_inversion(variant, hgnc_id, gene_symbol)
                    .map_err(|e| e.to_string())?;
                    self.structural_cache.insert(variant.to_string(), sv);
                },
                TRANSL => {
                    let sv= StructuralVariant::code_as_chromosomal_translocation(variant, hgnc_id, gene_symbol)
                    .map_err(|e| e.to_string())?;
                    self.structural_cache.insert(variant.to_string(), sv);
                },
                other => {
                    return Err(format!("Did not recognize SV prefix {prefix} in '{variant}'"));
                }
            }
        }
        Ok(())
    }
}