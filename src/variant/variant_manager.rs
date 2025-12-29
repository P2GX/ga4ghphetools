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
use std::{thread};
use std::time::Duration;

use crate::dto::cohort_dto::{CohortData, GeneTranscriptData};
use crate::dto::intergenic_variant::IntergenicHgvsVariant;
use crate::dto::variant_dto::{VariantDto, VariantType};
use crate::dto::hgvs_variant::HgvsVariant;
use crate::variant::intergenic_hgvs_validator::IntergenicHgvsValidator;
use crate::variant::structural_validator::StructuralValidator;
use crate::{variant::hgvs_variant_validator::HgvsVariantValidator};
use crate::dto::structural_variant::StructuralVariant;



pub struct VariantManager {
    hgvs_validator: HgvsVariantValidator,
    structural_validator: StructuralValidator,
    intergenic_validator: IntergenicHgvsValidator,
    /// Gene symbol for the variants we are validating
    gene_symbol: String,
    /// HUGO Gene Nomenclature Committee (HGNS) identifier for the above gene
    hgnc_id: String,
    /// Transcript of reference for theabove gene
    transcript: String,
    /// Set of all allele strings (e.g., c.123A>T or DEL Ex 5)
    allele_set: HashSet<String>,   
}




impl VariantManager {
    /// Construct a VariantManager object for a specific gene/HGNC/transcript
    /// # Arguments
    ///
    /// * `symbol`     – Gene symbol (e.g. `"BRCA1"`).
    /// * `hgnc`       – HGNC identifier for the gene (e.g., `"HGNC:123"``).
    /// * `transcript` – Transcript identifier against which the variants should be validated (e.g., `"NM_123.1"`).
    pub fn new(symbol: &str, hgnc: &str, transcript: &str) -> Self {
        Self {
            hgvs_validator: HgvsVariantValidator::hg38(),
            structural_validator: StructuralValidator::hg38(),
            intergenic_validator: IntergenicHgvsValidator::hg38(),
            gene_symbol: symbol.to_string(),
            hgnc_id: hgnc.to_string(),
            transcript: transcript.to_string(),
            allele_set: HashSet::new(),
        }
    }

    /// Construct a VariantManager object for a specific gene/HGNC/transcript
    pub fn from_gene_transcript_dto(dto: &GeneTranscriptData) -> Self {
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
                if ! allele.is_ascii() {
                    return Err(format!("Non-ASCII character in allele label: '{allele}'"));
                }
                if allele.starts_with("c.") || allele.starts_with("n.") {
                    if self.validate_hgvs(allele).is_ok() {
                        n_validated += 1;
                    }
                } else if allele.starts_with("NC_") {
                    if self.validate_intergenic(allele).is_ok() {
                        n_validated += 1;
                    }
                } else if self.validate_sv(&allele).is_ok() {
                     n_validated += 1;
                }
                // sleep to try to avoid network issues; (start at 250 milliseconds, increase as much in each iteration)
                thread::sleep(Duration::from_millis(latency));
                progress_cb(n_validated, n_alleles);
            }
            latency += 250;
            attempts += 1;
        }
        Ok(())
    }

    /// Perform up to 4 rounds of validation using the VariantValidator API
    /// For each round, increase the latency between network calls
    pub fn validate_all_sv<F>(
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
                if ! allele.is_ascii() {
                    return Err(format!("Non-ASCII character in allele label: '{allele}'"));
                }
                if ! allele.starts_with("c.") && ! allele.starts_with("n.") {
                    if  self.validate_sv(&allele).is_ok() {
                        n_validated += 1;
                    }
                }
                // sleep to try to avoid network issues; (start at 250 milliseconds, increase as much in each iteration)
                thread::sleep(Duration::from_millis(latency));
                progress_cb(n_validated, n_alleles);
            }
            latency += 250;
            attempts += 1;
        }
        Ok(())
    }


   pub fn validate_all_hgvs<F>(
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
                    match self.validate_hgvs(allele) {
                        Ok(_) => n_validated += 1,
                        Err(e) => {eprintln!("{e}");} 
                    }
                } 
                // sleep to try to avoid network issues; (start at 250 milliseconds, increase as much in each iteration)
                thread::sleep(Duration::from_millis(latency));
                progress_cb(n_validated, n_alleles);
            }
            latency += 250;
            attempts += 1;
        }       
        // When we get here, we will have all variants that could be validated. If some were not validated, either we had not
        // internet or there is actually an error. We will enter their variantKey as na, and the front end will need to do something.
        Ok(())
    }

   


    /// Completely analogous to validate_all_sv, see there for documentation
    fn validate_hgvs(&mut self, hgvs: &str) -> Result<(), String> {
        let vv_dto = VariantDto::hgvs_c(hgvs, &self.transcript, &self.hgnc_id, &self.gene_symbol);
        self.hgvs_validator.validate(vv_dto)
    }

    fn validate_intergenic(&mut self, intergenic: &str) -> Result<(), String> {
        let vv_dto = VariantDto::hgvs_g(intergenic, &self.hgnc_id, &self.gene_symbol);
        self.intergenic_validator.validate(vv_dto)
    }

    pub(crate) fn get_validated_hgvs(&mut self, hgvs: &str) 
    -> Result<HgvsVariant, String> {
        let vv_dto = VariantDto::hgvs_c(hgvs, &self.transcript, &self.hgnc_id, &self.gene_symbol);
        self.hgvs_validator.get_validated_hgvs(&vv_dto)
    }

    pub fn get_validated_structural_variant(&mut self, allele: &str, var_type: VariantType)
    -> Result<StructuralVariant, String> {
        let vv_dto = VariantDto::sv(allele, &self.transcript, &self.hgnc_id, &self.gene_symbol, var_type);
        self.structural_validator.get_validated_sv(&vv_dto)
    }

    pub(crate) fn get_validated_intergenic_hgvs(&mut self, hgvs: &str) 
    -> Result<IntergenicHgvsVariant, String> {
        let vv_dto = VariantDto::hgvs_g(hgvs, &self.hgnc_id, &self.gene_symbol);
        self.intergenic_validator.get_validated_g_hgvs(&vv_dto)
    }


    /// Validates a single structural variant (SV) string.
    ///
    /// This function creates a `VariantDto` from the provided SV string and
    /// uses the `structural_validator` to perform validation. Validation
    /// includes checking the variant format and encoding it as a
    /// chromosomal structural variant.
    ///
    /// # Arguments
    ///
    /// * `sv` - A structural variant string, e.g., `DEL Ex 5` or other symbolic non-precise SVs.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the variant was successfully validated and added to the
    ///   set of validated structural variants.
    /// * `Err(String)` if validation failed (e.g., invalid format or
    ///   unsupported variant type).
    ///
    /// # Notes
    ///
    /// This function is intended to be called multiple times for multiple
    /// variants. Once a variant is validated, it is stored internally and
    /// will be skipped in subsequent calls.
    pub fn validate_sv(&mut self, sv: &str) -> Result<(), String> {
        let vv_dto = VariantDto::sv(sv, &self.transcript, &self.hgnc_id, &self.gene_symbol, VariantType::Sv);
        self.structural_validator.validate(vv_dto)
    }




    /// Columns 6,7,8 "HGNC_id",	"gene_symbol",  "transcript"
    ///   'Legacy' method for transforming the Excel files. 
    /// TODO delete after last legacy file has been transformed!
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
        let n_header_rows = 2;
        for row in matrix.into_iter().skip(n_header_rows) {
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
        self.hgvs_validator.hgvs_map()
    }
    /// Take ownership of the map of validated Structural variants
    pub fn sv_map(&mut self) -> HashMap<String, StructuralVariant> {
        self.structural_validator.sv_map()
    }

    pub fn intergenic_map(&mut self) -> HashMap<String, IntergenicHgvsVariant> {
        self.intergenic_validator.ig_map()
    }

    /// Analyze cohort variants and report their validation status.
    ///
    /// This method iterates over all alleles observed in the cohort and
    /// aggregates them into a per-variant summary. Each unique variant is
    /// represented by a [`VariantDto`] that records how often it occurs and
    /// whether it has been validated.
    ///
    /// The resulting DTOs are intended for consumption by the front end,
    /// allowing it to identify which variants still require validation.
    ///
    /// # Arguments
    ///
    /// * `cohort_dto` – Cohort data containing observed alleles and variant definitions.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<VariantDto>)` containing one entry per unique variant observed
    ///   in the cohort.
    /// * `Err(String)` if variant analysis cannot be completed.
    ///
    /// # Behavior
    ///
    /// * Variants are keyed by allele identifier.
    /// * If a variant can be resolved to an HGVS or structural variant, the
    ///   corresponding [`VariantDto`] is created from that representation.
    /// * If a variant cannot be resolved, it is marked as
    ///   [`VariantType::Unknown`] and flagged as not validated.
    ///
    /// # Notes
    ///
    /// * Variant counts reflect the total number of times an allele appears across
    ///   all cohort rows (i.e., biallelic variants in a sample are counted 2 times).
    /// * Unknown or unresolved variants are still included in the output to
    ///   ensure completeness.
    pub fn analyze_variants(&self, cohort_dto: CohortData)
    -> Result<Vec<VariantDto>, String> {
        let mut var_ana_map: HashMap<String, VariantDto> = HashMap::new();
        for row in &cohort_dto.rows {
            for allele_key in row.allele_count_map.keys() {
                if let Some(hgvs) = cohort_dto.hgvs_variants.get(allele_key) {
                    var_ana_map
                        .entry(allele_key.to_string()) // key type = String
                        .and_modify(|existing| {
                            existing.count += 1; 
                        })
                        .or_insert_with(|| VariantDto::from_hgvs(hgvs, allele_key));
                } else if let Some(sv) = cohort_dto.structural_variants.get(allele_key) {
                    var_ana_map
                        .entry(allele_key.to_string()) 
                        .and_modify(|existing| {
                            existing.count += 1; 
                        })
                        .or_insert_with(|| VariantDto::from_sv(sv, allele_key));   
                } else {
                    let v_ana = VariantDto::not_validated(allele_key);
                    var_ana_map.insert(allele_key.to_string(), v_ana);
                }
            }
        }
        let var_list: Vec<VariantDto> = var_ana_map.into_values().collect();
        Ok(var_list)
    }


}



#[cfg(test)]
mod tests {
    use crate::{dto::{structural_variant::StructuralVariant, variant_dto::VariantType}, variant::variant_manager::VariantManager};

    

    #[test]
    #[ignore = "API call"]
    fn test_sv_key_is_identical() {
        let label = "deletion of exons 2-9";
        let mut manager = VariantManager::new("CNTNAP2", "HGNC:13830", "NM_014141.6");
        let sv = manager.validate_sv(label);
        let sv = manager.get_validated_structural_variant(label, VariantType::Del).unwrap();
        let vkey = StructuralVariant::generate_variant_key(label, "CNTNAP2", crate::dto::structural_variant::SvType::Sv);
        assert_eq!(sv.variant_key(), vkey);
    }   

    #[test]
    fn test_malformed_sv() {
        // Note this test does not make it to the API and thus does not touch the network
        let label = "deletion:c.[6236 + 1_6237–1]_[6432 + 1_6433–1]del";
        let hgnc = "HGNC:15625";
        let symbol = "NBAS";
        let transcript = "NM_015909.4";
        let mut manager = VariantManager::new(symbol, hgnc, transcript);
        let sv = manager.get_validated_structural_variant(label, VariantType::Del);
        let result = manager.validate_sv(label);
        assert!(result.is_err());
        let err_msg = result.unwrap_err();
        assert_eq!( "'deletion:c.[6236\u{2009}+\u{2009}1_6237–1]_[6432\u{2009}+\u{2009}1_6433–1]del': Non-ASCII character '\u{2009}' at index 16", err_msg);
    }


    #[test]
    fn test_intergenic() {
        let symbol = "KLF1";
        let transcript = "NM_006563.5";
        let hgnc = "HGNC:6345";
        let allele = "NC_000019.10:g.12887294G>A";
        let mut manager = VariantManager::new(symbol, hgnc, transcript);
        let result = manager.get_validated_intergenic_hgvs(allele);
        assert!(result.is_ok());
        let ig = result.unwrap();
        println!("{:?}", ig);
    }


}