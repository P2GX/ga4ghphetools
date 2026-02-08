use std::{collections::HashMap, mem};

use reqwest::blocking::get;
use serde_json::Value;

use crate::{dto::variant_dto::VariantDto, dto::structural_variant::{StructuralVariant, SvType}};




const GENOME_ASSEMBLY_HG38: &str = "hg38";

const ACCEPTABLE_GENOMES: [&str; 2] = [ "GRCh38",  "hg38"];


pub struct StructuralValidator {
    genome_assembly: String,
    validated_sv: HashMap<String, StructuralVariant>,
}

impl StructuralValidator {
    
    pub fn new(genome_build: &str) -> Result<Self, String> {
        if !ACCEPTABLE_GENOMES.contains(&genome_build) {
            return Err(format!("genome_build \"{}\" not recognized", genome_build));
        }
        Ok(Self {
            genome_assembly: genome_build.to_string(),
            validated_sv: HashMap::new(),
        })
    }

    pub fn hg38() -> Self {
        Self {
            genome_assembly: GENOME_ASSEMBLY_HG38.to_string(),
            validated_sv: HashMap::new(),
        }
    }

    /// We only allow valid ASCII symbols in the labels for the structural variants.
   fn check_ascii(s: &str) -> Result<(), String> {
        for (i, c) in s.char_indices() {
            if !c.is_ascii() {
                return Err(format!("'{}': Non-ASCII character '{}' at index {}", s, c, i));
            }
        }
        Ok(())
    }

    /// Validate and register a symbolic (non-precise) structural variant.
    ///
    /// This method performs basic syntactic and semantic validation of a
    /// structural variant represented by a [`VariantDto`]. The variant is
    /// interpreted as a *symbolic* structural variant (e.g. DEL, DUP, INV,
    /// TRANSLOC) rather than a breakpoint-precise event.
    ///
    /// On successful validation, the variant is converted into a
    /// [`StructuralVariant`] and stored internally as a validated entry.
    ///
    /// # Arguments
    ///
    /// * `vv_dto` – Variant descriptor containing symbolic SV information.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the variant is successfully validated and registered.
    /// * `Err(String)` if validation or conversion fails.
    ///
    /// # Validation steps
    ///
    /// * Ensures the variant string contains only valid ASCII characters.
    /// * Resolves the chromosome from the associated gene symbol.
    /// * Converts the declared variant type into an [`SvType`].
    /// * Constructs the corresponding symbolic [`StructuralVariant`] variant.
    ///
    /// # Errors
    ///
    /// This method returns an error if:
    ///
    /// * the variant string is malformed,
    /// * the chromosome cannot be resolved,
    /// * the variant type is unsupported or inconsistent, or
    /// * construction of the structural variant fails.
    ///
    /// # Side Effects
    ///
    /// On success, the validated structural variant is inserted into the
    /// internal `validated_sv` collection, keyed by its variant identifier.
    ///
    /// # Notes
    ///
    /// * Breakpoint-precise structural variants are not supported by this method.
    /// * The input [`VariantDto`] is consumed during validation.
    pub fn validate(&mut self,  vv_dto: VariantDto) -> Result<(), String> {
            Self::check_ascii(&vv_dto.variant_string)?;
            let chrom = self.get_chromosome_from_vv(&vv_dto.gene_symbol)?;
            let sv_type: SvType = vv_dto.variant_type.try_into()?;
            let sv = match sv_type {
                SvType::Del => StructuralVariant::code_as_chromosomal_deletion(vv_dto, chrom)?,
                SvType::Inv => StructuralVariant::code_as_chromosomal_inversion(vv_dto, chrom)?,
                SvType::Transl => StructuralVariant::code_as_chromosomal_translocation(vv_dto, chrom)?,
                SvType::Dup => StructuralVariant::code_as_chromosomal_duplication(vv_dto, chrom)?,
                SvType::Sv => StructuralVariant::code_as_chromosomal_structure_variation(vv_dto, chrom)?
            };
            self.validated_sv.insert(sv.variant_key().to_string(), sv);
            Ok(())
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
        Ok(chrom.to_string())
    }

    pub fn get_validated_sv(&mut self, vv_dto: &VariantDto) 
    -> Result<StructuralVariant, String> {
        let sv_type = SvType::try_from(vv_dto.variant_type)?;
        let variant_key = StructuralVariant::generate_variant_key(&vv_dto.variant_string, &vv_dto.gene_symbol, sv_type);
        if let Some(sv) = self.validated_sv.get(&variant_key) {
            return Ok(sv.clone());
        }
       // If not found, validate it. 
        self.validate(vv_dto.clone())?;
        self.validated_sv
            .get(&variant_key)
            .cloned()
            .ok_or_else(|| "Internal error: Variant missing after validation".to_string())
    }

    pub fn sv_map(&mut self) -> HashMap<String, StructuralVariant> {
         mem::take(&mut self.validated_sv)
    }

}



#[cfg(test)]
mod tests {
    use rstest::{fixture, rstest};

    use crate::dto::variant_dto::{VariantType};

    use super::*;


    #[fixture]
    fn valid_sv1() -> VariantDto {
        VariantDto{
            variant_string:"arr 16q24.3 DEL89,754,790-89,757,400".to_string(),
            transcript:"NM_052988.5".to_string(),
            hgnc_id:"HGNC:1770".to_string(),
            gene_symbol:"CDK10".to_string(),
            variant_type:VariantType::Del, 
            variant_key: None, 
            is_validated: false, 
            count: 0 
        } 
    }
    
    #[fixture]
    fn invalid_sv1() -> VariantDto {
        VariantDto{ 
            variant_string:"arr 16q24.3 DEL89,754,790 −89,757,400".to_string(), 
            variant_key: None, 
            transcript: "NM_052988.5".to_string(), 
            hgnc_id: "HGNC:1770".to_string(), 
            gene_symbol: "CDK10".to_string(), 
            variant_type: VariantType::Del ,
            is_validated: false, 
            count: 0 
        }
    }

   
    #[rstest]
    #[ignore = "API call"]
    fn test_valid_sv()  {
        let dto = valid_sv1();
        let mut validator = StructuralValidator::hg38();
        let result = validator.validate(dto);
        assert!(result.is_ok());
    }


    #[rstest]
    #[ignore = "API call"]
    fn test_invalid_sv()  {
        let dto = invalid_sv1();
        let mut validator = StructuralValidator::hg38();
        let result = validator.validate(dto);
        assert!(result.is_err());
        let msg = result.err().unwrap();
        let expected = "'arr 16q24.3 DEL89,754,790 −89,757,400': Non-ASCII character '−' at index 26";
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

    #[test]
    #[ignore = "API call"]
    pub fn test_sv_ingest() {
        let expected_chr = "14";
        let validator = StructuralValidator::hg38();
        let chr = validator.get_chromosome_from_vv("HNRNPC");
        assert!(chr.is_ok());
        let chr = chr.unwrap();
        assert_eq!(expected_chr, chr);
    }

    #[test]
    fn validate_sv_complicated() {
        let cell_contents = "NC_000014.9:g.21220392_21352183del(NM_004500.4:c.-82945_366-7272del)";
        let symbol= "HNRNPC";
        let transcript = "NM_004500.4";
        let hgnc = "HGNC:5035";
        let expected_chr = "14";
        let dto = VariantDto{
            variant_string: cell_contents.to_string(),
            variant_key: None,
            transcript: transcript.to_string(),
            hgnc_id:  "HGNC:5035".to_string(),
            gene_symbol: symbol.to_string(),
            variant_type: VariantType::Sv,
            is_validated: false,
            count: 0,
        };
        let mut validator = StructuralValidator::hg38();
        let result = validator.validate(dto);
        assert!(result.is_ok())
    }

    #[rstest]
    fn test_cv() {
        let sv = "Chr9:108,331,353–110,707,332(hg19)";
        let transcript = "NM_021224.6)";
        let hgnc = "HGNC:21684";
        let symbol = "ZNF462";
         let expected_chr = "9";
        let dto = VariantDto{
            variant_string: sv.to_string(),
            variant_key: None,
            transcript: transcript.to_string(),
            hgnc_id:  hgnc.to_string(),
            gene_symbol: symbol.to_string(),
            variant_type: VariantType::Del,
            is_validated: false,
            count: 0,
        };
        let mut validator = StructuralValidator::hg38();
        let result = validator.validate(dto);
        println!("{:?}", result);
        assert!(result.is_err())
    }
    
}

