//! VariantManager
//! This class is meant to help with the refactoring of the legacy Excel sheets to our new
//! format for recording variants. When we ingest an Excel file, we store all information
//! about variants from the legacy sheets here. We know that we only have Mendelian variants
//! (possibly structural or HGVS). The user can send requests from the GUI to validate these
//! variants. If we start with a large number of such variants, then it is possible that the
//! VariantValidator API may not correctly validate all variants; but we add all validated variants
//! to the CohortDto (that the user sends from the front end, because the CohortDto in the front-end
//! represents our source of truth). This step may need to be repeated. We expect that all of the
//! variants can eventually be validated (because they were all validated in pyphetools), but if there is
//! an insoluble problem with one of them, then we can edit the variant by hand in the GUI or in the
//! Excel sheet as needed. 
//! When we import a legacy Excel file, this step needs to be completed before any other work can be done.
//! Note that we know there is exactly one gene symbol, HGNC id, and transcript for all of our legacy 
//! variants, so we add them here to the struct.
use std::collections::{HashMap, HashSet};
use std::{mem, thread};
use std::time::Duration;

use crate::dto::cohort_dto::{CohortDto, GeneTranscriptDto};


use crate::dto::variant_dto::{VariantValidationDto, VariantValidationType};
use crate::dto::hgvs_variant::HgvsVariant;
use crate::variant::structural_validator::StructuralValidator;
use crate::{variant::hgvs_variant_validator::HgvsVariantValidator};


use crate::dto::structural_variant::{StructuralVariant, SvType};



pub struct VariantManager {
    hgvs_validator: HgvsVariantValidator,
    structural_validator: StructuralValidator,
    /// Gene symbol for the variants we are validating
    gene_symbol: String,
    /// HUGO Gene Nomenclature Committee (HGNS) identifier for the above gene
    hgnc_id: String,
    /// Transcript of reference for theabove gene
    transcript: String,
    /// Set of all allele strings (e.g., c.123A>T or DEL Ex 5)
    allele_set: HashSet<String>,
    /// HGVS Variants that could be validated. The key is the original allele denomination (e.g., c.1234A>T), not the variantKey
    validated_hgvs: HashMap<String, HgvsVariant>,
    /// HGStructural Variants that could be validated. The key is the original allele denomination (e.g., DEL Ex 5), not the variantKey
    validated_sv: HashMap<String, StructuralVariant>
}




impl VariantManager {
    /// Construct a VariantManager object for a specific gene/HGNC/transcript
    pub fn new(symbol: &str, hgnc: &str, transcript: &str) -> Self {
        Self {
            hgvs_validator: HgvsVariantValidator::hg38(),
            structural_validator: StructuralValidator::hg38(),
            gene_symbol: symbol.to_string(),
            hgnc_id: hgnc.to_string(),
            transcript: transcript.to_string(),
            allele_set: HashSet::new(),
            validated_hgvs: HashMap::new(),
            validated_sv: HashMap::new(),
        }
    }

    /// Construct a VariantManager object for a specific gene/HGNC/transcript
    pub fn from_gene_transcript_dto(dto: &GeneTranscriptDto) -> Self {
        Self::new(&dto.gene_symbol, &dto.hgnc_id, &dto.transcript)
    }

    /// Perform up to 4 rounds of validation using the VariantValidator API
    /// For each round, increase the latency between network calls
    pub fn validate_all_variants<F>(
        &mut self, all_alleles: &HashSet<String>,
        mut progress_cb: F)  
    -> Result<(), String> 
    where F: FnMut(u32, u32) {
        let n_alleles = all_alleles.len();
        let mut attempts = 0;
        let max_attempts = 4;
        let mut latency = 250 as u64; // time in milliseconds to wait between API calls
        let mut n_validated: u32 = 0;
        let n_alleles = all_alleles.len() as u32;
        self.allele_set = all_alleles.clone();
        while n_validated < n_alleles && attempts < max_attempts {
            for allele in all_alleles {
                if allele.starts_with("c.") || allele.starts_with("n.") {
                    if self.validate_hgvs(allele) {
                        n_validated += 1;
                    }
                } else if  self.validate_sv(&allele) {
                     n_validated += 1;
                }
                // sleep to try to avoid network issues; (start at 250 milliseconds, increase as much in each iteration)
                thread::sleep(Duration::from_millis(latency));
                progress_cb(n_validated, n_alleles);
            }
            latency += 250;
            attempts += 1;
        }
        println!("Round {}: validated: {}", attempts, n_validated);
        
        // When we get here, we will have all variants that could be validated. If some were not validated, either we had not
        // internet or there is actually an error. We will enter their variantKey as na, and the front end will need to do something.
        Ok(())
    }

    ///When we get here, the construction of the CohortDto has proceeded with the addition of the
    /// HashMaps of validated variants. We now need to replace the allele entries (e.g., c.8242G>T) 
    /// with the corresponding keys that we use in the JSON serialization (e.g., c8242GtoT_FBN1_NM_000138v5)
    /// If validation was not successful for some allele, then we return "na" for the variant key (and there will
    /// be no Variant in the HashMaps of the CohortDto); in this case, the user will need to manually edit the
    /// unvalidated variant and validate it from the GUI. This step will be needed both for transforming the
    /// legacy Excel templates and also moving forward for important external supplemental tables.
    pub fn get_variant_key(&self, allele: &str) -> Option<String> {
        if let Some(hgvs) = self.validated_hgvs.get(allele) {
            return Some(hgvs.variant_key());
        }
        if let Some(sv) = self.validated_sv.get(allele) {
            return Some(sv.variant_key().to_string());
        }
        None
    }

    pub fn allele_set(&self) -> HashSet<String> {
        self.allele_set.clone()
    }

    /// Completely analogous to validate_all_sv, see there for documentation
    pub fn validate_hgvs(&mut self, hgvs: &str) -> bool {
        let vv_dto = VariantValidationDto::hgvs_c(hgvs, &self.transcript, &self.hgnc_id, &self.gene_symbol);
        let variant_key = HgvsVariant::generate_variant_key(hgvs, &self.gene_symbol, &self.transcript);
        if self.validated_hgvs.contains_key(&variant_key) {
            println!("Previously validated {}", variant_key);
            return true;
        }
        if let Ok(hgvs) = self.hgvs_validator.validate(vv_dto) {
            println!("Validated {}:{:?}", variant_key, hgvs);
            self.validated_hgvs.insert(variant_key, hgvs.clone());
            return true;
        } else {
            eprint!("Could not validate {hgvs}/{variant_key}");
            return false;
        }
    }


    /// Validates all structural variants in the given set.
    ///
    /// # Arguments
    ///
    /// * `variants` - A set of variant strings (`String`) to validate (allele strings, e.g., c.123A>C or DEL Ex 5).
    /// * `latency` - A latency value (in seconds?) that controls how long to wait between VariantValidator calls
    ///
    /// # Returns
    ///
    /// The number of successfully validated variants.
    ///
    /// # Errors
    ///
    /// Errors are silently skipped under the assumption they may be network errors and this function can 
    /// be called multiple times to get all variants (one a variant string is validated, it is skipped in this function)
    pub fn validate_sv(&mut self, sv: &str) -> bool {
        let vv_dto = VariantValidationDto::sv(sv, &self.transcript, &self.hgnc_id, &self.gene_symbol);
        let sv_type = SvType::try_from(vv_dto.validation_type);
        if sv_type.is_err() {
            eprint!("Could not extract SvType from variant {sv}");
            return false;
        }
        let sv_type = sv_type.unwrap();
        let variant_key = StructuralVariant::generate_variant_key(sv, &self.gene_symbol, sv_type);
        if self.validated_sv.contains_key(&variant_key) {
            println!("Previously validated {}", variant_key);
            return true;
        }
        if let Ok(sv) = self.structural_validator.validate(vv_dto) {
            self.validated_sv.insert(variant_key, sv.clone());
            return true;
        } else {
            eprint!("Could not validate {sv}/{variant_key}");
            return false;
        }
    }


    /// Validate a single variant (either HGVS or structural)
    /// Precise SV not yet implemented.
    pub fn validate_variant(
        &self, 
        vv_dto: VariantValidationDto, 
        mut cohort_dto: CohortDto)
    -> Result<CohortDto, String> {
        match &vv_dto.validation_type {
            VariantValidationType::Hgvs => {
                let hgvs = self.hgvs_validator.validate(vv_dto)?;
                cohort_dto.hgvs_variants.insert(hgvs.variant_key(), hgvs);
                return Ok(cohort_dto);
            } 
            VariantValidationType::PreciseSv => {
                return Err("Precise SV validation not implemented".to_string())
            }
            VariantValidationType::Del 
            | VariantValidationType::Inv 
            | VariantValidationType::Transl 
            | VariantValidationType::Dup
            | VariantValidationType::Sv => {
                let sv = self.structural_validator.validate(vv_dto)?;
                cohort_dto.structural_variants.insert(sv.variant_key().to_string(), sv);
                return Ok(cohort_dto);
            }
        }
    }

    /// Extract a list of the variant DTOs sorted such that the HGVS variants come first and are sorted
    /// by gene symbol and then alphanumerbetically by HGVS nomenclature
    /*
    pub fn sorted_variant_dtos(&self) -> Vec<VariantDto> {
        let mut variant_list: Vec<VariantDto> = self.hgvs_validator.values().cloned().collect();
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
 */


    /// Columns 6,7,8 "HGNC_id",	"gene_symbol", 
    ///    "transcript"
    pub fn from_mendelian_matrix<F>(
        matrix: &Vec<Vec<String>>,  
        progress_cb: F) 
    -> Result<Self, String> 
        where F: FnMut(u32, u32) {
        let hgnc_id_index = 6 as usize;
        let gene_symbol_index = 7 as usize;
        let transcript_index = 8 as usize;
        let allele1_idx = 9 as usize;
        let allele2_idx = 10 as usize;
        if matrix.len() < 3 {
            return Err(format!("Error: Mendelian matrix with too few rows: {}", matrix.len()));
        } 
        let row0 = matrix.get(0).unwrap(); // we know we have thie first row
        if row0.len() < 11 {
            return Err(format!("First matrix row too short: {} fields", row0.len()));
        }
        if row0[hgnc_id_index] != "HGNC_id" {
            return Err(format!("Expected 'HGNC_id' at index {} but got {}", hgnc_id_index, row0[hgnc_id_index] ));
        }
         if row0[gene_symbol_index] != "gene_symbol" {
            return Err(format!("Expected 'gene_symbol' at index {} but got {}", gene_symbol_index, row0[gene_symbol_index] ));
        }
        if row0[transcript_index] != "transcript" {
            return Err(format!("Expected 'transcript' at index {} but got {}", transcript_index, row0[transcript_index] ));
        }
        // get the information from the third row
        let row2 = matrix.get(2).unwrap();
        let hgnc = &row2[hgnc_id_index];
        let symbol = &row2[gene_symbol_index];
        let transcript = &row2[transcript_index];
        let mut vmanager = VariantManager::new(symbol, hgnc, transcript);
        // extract all allele strings
        let mut allele_set: HashSet<String> = HashSet::new();
        let HEADER_ROWS = 2;
        for row in matrix.into_iter().skip(HEADER_ROWS) {
            let a1 = row[allele1_idx].clone();
            let a2 = row[allele2_idx].clone();
            if a1 != "na" {
                allele_set.insert(a1);
            }
            if a2 != "na" {
                allele_set.insert(a2);
            }
        }
        vmanager.validate_all_variants(&allele_set, progress_cb)?;
        Ok(vmanager)
    }


    
    /// Take ownership of the map of validated HGVS variants (map is replaced with empty map in the struct)
    pub fn hgvs_map(&mut self) -> HashMap<String, HgvsVariant> {
         mem::take(&mut self.validated_hgvs)
    }
    /// Take ownership of the map of validated Structural variants
     pub fn sv_map(&mut self) -> HashMap<String, StructuralVariant> {
         mem::take(&mut self.validated_sv)
    }


}



#[cfg(test)]
mod tests {
    use rstest::{rstest};

    use super::*;

    /*
    #[rstest]
    fn test_check_all_vars() {
        let template = "/Users/robin/GIT/phenopacket-store/notebooks/ATP6V0C/input/ATP6V0C_EPEO3_individuals.xlsx";
        let matrix = excel::read_excel_to_dataframe(template).unwrap();
        let mut vmanager = VariantManager::new( "ATP6V0C","HGNC:855", "NM_001694.4");
        vmanager.from_path(template).unwrap();
        
    }*/

}