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
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use ontolius::io::OntologyLoaderBuilder;
use ontolius::ontology::csr::FullCsrOntology;

use crate::dto::cohort_dto::CohortDto;


use crate::dto::variant_dto::{VariantValidationDto, VariantValidationType};
use crate::dto::hgvs_variant::HgvsVariant;
use crate::ppkt::ppkt_row::PpktRow;
use crate::template::cohort_dto_builder::CohortDtoBuilder;
use crate::template::excel;
use crate::template::header_duplet_row::HeaderDupletRow;
use crate::variant::structural_validator::StructuralValidator;
use crate::{dto::variant_dto::VariantDto, variant::hgvs_variant_validator::HgvsVariantValidator};


use crate::dto::structural_variant::{StructuralVariant, SvType};



/// This struct validates variants sent from the front end. It evaluates either HGVS or symbolic (imprecise)
/// structural variants. If a variant is validated, then the CohortTemplate is sent back to the front end
/// with the new variant structure, otherwise errors are returned
pub struct VariantManager {
    hgvs_validator: HgvsVariantValidator,
    structural_validator: StructuralValidator,
    gene_symbol: String,
    hgnc_id: String,
    transcript: String,
    unvalidated_hgvs: HashSet<String>,
    unvalidated_sv: HashSet<String>,
    validated_hgvs: HashMap<String, HgvsVariant>,
    validated_sv: HashMap<String, StructuralVariant>
}




impl VariantManager {
    pub fn new(symbol: &str, hgnc: &str, transcript: &str) -> Self {
        Self {
            hgvs_validator: HgvsVariantValidator::hg38(),
            structural_validator: StructuralValidator::hg38(),
            gene_symbol: symbol.to_string(),
            hgnc_id: hgnc.to_string(),
            transcript: transcript.to_string(),
            unvalidated_hgvs: HashSet::new(),
            unvalidated_sv: HashSet::new(),
            validated_hgvs: HashMap::new(),
            validated_sv: HashMap::new(),
        }
    }


    pub fn excel_template_to_matrix(
        phetools_template_path: &str,
    ) -> Result<Vec<Vec<String>>, String> 
    {
        excel::read_excel_to_dataframe(phetools_template_path)
    }

    pub fn validate_all_variants(&mut self, all_alleles: &HashSet<String>) -> Result<(), String>{
        let n_alleles = all_alleles.len();
        let mut attempts = 0;
        let max_attempts = 4;
        let mut latency = 1 as u64; // time to wait between API calls
        let mut n_validated = 0;
        while n_validated < n_alleles && attempts < max_attempts {
            n_validated = 0;
            let validated_hgvs = self.validate_all_hgvs_variants(all_alleles, latency);
            let validated_sv = self.validate_all_sv(all_alleles, latency);
            n_validated = validated_hgvs + validated_sv;
            latency += 1;
            attempts += 1;
            println!("Round {}: validated: {} (HGVS: {}, SV: {})", attempts, n_validated, validated_hgvs, validated_sv);
        }
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
    pub fn get_variant_key(&self, allele: &str) -> String {
        if let Some(hgvs) = self.validated_hgvs.get(allele) {
            return hgvs.variant_key();
        }
        if let Some(sv) = self.validated_sv.get(allele) {
            return sv.variant_key().to_string();
        }
        "na".to_string()
    }


    pub fn validate_all_hgvs_variants(&mut self, variants: &HashSet<String>, latency: u64) -> usize {
        let mut n_valid = 0 as usize;
        for v in variants {
            if ! v.starts_with("c.") && ! v.starts_with("n.") {
                eprint!("error: attempt to HGVS validate non-HGVS variant {}", v);
                continue;
            }
            let vv_dto = VariantValidationDto::hgvs_c(v, &self.transcript, &self.hgnc_id, &self.gene_symbol);
            let variant_key = HgvsVariant::generate_variant_key(v, &self.gene_symbol, &self.transcript);
            if self.validated_hgvs.contains_key(&variant_key) {
                println!("Previously validated {}", variant_key);
                n_valid += 1;
                continue;
            }
            if let Ok(hgvs) = self.hgvs_validator.validate(vv_dto) {
                self.validated_hgvs.insert(variant_key, hgvs.clone());
                n_valid += 1;
            } else {
                eprint!("Could not validate {v}/{variant_key}");
            }
            // sleep to try to avoid network issues; we have a lot of time, so let's sleep for 2 seconds!
            thread::sleep(Duration::from_secs(latency));
        }
        n_valid
    }


    pub fn validate_all_sv(&mut self, variants: &HashSet<String>,  latency: u64) -> usize {
        let mut n_valid = 0 as usize;
         for v in variants {
            let vv_dto = VariantValidationDto::sv(v, &self.transcript, &self.hgnc_id, &self.gene_symbol);
            let sv_type = SvType::try_from(vv_dto.validation_type);
            if sv_type.is_err() {
                eprint!("Could not extract SvType from variant {v}");
                continue;
            }
            let sv_type = sv_type.unwrap();
            let variant_key = StructuralVariant::generate_variant_key(v, &self.gene_symbol, sv_type);
            if self.validated_sv.contains_key(&variant_key) {
                println!("Previously validated {}", variant_key);
                n_valid += 1;
                continue;
            }
            if let Ok(sv) = self.structural_validator.validate(vv_dto) {
                self.validated_sv.insert(variant_key, sv.clone());
                n_valid += 1;
            } else {
                eprint!("Could not validate {v}/{variant_key}");
            }
            // sleep to try to avoid network issues; we have a lot of time, so let's sleep for 2 seconds!
            thread::sleep(Duration::from_secs(latency));

         }

        n_valid
    }


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


    pub fn get_allele_set(ppt_rows: &Vec<PpktRow>) -> HashSet<String> {
       let mut allele_set = HashSet::new();
        for row in ppt_rows {
            for gv in  row.get_gene_var_dto_list() {
                if !gv.allele1_is_present() {
                    allele_set.insert(gv.allele1.clone());
                }
                if gv.allele2_is_present() {
                    allele_set.insert(gv.allele2);
                }
            }
        }
        allele_set
    }

     pub fn from_mendelian_template_test(
        &mut self,
        matrix: Vec<Vec<String>>,
        hpo: Arc<FullCsrOntology>,
    ) -> std::result::Result<(), String> {
        let fix_errors = true;
        let header = HeaderDupletRow::mendelian(&matrix, hpo.clone(), fix_errors)?;
        //println!("{:?}", matrix);
        const HEADER_ROWS: usize = 2; // first two rows of template are header
        let hdr_arc = Arc::new(header);
        let mut ppt_rows: Vec<PpktRow> = Vec::new();
        let dg_dto = CohortDtoBuilder::get_disease_dto_from_excel(&matrix)?;
        for row in matrix.into_iter().skip(HEADER_ROWS) {
            let hdr_clone = hdr_arc.clone();
            let ppkt_row = PpktRow::from_row(hdr_clone, row)?;
            ppt_rows.push(ppkt_row);
        }
        println!("We got {} ppklt rows", ppt_rows.len());
        let allele_set = Self::get_allele_set(&ppt_rows);
        self.validate_all_variants(&allele_set);
        Ok(())
    }

    pub fn from_path(&mut self, tpl_path: &str)-> std::result::Result<(), String> {
        let matrix = Self::excel_template_to_matrix(tpl_path)?;
        let hp_json = "/Users/robin/data/hpo/hp.json";
        let loader = OntologyLoaderBuilder::new().obographs_parser().build();
            let ontology: FullCsrOntology = loader
                            .load_from_path(hp_json)
                            .expect("Could not load {file_path}");
        let hpo = Arc::new(ontology);
        self.from_mendelian_template_test(matrix, hpo)?;
        Ok(())
    }


    pub fn validate_variant_dto_list(&mut self, variant_dto_list: Vec<VariantDto>) -> Result<Vec<VariantDto>, String> {
       /*  let mut evaluated_dto_list: Vec<VariantDto> = Vec::with_capacity(variant_dto_list.len());
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
        Ok(evaluated_dto_list)*/
        Err("refactoring".to_ascii_lowercase())
    }





}



#[cfg(test)]
mod tests {
    use rstest::{rstest};

    use super::*;

    /// 	ATP6V0C	
    #[rstest]
    fn test_check_all_vars() {
        let template = "/Users/robin/GIT/phenopacket-store/notebooks/ATP6V0C/input/ATP6V0C_EPEO3_individuals.xlsx";
        let mut vmanager = VariantManager::new( "ATP6V0C","HGNC:855", "NM_001694.4");
        vmanager.from_path(template).unwrap();
        
    }

}