// This is a Rust port of the Python VariantValidator class
// Dependencies: reqwest = { version = "0.11", features = ["blocking", "json"] }, serde, serde_json, anyhow


use reqwest::blocking::get;
use serde_json::Value;
use crate::{dto::{hgvs_variant::HgvsVariant, variant_dto::VariantDto}, variant::vcf_var::VcfVar};

const URL_SCHEME: &str = "https://rest.variantvalidator.org/VariantValidator/variantvalidator/{}/{0}%3A{}/{1}?content-type=application%2Fjson";

const GENOME_ASSEMBLY_HG38: &str = "hg38";

pub struct HgvsVariantValidator {
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

impl HgvsVariantValidator {
    
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
    pub fn validate(
        &self, 
        vv_dto: VariantDto
    ) -> Result<HgvsVariant, String> 
    {
        let hgvs = &vv_dto.variant_string;
        let transcript = &vv_dto.transcript;
        let url = get_variant_validator_url(&self.genome_assembly, transcript, hgvs);
        let response: Value = get(&url)
            .map_err(|e| format!("Could not map {hgvs}: {e}"))?
            .json()
            .map_err(|e| format!("Could not parse JSON for {hgvs}: {e}"))?;
        Self::extract_variant_validator_warnings(&response)?;

        if let Some(flag) = response.get("flag") {
            if flag != "gene_variant" {
                return Err(format!("Expecting to get a gene_variant but got {}", flag));
            }
        }

        let variant_key = response.as_object()
            .unwrap()
            .keys()
            .find(|&k| k != "flag" && k != "metadata")
            .ok_or_else(|| "Missing variant key".to_string())?;

        let var = &response[variant_key];
        //println!("{}", serde_json::to_string_pretty(var).unwrap());

        let hgnc = var.get("gene_ids")
            .and_then(|ids| ids.get("hgnc_id"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| "Missing hgnc_id".to_string())?;

        let symbol = var.get("gene_symbol")
            .and_then(|s| s.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| "Missing gene_symbol".to_string())?;

        let assemblies = var.get("primary_assembly_loci")
            .ok_or_else(|| "Missing primary_assembly_loci".to_string())?;

        let assembly = assemblies.get(&self.genome_assembly)
            .ok_or_else(|| format!("Could not identify {} in response", self.genome_assembly))?;

        let hgvs_transcript_var = var.get("hgvs_transcript_variant")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| "Missing field: hgvs_transcript_variant".to_string())?;
        // this field is like NM_000138.5:c.8242G>T - let's just take the first part
        let transcript = hgvs_transcript_var.split(':').next().unwrap_or("");
        println!("transcript: {transcript} hgvs var tr {hgvs_transcript_var}");

        let genomic_hgvs = assembly.get("hgvs_genomic_description")
            .and_then(|v| v.as_str())
            .map(str::to_string)
            .ok_or_else(|| "Missing field: hgvs_genomic_description".to_string())?;

        let vcf = assembly.get("vcf")
            .ok_or_else(|| "Could not identify vcf element".to_string())?;
        let chrom: String = vcf.get("chr")
                .and_then(Value::as_str)
                .ok_or_else(|| format!("Malformed chr: {:?}", vcf))? 
                .to_string();
            let position: u32 = vcf.get("pos")
            .and_then(Value::as_str) // "pos" is stored as a string
            .ok_or_else(|| format!("Malformed pos: {:?}", vcf))? 
            .parse() 
            .map_err(|e| format!("Error '{}'", e))?; 
        let reference = vcf.get("ref").
            and_then(Value::as_str)
            .ok_or_else(|| format!("Malformed REF: '{:?}'", vcf))?
            .to_string();
        let alternate = vcf.get("alt").
            and_then(Value::as_str)
            .ok_or_else(|| format!("Malformed ALT: '{:?}'", vcf))?
            .to_string();
        let vcf_var = VcfVar::new(chrom, position, reference, alternate);
        let hgvs_v = HgvsVariant::new(
            self.genome_assembly.clone(),
            vcf_var, 
            symbol,
            hgnc,
            vv_dto.variant_string,
            transcript.to_string(),
            genomic_hgvs,
        );
        Ok(hgvs_v)
    }

    
    fn extract_variant_validator_warnings(response: &Value) -> Result<(), String> {
        if let Some(flag) = response.get("flag").and_then(|f| f.as_str()) {
            if flag == "warning" {
                if let Some(warnings) = response
                    .get("validation_warning_1")
                    .and_then(|v| v.get("validation_warnings"))
                    .and_then(|w| w.as_array())
                {
                    let warning_strings: Vec<String> = warnings
                        .iter()
                        .filter_map(|w| w.as_str().map(|s| s.to_string()))
                        .collect();
                    if let Some(first_warning) = warning_strings.into_iter().next() {
                        return Err(first_warning);
                    } else {
                        // Should never happen, if it does, we need to check parsing of variant validator API.
                        return Err(format!(
                            "[variant_validator: {}:{}] invalid HGVS",
                            file!(),
                            line!()
                        ));
                    }
                }
            }
        }
        Ok(())
    }

  
}


// region:    --- Tests

#[cfg(test)]
mod tests {
    use rstest::{fixture, rstest};

    use super::*;

    // NM_000138.5(FBN1):c.8230C>T (p.Gln2744Ter)
    #[fixture]
    fn vvdto() -> VariantValidationDto {
        VariantValidationDto::hgvs_c(
            "c.8230C>T",
            "NM_000138.5", 
            "HGNC:3603", 
            "FBN1")
    }

    /// Invalid version of the above with the wrong nucleotide (G instead of C) 
    /// Designed to elicit an error from VariantValidator
    #[fixture]
    fn invalid_vvdto(mut vvdto: VariantValidationDto) -> VariantValidationDto {
        vvdto.variant_string = "c.8230G>T".to_string();
        vvdto
    }

    #[rstest]
    fn test_url(
        vvdto: VariantValidationDto
    ){
        let expected = "https://rest.variantvalidator.org/VariantValidator/variantvalidator/hg38/NM_000138.5%3Ac.8230C>T/NM_000138.5?content-type=application%2Fjson";
        let my_url = get_variant_validator_url("hg38", &vvdto.transcript, &vvdto.variant_string);
        assert_eq!(expected, my_url);
    }

    #[rstest]
    #[ignore = "runs with API"]
    fn test_variant_validator(vvdto: VariantValidationDto) {
        let vvalidator = HgvsVariantValidator::hg38();
        let json = vvalidator.validate(vvdto);
        assert!(json.is_ok());
        let json = json.unwrap();
        println!("{:?}", json);
    }

    #[rstest]
    #[ignore = "runs with API"]
    fn test_variant_validator_invalid(invalid_vvdto: VariantValidationDto) {
        let vvalidator = HgvsVariantValidator::hg38();
        // This is an invalid HGVS because the reference base should be C and not G
        let result = vvalidator.validate(invalid_vvdto);
        assert!(result.is_err());
        if let Err(e) = result {
            assert_eq!("NM_000138.5:c.8230G>T: Variant reference (G) does not agree with reference sequence (C)", e);
        } 
    }
}

// endregion: --- Tests