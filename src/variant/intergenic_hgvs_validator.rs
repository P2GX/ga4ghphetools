

use reqwest::blocking::get;
use serde_json::Value;
use crate::{dto::{hgvs_variant::HgvsVariant, intergenic_variant::IntergenicHgvsVariant, variant_dto::VariantDto}, variant::{variant_validation_handler::VariantValidatorHandler, vcf_var::VcfVar}};



const BASE_URL: &str = "https://rest.variantvalidator.org/VariantValidator/variantvalidator";




const GENOME_ASSEMBLY_HG38: &str = "hg38";

pub struct IntergenicHgvsValidator {
    genome_assembly: String,
}

/// Construct the VariantValidator request from the genome assembly and the g.HGVS
fn get_variant_validator_url(
    genome_assembly: &str,
    hgvs: &str
) -> String
{
    let encoded_hgvs = urlencoding::encode(&hgvs);
    

    let full_url = format!("{}/{}/{}/mane?content-type=application%2Fjson", 
        BASE_URL, 
        genome_assembly, 
        encoded_hgvs
    );
    println!("{}",full_url);
    
    full_url
}


impl IntergenicHgvsValidator {
    
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
    ) -> Result<IntergenicHgvsVariant, String> 
    {
        let hgvs = &vv_dto.variant_string;
        let url = get_variant_validator_url(&self.genome_assembly, hgvs);
        let response: Value = get(&url)
            .map_err(|e| format!("Could not map {hgvs}: {e}"))?
            .json()
            .map_err(|e| format!("Could not parse JSON for {hgvs}: {e}"))?;
        self.from_json(response)
    }


    pub fn from_json(&self, response: Value) -> Result<IntergenicHgvsVariant, String> {
        self.extract_variant_validator_warnings(&response)?;
        if let Some(flag) = response.get("flag") {
            if flag != "intergenic" {
                return Err(format!("Expecting to get an intergenic variant but got {}", flag));
            }
        }

        let var = self.get_variant_data(&response)?;
        //println!("{}", serde_json::to_string_pretty(var).unwrap());
        let assembly = self.get_assembly_block(var, &self.genome_assembly)?;
        let g_hgvs = self.get_genomic_hgvs(assembly)?;
        let vcf_var = self.get_vcf_var(assembly)?;
        let gene_symbol = self.get_gene_symbol(var); // Option because we might not get one
        let hgnc = self.get_hgnc(var);
        let gene_hgvs = var.get("hgvs_refseqgene_variant")
            .and_then(|s| s.as_str())
            .map(|s| s.to_string());
        let intergenic_v = IntergenicHgvsVariant::new(
            self.genome_assembly.clone(),
            vcf_var, 
            gene_symbol,
            hgnc,
            g_hgvs,
            gene_hgvs,
        );
        Ok(intergenic_v)

    }


    

}

impl VariantValidatorHandler for IntergenicHgvsValidator {}



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

    #[fixture]
    fn vv_response() -> String {
        let response = 
r#"{
  "flag": "intergenic",
  "intergenic_variant_1": {
    "alt_genomic_loci": [],
    "annotations": {},
    "gene_ids": {
      "ccds_ids": [
        "CCDS12286"
      ],
      "ensembl_gene_id": "ENSG00000105607",
      "entrez_gene_id": "2639",
      "hgnc_id": "HGNC:4189",
      "omim_id": [
        "608801"
      ],
      "ucsc_id": "uc002mvq.5"
    },
    "gene_symbol": "GCDH",
    "genome_context_intronic_sequence": "",
    "hgvs_lrg_transcript_variant": "",
    "hgvs_lrg_variant": "",
    "hgvs_predicted_protein_consequence": {
      "lrg_slr": "",
      "lrg_tlr": "",
      "slr": "",
      "tlr": ""
    },
    "hgvs_refseqgene_variant": "NG_009292.1:g.1135G>A",
    "hgvs_transcript_variant": "",
    "lovd_corrections": null,
    "lovd_messages": null,
    "primary_assembly_loci": {
      "grch37": {
        "hgvs_genomic_description": "NC_000019.9:g.12998108G>A",
        "vcf": {
          "alt": "A",
          "chr": "19",
          "pos": "12998108",
          "ref": "G"
        }
      },
      "grch38": {
        "hgvs_genomic_description": "NC_000019.10:g.12887294G>A",
        "vcf": {
          "alt": "A",
          "chr": "19",
          "pos": "12887294",
          "ref": "G"
        }
      },
      "hg19": {
        "hgvs_genomic_description": "NC_000019.9:g.12998108G>A",
        "vcf": {
          "alt": "A",
          "chr": "chr19",
          "pos": "12998108",
          "ref": "G"
        }
      },
      "hg38": {
        "hgvs_genomic_description": "NC_000019.10:g.12887294G>A",
        "vcf": {
          "alt": "A",
          "chr": "chr19",
          "pos": "12887294",
          "ref": "G"
        }
      }
    },
    "reference_sequence_records": {
      "refseqgene": "https://www.ncbi.nlm.nih.gov/nuccore/NG_009292.1"
    },
    "refseqgene_context_intronic_sequence": "",
    "rna_variant_descriptions": null,
    "selected_assembly": "hg38",
    "submitted_variant": "NC_000019.10:g.12887294G>A",
    "transcript_description": "",
    "validation_warnings": [
      "No individual transcripts have been identified that fully overlap the described variation in the genomic sequence. Large variants might span one or more genes and are currently only described at the genome (g.) level."
    ],
    "variant_exonic_positions": null
  },
  "metadata": {
    "variantvalidator_hgvs_version": "2.2.1.dev17+gd620dd190",
    "variantvalidator_version": "3.0.2.dev143+g6213c80fe",
    "vvdb_version": "vvdb_2025_3",
    "vvseqrepo_db": "VV_SR_2025_02/master",
    "vvta_version": "vvta_2025_02"
  }
}"#.to_string();
response
}



    #[rstest]
    fn test_url(
        vvdto: VariantDto
    ){
        let intergenic = "NC_000019.10:g.12887294G>A";
        let expected = "https://rest.variantvalidator.org/VariantValidator/variantvalidator/hg38/NC_000019.10%3Ag.12887294G%3EA/mane?content-type=application%2Fjson";
        let my_url = get_variant_validator_url("hg38",  intergenic);
        assert_eq!(expected, my_url);
    }


    #[rstest]
    fn test_decode(
        vv_response: String
    ) {
        let validator = IntergenicHgvsValidator::hg38();
        let json_value: serde_json::Value = serde_json::from_str(&vv_response)
            .expect("Fixture should be valid JSON");
        let result = validator.from_json(json_value);
        assert!(result.is_ok());
        let intergen = result.unwrap();
        println!("{:?}", intergen);
        assert_eq!("hg38", intergen.assembly());
        assert_eq!("chr19", intergen.chr());
        assert_eq!(12887294, intergen.position());
        assert_eq!("G", intergen.ref_allele());
        assert_eq!("A", intergen.alt_allele());
        assert!(intergen.symbol().is_some());
        assert_eq!("GCDH", intergen.symbol().unwrap());
        assert!(intergen.hgnc_id().is_some());
        assert_eq!("HGNC:4189", intergen.hgnc_id().unwrap());
        assert_eq!("NC_000019.10:g.12887294G>A", intergen.g_hgvs());
        assert!(intergen.gene_hgvs().is_some());
        assert_eq!("NG_009292.1:g.1135G>A", intergen.gene_hgvs().unwrap());
        let expected_var_key= "NC_000019_10_g_12887294GtoA";
        assert_eq!(expected_var_key, intergen.variant_key());

    }



}