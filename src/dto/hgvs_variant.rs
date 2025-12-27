//! HgvsVariant
//! A data transfer object to represent all of the information we
//! need about a Variant to represent it in a GA4GH Phenopacket
//! The information in our implementation is taken from the wonderful
//! VariantValidator API.

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
    /// Protein level HGVS, if available
    p_hgvs: Option<String>,
    /// Key to specify this variant in the HGVS HashMap of the CohortDto
    variant_key: String 
}

impl HgvsVariant {
    pub fn new(
        assembly: String,
        vcf_var: VcfVar,
        symbol: String,
        hgnc_id: String,
        hgvs: String,
        p_hgvs: Option<String>,
        transcript: String,
        g_hgvs: String,
    ) -> Self {
        let chr = vcf_var.chrom();
        let pos = vcf_var.pos();
        let ref_allele = vcf_var.ref_allele();
        let alt_allele = vcf_var.alt_allele();
        let v_key = Self::generate_variant_key(&hgvs, &symbol, &transcript);
        
        HgvsVariant {
            assembly,
            chr,
            position: pos,
            ref_allele,
            alt_allele,
            symbol,
            hgnc_id,
            hgvs,
            p_hgvs,
            transcript,
            g_hgvs,
            variant_key: v_key
        }
    }

    pub fn new_from_parts(
        assembly: String,
        chromosome: String,
        pos: u32,
        reference: String,
        alternate: String,
        symbol: String,
        hgnc_id: String,
        hgvs: String,
        transcript: String,
        g_hgvs: String,
    ) -> Self {
        let vcf_var = VcfVar::new(chromosome, pos, reference, alternate);
        Self::new(assembly, vcf_var, symbol, hgnc_id, hgvs, None, transcript, g_hgvs)
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

    pub fn p_hgvs(&self) -> Option<String> {
        match &self.p_hgvs {
            Some(phgvs) => Some(phgvs.to_string()),
            None => None
        }
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
        HgvsVariant::generate_variant_key(self.hgvs(), self.symbol(), self.transcript())
    }
    
    /// Create a key to use in our HashMap. It will also be serialized to JSON 
    /// and for for maximal safety/portability, we transform or remove
    /// non-alphanumerical characters (we allow underscore)
    /// For example, we would get c8242GtoT_FBN1_NM_000138v5
    /// from c.8242G>T, FBN1, and NM_000138.5
    pub fn generate_variant_key(hgvs: &str, symbol: &str, transcript: &str) -> String {
        let mut hgvs_norm = hgvs
            .replace("c.", "c")
            .replace('>', "to");
        hgvs_norm = hgvs_norm
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '_' })
            .collect();

        let transcript_norm = transcript.replace('.', "v");

        format!("{}_{}_{}", hgvs_norm, symbol, transcript_norm)
    }
}




#[cfg(test)]
mod tests {

    use crate::{dto::variant_dto::VariantDto, variant::hgvs_variant_validator::HgvsVariantValidator};
    use rstest::rstest;

    // test NM_000138.5(FBN1):c.8242G>T (p.Glu2748Ter)
    // We expect to get this back
    // HgvsVariant { assembly: "hg38", chr: "chr15", position: 48411364, ref_allele: "C", alt_allele: "A", symbol: Some("FBN1"), 
    // hgnc_id: Some("HGNC:3603"), hgvs: Some("NM_000138.5"), transcript: Some("NM_000138.5:c.8242G>T"), 
    // g_hgvs: Some("NC_000015.10:g.48411364C>A"), genotype: None, variant_id: "var_JgacXpZdmwKjarf125ud6ILjA" }
    #[rstest]
    #[ignore = "testing API"]
    fn test_hgvs_c_fbn1() {
        let mut vvalidator = HgvsVariantValidator::hg38();
        let vv_dto = VariantDto::hgvs_c("c.8242G>T", "NM_000138.5", "HGNC:3603", "FBN1");
        let result = vvalidator.get_validated_hgvs(&vv_dto);
        assert!(result.is_ok());
        let hgvs_var = result.unwrap();
        assert_eq!("hg38", hgvs_var.assembly());
        assert_eq!("chr15", hgvs_var.chr());
        assert_eq!(48411364, hgvs_var.position());
        assert_eq!("C", hgvs_var.ref_allele());
        assert_eq!("A", hgvs_var.alt_allele());
        assert_eq!("FBN1", hgvs_var.symbol());
        assert_eq!("HGNC:3603", hgvs_var.hgnc_id());
        assert_eq!("NM_000138.5", hgvs_var.transcript());
        assert_eq!("NC_000015.10:g.48411364C>A", hgvs_var.g_hgvs());
        let expected_key = "c8242GtoT_FBN1_NM_000138v5";
        assert_eq!(expected_key, hgvs_var.variant_key());
    }


}