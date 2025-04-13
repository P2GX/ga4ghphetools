// src/variant/hgvs_variant.rs
use crate::variant::acmg::AcmgPathogenicityClassification;
use crate::variant::variant_trait::Variant;
use crate::{error::Error, transcript, variant::vcf_var::{self, VcfVar}};
use ontolius::term::simple::SimpleMinimalTerm;
use rand::{self, distr::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};



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
    
   
    /* 
    fn to_variant_interpretation(&self, acmg: Option<AcmgPathogenicityClassification>) -> VariantInterpretation {
        let mut vcf_record = VcfRecord::new();
        vcf_record.genome_assembly = self.assembly.clone();
        vcf_record.chrom = self.chr.clone();
        vcf_record.pos = self.position;
        vcf_record.r#ref = self.ref_allele.clone();
        vcf_record.alt = self.alt_allele.clone();

        let mut vdescriptor = VariationDescriptor::new();
        vdescriptor.id = self.variant_id.clone();
        vdescriptor.vcf_record = Some(vcf_record);
        vdescriptor.molecule_context = MoleculeContext::Genomic as i32;

        if let (Some(id), Some(sym)) = (&self.hgnc_id, &self.symbol) {
            let gene = GeneDescriptor {
                value_id: id.clone(),
                symbol: sym.clone(),
                ..Default::default()
            };
            vdescriptor.gene_context = Some(gene);
        }

        if let Some(hgvs) = &self.hgvs {
            vdescriptor.expressions.push(Expression {
                syntax: "hgvs.c".to_string(),
                value: hgvs.clone(),
                ..Default::default()
            });
        }
        if let Some(g_hgvs) = &self.g_hgvs {
            vdescriptor.expressions.push(Expression {
                syntax: "hgvs.g".to_string(),
                value: g_hgvs.clone(),
                ..Default::default()
            });
        }

        if let Some(gt) = &self.genotype {
            if let Some(term) = Self::get_genotype_term(gt) {
                vdescriptor.allelic_state = Some(term);
            }
        }

        let mut interpretation = VariantInterpretation::new();
        interpretation.variation_descriptor = Some(vdescriptor);
        if let Some(code) = acmg {
            interpretation.acmg_pathogenicity_classification = code as i32;
        }
        interpretation
    }*/
}
