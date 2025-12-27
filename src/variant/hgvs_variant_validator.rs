//! # VariantValidator Integration
//! 
//! This module provides a high-level interface to the [VariantValidator REST API](https://variantvalidator.org/),
//! specifically designed to validate and normalize HGVS (Human Genome Variation Society) variant strings.
//! 
//! ## Features
//! - **Validation**: Confirms if an HGVS string is mathematically and biologically consistent with the reference genome.
//! - **Normalization**: Maps transcript-level HGVS (c. or n.) to genomic coordinates (g.) and VCF format.
//! - **Memoization**: Uses an internal `HashMap` to cache results, preventing redundant network calls for 
//!   the same variant within a single validator session.
//! - **Error Handling**: Gracefully extracts specific biological validation errors (e.g., reference mismatches) 
//!   from the API response.


use std::{collections::HashMap, mem};

use reqwest::blocking::get;
use serde_json::Value;
use crate::{dto::{hgvs_variant::HgvsVariant, variant_dto::VariantDto}, variant::{variant_validation_handler::VariantValidatorHandler, vcf_var::VcfVar}};

const URL_SCHEME: &str = "https://rest.variantvalidator.org/VariantValidator/variantvalidator/{}/{0}%3A{}/{1}?content-type=application%2Fjson";

const GENOME_ASSEMBLY_HG38: &str = "hg38";

pub struct HgvsVariantValidator {
    genome_assembly: String,
    /// HGVS Variants that could be validated. The key is the original allele denomination (e.g., c.1234A>T), not the variantKey
    validated_hgvs: HashMap<String, HgvsVariant>,
}

fn get_variant_validator_url(
    genome_assembly: &str,
    transcript: &str,
    hgvs: &str
) -> String
{
    let api_url = format!(
        "https://rest.variantvalidator.org/VariantValidator/variantvalidator/{genome}/{transcript}%3A{hgvs}/{transcript}?content-type=application%2Fjson",
        genome = genome_assembly,
        transcript = transcript,
        hgvs = hgvs,
    );
    api_url
}

impl HgvsVariantValidator {
    
    pub fn hg38() -> Self {
        Self {
            genome_assembly: GENOME_ASSEMBLY_HG38.to_string(),
            validated_hgvs: HashMap::new(),
        }
    }

    /// Reach out to the VariantValidator API and create an HgvsVariant object from a transcript and HGVS expression
    /// 
    /// # Arguments
    /// 
    /// * `hgvs` - a Human Genome Variation Society (HGVS) string such as c.123C>T
    /// * `transcript`- the transcript with version number for the HGVS expression
    /// 
    /// # Returns
    /// 
    /// - `Ok(HgvsVariant)` - An object with information about the variant derived from VariantValidator
    /// - `Err(Error)` - An error if the API call fails (which may happen because of malformed input or network issues).
    pub fn validate(
        &mut self, 
        vv_dto: VariantDto
    ) -> Result<(), String>
    {
        let hgvs = &vv_dto.variant_string;
        let allele_key = HgvsVariant::generate_variant_key(hgvs, &vv_dto.gene_symbol, &vv_dto.transcript);
        if self.validated_hgvs.contains_key(&allele_key) {
            return Ok(());
        }
        let url = get_variant_validator_url(&self.genome_assembly, &vv_dto.transcript, hgvs);
        let response: Value = get(&url)
            .map_err(|e| format!("Could not map {hgvs}: {e}"))?
            .json()
            .map_err(|e| format!("Could not parse JSON for {hgvs}: {e}"))?;
        self.extract_variant_validator_warnings(&response)?;

        if let Some(flag) = response.get("flag") {
            if flag != "gene_variant" {
                return Err(format!("Expecting to get a gene_variant but got {}", flag));
            }
        }
        let var = self.get_variant_data(&response)?;
        let hgnc = self.get_hgnc(var)
            .ok_or_else(|| "could not extract hgnc from c_hgvs".to_string())?;
        let symbol = self.get_gene_symbol(var) 
            .ok_or_else(|| "Missing gene_symbol".to_string())?;
        let assembly = self.get_assembly_block(var, &self.genome_assembly)?;
        // The following will either be a String or None, and can be assigned to an Option<String>
        // if we have a non-coding RNA variant, e.g., n.4G>A, then this will evaluate to None
        let p_hgvs = self.get_hgvs_predicted_protein_consequence(var);
        
        
        let hgvs_transcript_var = var.get("hgvs_transcript_variant")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| "Missing field: hgvs_transcript_variant".to_string())?;
        // this field is like NM_000138.5:c.8242G>T - let's just take the first part
        let transcript = hgvs_transcript_var.split(':').next().unwrap_or("");
        let g_hgvs = self.get_genomic_hgvs(assembly)?;
        let vcf_var = self.get_vcf_var(assembly)?;
        
        let hgvs_v = HgvsVariant::new(
            self.genome_assembly.clone(),
            vcf_var, 
            symbol,
            hgnc,
            vv_dto.variant_string,
            p_hgvs,
            transcript.to_string(),
            g_hgvs,
        );
        self.validated_hgvs.insert(hgvs_v.variant_key().clone(), hgvs_v);
        Ok(())
    }

    pub fn get_validated_hgvs(&mut self, vv_dto: &VariantDto) 
    -> Result<HgvsVariant, String> {
        let variant_key = HgvsVariant::generate_variant_key(&vv_dto.variant_string, &vv_dto.gene_symbol, &vv_dto.transcript);
        if let Some(hgvs) = self.validated_hgvs.get(&variant_key) {
            return Ok(hgvs.clone());
        }
       // If not found, validate it. 
      self.validate(vv_dto.clone())?;
      self.validated_hgvs
        .get(&variant_key)
        .cloned()
        .ok_or_else(|| "Internal error: Variant missing after validation".to_string())
    }

    
     /// Take ownership of the map of validated HGVS variants (map is replaced with empty map in the struct)
    pub fn hgvs_map(&mut self) -> HashMap<String, HgvsVariant> {
         mem::take(&mut self.validated_hgvs)
    }

  
}

impl VariantValidatorHandler for HgvsVariantValidator {
}


#[cfg(test)]
mod tests {
    use rstest::{fixture, rstest};

    use super::*;

    // NM_000138.5(FBN1):c.8230C>T (p.Gln2744Ter)
    #[fixture]
    fn vvdto() -> VariantDto {
        VariantDto::hgvs_c(
            "c.8230C>T",
            "NM_000138.5", 
            "HGNC:3603", 
            "FBN1")
    }

    #[fixture]
    fn rnu4_2() -> VariantDto {
        VariantDto::hgvs_c(
            "n.64_65insT",
            "NR_003137.2", 
            "HGNC:10193", 
            "NU4-2")
    }

    /// Invalid version of the above with the wrong nucleotide (G instead of C) 
    /// Designed to elicit an error from VariantValidator
    #[fixture]
    fn invalid_vvdto(mut vvdto: VariantDto) -> VariantDto {
        vvdto.variant_string = "c.8230G>T".to_string();
        vvdto
    }

    #[rstest]
    fn test_url(
        vvdto: VariantDto
    ){
        let expected = "https://rest.variantvalidator.org/VariantValidator/variantvalidator/hg38/NM_000138.5%3Ac.8230C>T/NM_000138.5?content-type=application%2Fjson";
        let my_url = get_variant_validator_url("hg38", &vvdto.transcript, &vvdto.variant_string);
        assert_eq!(expected, my_url);
    }

    #[rstest]
    #[ignore = "runs with API"]
    fn test_variant_validator(vvdto: VariantDto) {
        let mut vvalidator = HgvsVariantValidator::hg38();
        let json = vvalidator.validate(vvdto);
        assert!(json.is_ok());
        let json = json.unwrap();
        println!("{:?}", json);
    }

    #[rstest]
    #[ignore = "runs with API"]
    fn test_variant_validator_invalid(invalid_vvdto: VariantDto) {
        let mut vvalidator = HgvsVariantValidator::hg38();
        // This is an invalid HGVS because the reference base should be C and not G
        let result = vvalidator.validate(invalid_vvdto);
        assert!(result.is_err());
        if let Err(e) = result {
            assert_eq!("NM_000138.5:c.8230G>T: Variant reference (G) does not agree with reference sequence (C)", e);
        } 
    }

    /// Check we can retrieve an n. noncoding variant
    #[rstest]
    #[ignore = "runs with API"]
    fn test_variant_validator_noncoding(rnu4_2: VariantDto) {
        let mut vvalidator = HgvsVariantValidator::hg38();
        // This is an invalid HGVS because the reference base should be C and not G
        let result = vvalidator.validate(rnu4_2);
        assert!(result.is_ok());
        let hgvs = result.unwrap();
        print!("{:?}", hgvs);
    }



}

// endregion: --- Tests