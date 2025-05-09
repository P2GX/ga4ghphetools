// This is a Rust port of the Python VariantValidator class
// Dependencies: reqwest = { version = "0.11", features = ["blocking", "json"] }, serde, serde_json, anyhow

use std::collections::HashMap;
use std::convert::TryInto;
use anyhow::{Result, bail};
use polars::series::implementations;
use reqwest::blocking::get;
use serde_json::Value;
use crate::{error::Error, variant::{hgvs_variant::HgvsVariant, vcf_var::{self, VcfVar}}};

const URL_SCHEME: &str = "https://rest.variantvalidator.org/VariantValidator/variantvalidator/{}/{0}%3A{}/{1}?content-type=application%2Fjson";

const GENOME_ASSEMBLY_HG38: &str = "hg38";

const ACCEPTABLE_GENOMES: [&str; 2] = [ "GRCh38",  "hg38"];

impl Error {
    pub fn malformed_chr(vcf: &Value) -> Self
    {
        Self::VcfError { msg: format!("Missing or invalid 'chrom' in VCF object: {vcf}"),}
    } 
    pub fn malformed_ref(vcf: &Value) -> Self
    {
        Self::VcfError { msg: format!("Missing or invalid 'REF' in VCF object: {vcf}"),}
    } 

    pub fn malformed_alt(vcf: &Value) -> Self
    {
        Self::VcfError { msg: format!("Missing or invalid 'ALT' in VCF object: {vcf}"),}
    } 

    pub fn malformed_pos(vcf: &Value) -> Self
    {
        Self::VcfError { msg: format!("Missing or invalid 'POS' in VCF object: {vcf}"),}
    } 
}

pub struct VariantValidator {
    genome_assembly: String,
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

impl VariantValidator {
    pub fn new(genome_build: &str) -> Result<Self> {
        if !ACCEPTABLE_GENOMES.contains(&genome_build) {
            bail!("genome_build \"{}\" not recognized", genome_build);
        }
        Ok(Self {
            genome_assembly: genome_build.to_string(),
        })
    }

    pub fn hg38() -> Self {
        Self {
            genome_assembly: GENOME_ASSEMBLY_HG38.to_string(),
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
    pub fn encode_hgvs(
        &self, 
        hgvs: &str, 
        transcript: &str
    ) -> Result<HgvsVariant> 
    {
        let url = get_variant_validator_url(&self.genome_assembly, transcript, hgvs);
        let response: Value = get(&url)?.json()?;

        if let Some(flag) = response.get("flag") {
            if flag != "gene_variant" {
                let msg = format!("Expecting to get a gene_variant but got {}", flag);
                bail!(Error::VcfError{msg});
            }
        }

        let variant_key = response.as_object()
            .unwrap()
            .keys()
            .find(|&k| k != "flag" && k != "metadata")
            .ok_or_else(|| anyhow::anyhow!("Missing variant key"))?;

        let var = &response[variant_key];

        let hgnc = var.get("gene_ids")
            .and_then(|ids| ids.get("hgnc_id"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let symbol = var.get("gene_symbol")
            .and_then(|s| s.as_str())
            .map(|s| s.to_string());

        let assemblies = var.get("primary_assembly_loci")
            .ok_or_else(|| anyhow::anyhow!("Missing primary_assembly_loci"))?;

        let assembly = assemblies.get(&self.genome_assembly)
            .ok_or_else(|| anyhow::anyhow!("Could not identify {} in response", self.genome_assembly))?;

        let hgvs_transcript_var = var.get("hgvs_transcript_variant")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let genomic_hgvs = assembly.get("hgvs_genomic_description")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let transcript = var.get("reference_sequence_records")
            .and_then(|r| r.get("transcript"))
            .and_then(|t| t.as_str())
            .map(|t| {
                if t.starts_with("https://www.ncbi.nlm.nih.gov/nuccore/") {
                    t[37..].to_string()
                } else {
                    t.to_string()
                }
            });

        let vcf = assembly.get("vcf")
            .ok_or_else(|| anyhow::anyhow!("Could not identify vcf element"))?;
        let chrom: String = vcf.get("chr")
                .and_then(Value::as_str)
                .ok_or_else(|| Error::malformed_chr(vcf))?
                .to_string();
            let position: u32 = vcf.get("pos")
            .and_then(Value::as_str) // "pos" is stored as a string
            .ok_or_else(|| Error::VcfError{msg: format!("Malformed pos: {:?}", vcf)})? 
            .parse() 
            .map_err(|e| Error::VcfError { msg: format!("{}", e) })?; 
        let reference = vcf.get("ref").
            and_then(Value::as_str)
            .ok_or_else(|| Error::malformed_ref(vcf))?
            .to_string();
        let alternate = vcf.get("alt").
            and_then(Value::as_str)
            .ok_or_else(|| Error::malformed_alt(vcf))?
            .to_string();
        let vcf_var = VcfVar::new(chrom, position, reference, alternate);
        let hgvs_v = HgvsVariant::new(
            self.genome_assembly.clone(),
            vcf_var, 
            symbol,
            hgnc,
            transcript,
            hgvs_transcript_var,
            genomic_hgvs,
            None,
        );
        return Ok(hgvs_v);
    }
}


// region:    --- Tests

#[cfg(test)]
mod tests {
  
    use super::*;

    #[test]
    fn test_url()  {
        // NM_000138.5(FBN1):c.8230C>T (p.Gln2744Ter)
        let expected = "https://rest.variantvalidator.org/VariantValidator/variantvalidator/hg38/NM_000138.5%3Ac.8230C>T/NM_000138.5?content-type=application%2Fjson";
        let my_url = get_variant_validator_url("hg38", "NM_000138.5", "c.8230C>T");
        assert_eq!(expected, my_url);
    }

    #[test]
    #[ignore = "runs with API"]
    fn test_variant_validator() {
        let vvalidator = VariantValidator::new("hg38").unwrap();
        let json = vvalidator.encode_hgvs("c.8230C>T", "NM_000138.5");
        assert!(json.is_ok());
        let json = json.unwrap();
        println!("{:?}", json);
    }
}

// endregion: --- Tests