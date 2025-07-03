// src/variant/hgvs_variant.rs

use ontolius::term::simple::SimpleMinimalTerm;
use rand::{self, distr::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};

use crate::variant::acmg::AcmgPathogenicityClassification;
use crate::variant::variant_trait::Variant;
use crate::{error::Error, variant::vcf_var::{self, VcfVar}};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HgvsVariant {
    assembly: String,
    chr: String,
    position: u32,
    ref_allele: String,
    alt_allele: String,
    symbol: Option<String>,
    hgnc_id: Option<String>,
    hgvs: Option<String>,
    transcript: Option<String>,
    g_hgvs: Option<String>,
    genotype: Option<String>,
    variant_id: String,
}

impl HgvsVariant {
    pub fn new(
        assembly: String,
        vcf_var: VcfVar,
        symbol: Option<String>,
        hgnc_id: Option<String>,
        hgvs: Option<String>,
        transcript: Option<String>,
        g_hgvs: Option<String>,
        variant_id: Option<String>,
    ) -> Self {
        let chr = vcf_var.chrom();
        let pos = vcf_var.pos();
        let ref_allele = vcf_var.ref_allele();
        let alt_allele = vcf_var.alt_allele();

        let variant_id = variant_id.unwrap_or_else(|| {
            let rand_str: String = rand::rng()
                .sample_iter(&Alphanumeric)
                .take(25)
                .map(char::from)
                .collect();
            format!("var_{}", rand_str)
        });

        HgvsVariant {
            assembly,
            chr,
            position: pos,
            ref_allele,
            alt_allele,
            symbol,
            hgnc_id,
            hgvs,
            transcript,
            g_hgvs,
            genotype: None,
            variant_id,
        }
    }

    pub fn assembly(&self) -> &str {
        self.assembly.as_ref()
    }

    pub fn chr(&self) -> &str {
        self.chr.as_ref()
    }

    pub fn position(&self) -> u32 {
        self.position
    }

    pub fn ref_allele(&self) -> &str {
        self.ref_allele.as_ref()
    }

    pub fn alt_allele(&self) -> &str {
        self.alt_allele.as_ref()
    }

    pub fn symbol(&self) -> Option<&str> {
        self.symbol.as_deref()
    }

    pub fn hgnc_id(&self) -> Option<&str> {
        self.hgnc_id.as_deref()
    }

    pub fn hgvs(&self) -> Option<&str> {
        self.hgvs.as_deref()
    }

    pub fn transcript(&self) -> Option<&str> {
        self.transcript.as_deref()
    }

    pub fn g_hgvs(&self) -> Option<&str> {
        self.g_hgvs.as_deref()
    }
    pub fn genotype(&self) ->  Option<&str> {
        self.genotype.as_deref()
    }

    pub fn variant_id(&self) ->  &str {
        self.variant_id.as_ref()
    }




}

impl Variant for HgvsVariant {
    fn set_heterozygous(&mut self) {
        self.genotype = Some("heterozygous".to_string())
    }
    
    fn set_homozygous(&mut self) {
        self.genotype = Some("homozygous".to_string())
    }
    
    fn set_hemizygous(&mut self) {
        self.genotype = Some("hemizygous".to_string())
    }
}



#[cfg(test)]
mod tests {

    use crate::{error::Error, variant::variant_validator::VariantValidator};
    use super::*;
    use rstest::rstest;

    // test NM_000138.5(FBN1):c.8242G>T (p.Glu2748Ter)
    // We expect to get this back
    // HgvsVariant { assembly: "hg38", chr: "chr15", position: 48411364, ref_allele: "C", alt_allele: "A", symbol: Some("FBN1"), 
    // hgnc_id: Some("HGNC:3603"), hgvs: Some("NM_000138.5"), transcript: Some("NM_000138.5:c.8242G>T"), 
    // g_hgvs: Some("NC_000015.10:g.48411364C>A"), genotype: None, variant_id: "var_JgacXpZdmwKjarf125ud6ILjA" }
    #[rstest]
    #[ignore = "testing API"]
    fn test_hgvs_c_fbn1() {
        let vvalidator = VariantValidator::new("hg38").unwrap();
        let result = vvalidator.encode_hgvs("c.8242G>T", "NM_000138.5");
        assert!(result.is_ok());
        let hgvs_var = result.unwrap();
        println!("{:?}", hgvs_var);
        assert_eq!("hg38", hgvs_var.assembly());
        assert_eq!("chr15", hgvs_var.chr());
        assert_eq!(48411364, hgvs_var.position());
        assert_eq!("C", hgvs_var.ref_allele());
        assert_eq!("A", hgvs_var.alt_allele());
        assert_eq!(Some("FBN1"), hgvs_var.symbol());
        assert_eq!(Some("HGNC:3603"), hgvs_var.hgnc_id());
        assert_eq!(Some("NM_000138.5:c.8242G>T"), hgvs_var.transcript());
        assert_eq!(Some("NC_000015.10:g.48411364C>A"), hgvs_var.g_hgvs());
        assert!(hgvs_var.genotype().is_none()); // the variant validator call does not set the genotype
    }


}