// src/variant/hgvs_variant.rs

use rand::{self, distr::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};


use crate::variant::vcf_var::VcfVar;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HgvsVariant {
    /// Genome build, e.g., hg38
    assembly: String,
    /// Chromosome, e.g., "17"
    chr: String,
    /// Position on the chromosome
    position: u32,
    /// Reference allele
    ref_allele: String,
    /// Alternate allele
    alt_allele: String,
    /// Gene symbol, e.g., FBN1
    symbol: String,
    /// HUGO Gene Nomenclature Committee identifier, e.g., HGNC:3603
    hgnc_id: String,
    /// HGVS Nomenclature, e.g., c.8242G>T
    hgvs: String,
    /// Transcript, e.g., NM_000138.5
    transcript: String,
    /// Genomic HGVS nomenclature, e.g., NC_000015.10:g.48411364C>A
    g_hgvs: String,
}

impl HgvsVariant {
    pub fn new(
        assembly: String,
        vcf_var: VcfVar,
        symbol: String,
        hgnc_id: String,
        hgvs: String,
        transcript: String,
        g_hgvs: String,
    ) -> Self {
        let chr = vcf_var.chrom();
        let pos = vcf_var.pos();
        let ref_allele = vcf_var.ref_allele();
        let alt_allele = vcf_var.alt_allele();

       /*  let variant_id = match variant_id {
            Some(id) => id,
            None => rand::rng()
                .sample_iter(&Alphanumeric)
                .take(25)
                .map(char::from)
                .collect()
        };*/
        
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

    pub fn symbol(&self) -> &str {
        &self.symbol
    }

    pub fn hgnc_id(&self) -> &str {
        self.hgnc_id.as_ref()
    }

    pub fn hgvs(&self) -> &str {
        self.hgvs.as_ref()
    }

    pub fn transcript(&self) -> &str {
        self.transcript.as_ref()
    }

    pub fn g_hgvs(&self) -> &str {
        self.g_hgvs.as_ref()
    }

    pub fn is_x_chromosomal(&self) -> bool {
        return self.chr.contains("X");
    }

    /// returns a String key that can be used in HashMaps to unambiguously identify this variant
    pub fn variant_key(&self) -> String {
        format!("{}_{}_{}", self.hgvs, self.symbol, self.transcript)
    }

}




#[cfg(test)]
mod tests {

    use crate::{dto::variant_dto::VariantValidationDto, variant::hgvs_variant_validator::HgvsVariantValidator};
    use rstest::rstest;

    // test NM_000138.5(FBN1):c.8242G>T (p.Glu2748Ter)
    // We expect to get this back
    // HgvsVariant { assembly: "hg38", chr: "chr15", position: 48411364, ref_allele: "C", alt_allele: "A", symbol: Some("FBN1"), 
    // hgnc_id: Some("HGNC:3603"), hgvs: Some("NM_000138.5"), transcript: Some("NM_000138.5:c.8242G>T"), 
    // g_hgvs: Some("NC_000015.10:g.48411364C>A"), genotype: None, variant_id: "var_JgacXpZdmwKjarf125ud6ILjA" }
    #[rstest]
    #[ignore = "testing API"]
    fn test_hgvs_c_fbn1() {
        let vvalidator = HgvsVariantValidator::hg38();
        let vv_dto = VariantValidationDto::hgvs_c("c.8242G>T", "NM_000138.5", "HGNC:3603", "FBN1");
        let result = vvalidator.validate(vv_dto);
        assert!(result.is_ok());
        let hgvs_var = result.unwrap();
        println!("{:?}", hgvs_var);
        assert_eq!("hg38", hgvs_var.assembly());
        assert_eq!("chr15", hgvs_var.chr());
        assert_eq!(48411364, hgvs_var.position());
        assert_eq!("C", hgvs_var.ref_allele());
        assert_eq!("A", hgvs_var.alt_allele());
        assert_eq!("FBN1", hgvs_var.symbol());
        assert_eq!("HGNC:3603", hgvs_var.hgnc_id());
        assert_eq!("NM_000138.5", hgvs_var.transcript());
        assert_eq!("NC_000015.10:g.48411364C>A", hgvs_var.g_hgvs());
        let expected_key = "c.8242G>T_FBN1_NM_000138.5";
        assert_eq!(expected_key, hgvs_var.variant_key());
    }


}