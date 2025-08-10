use reqwest::blocking::get;
use serde_json::Value;

use crate::{dto::{cohort_dto::CohortDto, variant_dto::{VariantValidationDto}}, variant::structural_variant::{StructuralVariant, SvType}};




const GENOME_ASSEMBLY_HG38: &str = "hg38";

const ACCEPTABLE_GENOMES: [&str; 2] = [ "GRCh38",  "hg38"];


pub struct StructuralValidator {
    genome_assembly: String,
}

impl StructuralValidator {
    
    pub fn new(genome_build: &str) -> Result<Self, String> {
        if !ACCEPTABLE_GENOMES.contains(&genome_build) {
            return Err(format!("genome_build \"{}\" not recognized", genome_build));
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

    /// Validate a structural variant (symbolic, non-precise)
    /// If successful, add the StructuralVariant object to the cohort_dto, otherwise return an error
    /// Calling code should update the cohort dto in the front end if successful
    pub fn validate_sv(&self,  vv_dto: VariantValidationDto) ->
        Result<StructuralVariant, String> {
            let chrom = self.get_chromosome_from_vv(&vv_dto.gene_symbol)?;
            let sv_type: SvType = vv_dto.validation_type.try_into()?;
            match sv_type {
                SvType::Del => StructuralVariant::code_as_chromosomal_deletion(vv_dto, chrom),
                SvType::Inv => StructuralVariant::code_as_chromosomal_inversion(vv_dto, chrom),
                SvType::Transl => StructuralVariant::code_as_chromosomal_translocation(vv_dto, chrom),
                SvType::Dup => StructuralVariant::code_as_chromosomal_duplication(vv_dto, chrom),
                SvType::Sv => StructuralVariant::code_as_chromosomal_structure_variation(vv_dto, chrom)
            }
    }


    /// We want to have a little more information about the structural variant
    /// We will assume that there is always a gene that is in focus (because for this
    /// application, we are creating gene-based cohorts, so even if we have a translocation between
    /// two chromosomes, we will always have a "main" gene). 
    pub fn get_chromosome_from_vv(&self, gene: &str) -> Result<String, String> {
        // https://rest.variantvalidator.org/VariantValidator/tools/gene2transcripts/COL1A1
        let api_url = format!(
            "https://rest.variantvalidator.org/VariantValidator/tools/gene2transcripts/{gene}?content-type=application%2Fjson",
        );
        let response: Value = get(&api_url)
                .map_err(|e| format!("Could not map {gene}: {e}"))?
                .json()
                .map_err(|e| format!("Could not retrieve JSON for {gene}: {e}"))?;
        let transcripts = response
            .get("transcripts")
            .and_then(|t| t.as_array())
            .ok_or_else(|| "Missing transcripts in structural".to_string())?;
        if transcripts.is_empty() {
            return Err(format!("Transcript array was empty in VariantValidator response for {gene}"));
        }
        let transcript1 = transcripts.get(0)
            .ok_or_else(|| format!("Could not extract first transcript in non-empty transcript array from VariantValidator for {gene}"))?;
        let annotations = transcript1
            .get("annotations")
            .and_then(|a| a.as_object())
            .ok_or_else(|| format!(
                "Missing or invalid 'annotations' in transcript for {gene}"
            ))?;
        let chrom = annotations
            .get("chromosome")
            .and_then(|c| c.as_str())
            .ok_or_else(|| format!(
                "Could not extract chromosome from annotations map (VariantValidator) for '{gene}'"
            ))?;
        return Ok(chrom.to_string());
    }

}



#[cfg(test)]
mod tests {
    use clap::builder::Str;
    use rstest::{fixture, rstest};

    use crate::dto::variant_dto::{VariantValidationType};

    use super::*;


    #[fixture]
    fn valid_sv1() -> VariantValidationDto {
        VariantValidationDto{ 
            variant_string:"arr 16q24.3 DEL89,754,790 −89,757,400".to_string(), 
            transcript: "NM_052988.5".to_string(), 
            hgnc_id: "HGNC:1770".to_string(), 
            gene_symbol: "CDK10".to_string(), 
            validation_type: VariantValidationType::Del 
        }
    }
    
    #[fixture]
    fn invalid_sv1() -> VariantValidationDto {
        VariantValidationDto{ 
            variant_string:"arr 16q24.3 DEL89,754,790 −89,757,400".to_string(), 
            transcript: "NM_052988.5".to_string(), 
            hgnc_id: "HGNC:1770".to_string(), 
            gene_symbol: "CDK10".to_string(), 
            validation_type: VariantValidationType::Del 
        }
    }

   
    #[rstest]
    fn test_valid_sv()  {
        let dto = valid_sv1();
        let validator = StructuralValidator::hg38();
        let result = validator.validate_sv(dto);
        assert!(result.is_ok());
    }


    #[rstest]
    fn test_invalid_sv()  {
        let dto = invalid_sv1();
        let validator = StructuralValidator::hg38();
        let result = validator.validate_sv(dto);
        assert!(result.is_err());
        let msg = result.err().unwrap();
        let expected = "Variant string arr 16q24.3 DEL89,754,790 −89,757,400 contains non-ASCII character";
        assert_eq!(expected, msg);
    }

      #[rstest]
      #[ignore = "API call"]
      fn test_extract_chromosome17() {
        let expected_chr = "17";
         let validator = StructuralValidator::hg38();
        let chr = validator.get_chromosome_from_vv("COL1A1");
        assert!(chr.is_ok());
        let chr = chr.unwrap();
        assert_eq!(expected_chr, chr);
      }

       #[rstest]
       #[ignore = "API call"]
      fn test_extract_chromosome_x() {
        let expected_chr = "X";
         let validator = StructuralValidator::hg38();
        let chr = validator.get_chromosome_from_vv("FMR1");
        assert!(chr.is_ok());
        let chr = chr.unwrap();
        assert_eq!(expected_chr, chr);
      }

      
    
}

