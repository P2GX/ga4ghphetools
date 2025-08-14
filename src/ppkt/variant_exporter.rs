
use phenopackets::schema::v2::core::Interpretation;

use crate::{ppkt::ppkt_row::PpktRow, dto::{hgvs_variant::HgvsVariant, structural_variant::StructuralVariant}};

/*
 let variant_id = variant_id.unwrap_or_else(|| {
            let rand_str: String = rand::rng()
                .sample_iter(&Alphanumeric)
                .take(25)
                .map(char::from)
                .collect();
            format!("var_{}", rand_str)
        });
         */

/// A structure that is designed to transform the alleles and Variant objects into GA4GH Phenopacket Schema
/// The struct takes as input possibly empty lists of HGVS and SV variant objects. It determines whether the
/// Hgvs variants are X chromosomal (to call genotype hemizygous if the patient is male)
pub struct VariantExporter {
    /// this is only important for X-chromosomal diseases in which the genotype of "hemizygous" should be called for monoalleic X chromosomal variants in males
 is_male: bool
}

impl VariantExporter {
   pub fn new() -> Self {
        Self { 
            is_male: Default::default() 
        }
   }

   pub fn male() -> Self {
        Self { 
            is_male: true 
        }
   }

   pub fn to_ga4gh(&self, row: PpktRow, hgvs_list: Vec<HgvsVariant>, sv_list: Vec<StructuralVariant>) -> Vec<Interpretation> {
    println!("TODO to_ga4gh");
    /*for dto in &row.get_gene_var_dto_list() {
        *allele_to_count_map.entry(dto.get_key_allele1()).or_insert(0) += 1;
        if let Some(key) = dto.get_key_allele2() {
            *allele_to_count_map.entry(dto.get_key_allele1()).or_insert(0) += 1;
        }
    }
    for allell
*/

    vec![]
   }
}