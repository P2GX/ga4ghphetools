//! IntergenicHgvsVariant
//! A data transfer object to represent all of the information we
//! need about an intergenic variant (e.g., promoter, enhancer) to represent it in a GA4GH Phenopacket
//! The information in our implementation is taken from the wonderful
//! VariantValidator API.
//! For instance, this could be NC_000019.10:g.12887294G>A (in the [upstream!] promoter region of GCDH).
//! Some variants are located within the gene model, as is this one, which therefore also has the 
//! gene-level HGVS, NG_009292.1:g.1135G>A

use serde::{Deserialize, Serialize};


use crate::variant::vcf_var::VcfVar;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntergenicHgvsVariant {
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
    /// Gene symbol, e.g., FBN1 (optional, e.g., if the variant lies with the gene definition of NCBI, VV will return the gene and HGNC)
    symbol: Option<String>,
    /// HUGO Gene Nomenclature Committee identifier, e.g., HGNC:3603
    hgnc_id: Option<String>,
    /// Genomic HGVS nomenclature, e.g., NC_000015.10:g.48411364C>A
    g_hgvs: String,
    /// Key to specify this variant in the HGVS HashMap of the CohortDto
    /// In our implementation for PubMed curation we will also use the key as the variant_id
    /// to export to phenopacket
    /// gene level HGVS, if available
    gene_hgvs: Option<String>,
    /// Key to specify this variant in the gHGVS HashMap of the CohortDto
    variant_key: String 
}

impl IntergenicHgvsVariant {
    pub fn new(
        assembly: String,
        vcf_var: VcfVar,
        symbol: Option<String>,
        hgnc_id: Option<String>,
        g_hgvs: String,
        gene_hgvs: Option<String>
    ) -> Self {
        let chr = vcf_var.chrom();
        let position = vcf_var.pos();
        let ref_allele = vcf_var.ref_allele();
        let alt_allele = vcf_var.alt_allele();
        let variant_key = Self::generate_variant_key(&g_hgvs);
        
        Self {
            assembly,
            chr,
            position,
            ref_allele,
            alt_allele,
            symbol,
            hgnc_id,
            g_hgvs,
            gene_hgvs,
            variant_key,
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

    pub fn symbol(&self) -> Option<String> {
        self.symbol.clone()
    }

    pub fn hgnc_id(&self) -> Option<String> {
        self.hgnc_id.clone()
    }

    pub fn gene_hgvs(&self) -> Option<String> {
        self.gene_hgvs.clone()
    }

    pub fn g_hgvs(&self) -> &str {
        &self.g_hgvs
    }

     pub fn is_x_chromosomal(&self) -> bool {
        return self.chr.contains("X");
    }

    /// returns a String key that can be used in HashMaps to unambiguously identify this variant
    pub fn variant_key(&self) -> String {
        IntergenicHgvsVariant::generate_variant_key(self.g_hgvs())
    }


    /// Create a key to use in our HashMap. It will also be serialized to JSON 
    /// and for for maximal safety/portability, we transform or remove
    /// non-alphanumerical characters (we allow underscore)
    /// For example, we would get c8242GtoT_FBN1_NM_000138v5
    /// from c.8242G>T, FBN1, and NM_000138.5
    pub fn generate_variant_key(g_hgvs: &str) -> String {
        let mut hgvs_norm = g_hgvs
            .replace("g.", "g")
            .replace('>', "to");
        hgvs_norm = hgvs_norm
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '_' })
            .collect();
       hgvs_norm
    }

}


mod tests {
    use super::*;


    #[test]
    pub fn test_variant_key() {
        let key = IntergenicHgvsVariant::generate_variant_key("NC_000019.10:g.12887294G>A");
        let expected = "NC_000019_10_g12887294GtoA";
        assert_eq!(expected, key);
    }


}