use std::collections::HashMap;

use phenopackets::ga4gh::vrsatile::v1::Expression;
use phenopackets::ga4gh::vrsatile::v1::Extension;
use phenopackets::ga4gh::vrsatile::v1::GeneDescriptor;
use phenopackets::ga4gh::vrsatile::v1::MoleculeContext;
use phenopackets::ga4gh::vrsatile::v1::VariationDescriptor;
use phenopackets::ga4gh::vrsatile::v1::VcfRecord;
use phenopackets::schema::v2::core::OntologyClass;
use phenopackets::schema::v2::core::Interpretation;
use phenopackets::schema::v2::core::TherapeuticActionability;
use phenopackets::schema::v2::core::VariantInterpretation;
use phenopackets::schema::v2::core::genomic_interpretation::Call;
use phenopackets::schema::v2::core::genomic_interpretation::InterpretationStatus;
use phenopackets::schema::v2::core::interpretation::ProgressStatus;
use phenopackets::schema::v2::core::{AcmgPathogenicityClassification, GenomicInterpretation};
use phenopackets::schema::v2::core::Diagnosis;
use rand::distr::{Alphanumeric, Distribution};
use rand::RngExt;
use crate::dto::cohort_dto::CohortData;
use crate::dto::cohort_dto::DiseaseData;
use crate::dto::cohort_dto::RowData;
use crate::dto::hgvs_variant::HgvsVariant;
use crate::dto::intergenic_variant::IntergenicHgvsVariant;
use crate::dto::structural_variant::StructuralVariant;


/// This is a helper ot reduce redundancy 
enum VariantRef<'a> {
    Hgvs(&'a HgvsVariant),
    Sv(&'a StructuralVariant),
    Intergenic(&'a IntergenicHgvsVariant),
}

/// A convenience builder structure to simplify creation of the
/// GA4GH VariationDescriptor message
struct VDescBuilder {
    id: String,
    gene_context: Option<GeneDescriptor>,
    expressions: Vec<Expression>,
    vcf_record: Option<VcfRecord>,
    structural_type: Option<OntologyClass>,
    label: String,
    allelic_state: OntologyClass,
    extensions: Vec<Extension>,
}

impl VDescBuilder {
    fn build(self) -> VariationDescriptor {
        VariationDescriptor {
            id: self.id,
            variation: None,
            label: self.label,
            description: String::default(),
            gene_context: self.gene_context,
            expressions: self.expressions,
            vcf_record: self.vcf_record,
            xrefs: vec![],
            alternate_labels: vec![],
            extensions: self.extensions,
            molecule_context: MoleculeContext::Genomic.into(),
            structural_type: self.structural_type,
            vrs_ref_allele_seq: String::default(),
            allelic_state: Some(self.allelic_state),
        }
    }

    fn with_optional_gene(mut self, hgnc_id: Option<String>, symbol: Option<String>) -> Self {
        if let (Some(id), Some(sym)) = (hgnc_id, symbol) {
            self.gene_context = Some(PpktVariantExporter::gene_descriptor(id, sym));
        }
        self
    }
    
}




/// Structure to coordinate extraction of Variant (Interpretation) information to export to Phenopacket 

pub struct PpktVariantExporter {
    is_male: bool,
    hgvs_variants: HashMap<String, HgvsVariant>,
    structural_variants: HashMap<String, StructuralVariant>,
    intergenic_variants: HashMap<String, IntergenicHgvsVariant>,
    disease_list: Vec<DiseaseData>,
}


impl PpktVariantExporter {
    pub fn new(is_male: bool, cohort: &CohortData) -> Self {
        Self { 
            is_male, 
            hgvs_variants: cohort.hgvs_variants.clone(), 
            structural_variants: cohort.structural_variants.clone(), 
            intergenic_variants: cohort.intergenic_variants.clone(),
            disease_list: cohort.disease_list.clone()
        }
    }

    /// A helper function to simplify getting the desired Variant object 
    /// (HGVS, SV, intergenic, mitochondrial) from the allele string
    fn lookup_variant<'a>(
        &'a self,
        allele: &str,
    ) -> Option<VariantRef<'a>> {
        self.hgvs_variants
            .get(allele)
            .map(VariantRef::Hgvs)
            .or_else(|| self.structural_variants.get(allele).map(VariantRef::Sv))
            .or_else(|| self.intergenic_variants.get(allele).map(VariantRef::Intergenic))
    }

    /// We add the codes ACMG Pathogenic and Unknown Therapeutic actionability
    /// to each variant description
    fn pathogenic_variant(vdesc: VariationDescriptor) -> VariantInterpretation {
        VariantInterpretation {
            acmg_pathogenicity_classification:
                AcmgPathogenicityClassification::Pathogenic.into(),
            therapeutic_actionability:
                TherapeuticActionability::UnknownActionability.into(),
            variation_descriptor: Some(vdesc),
        }
    }

     /// Builds a list of `Interpretation` objects for a given phenopacket row.
    ///
    /// This function performs the following steps:
    /// 1. Iterates through each allele in the input `RowData` and constructs corresponding
    ///    `VariantInterpretation` objects based on HGVS or structural variant information.
    /// 2. Ensures allele counts are valid (1 or 2). Returns an error if invalid or if a matching
    ///    validated variant cannot be found.
    /// 3. Validates that only one disease is present (melded/multiple diseases not implemented yet).
    /// 4. Extracts disease information and maps `GenomicInterpretation` objects to gene symbols.
    /// 5. For each disease, builds a `Diagnosis` linking its known genes to the corresponding
    ///    genomic interpretations (if available).
    /// 6. Wraps all constructed diagnoses into `Interpretation` objects.
    ///
    /// # Arguments
    /// * `ppkt_row` - A `RowData` object containing per-patient genotype and phenotype information.
    ///
    /// # Returns
    /// * `Ok(Vec<Interpretation>)` if all data were valid and interpretable.
    /// * `Err(String)` if any validation, mapping, or extraction step failed (e.g., missing allele, 
    ///   missing gene symbol, inconsistent disease data).
    pub fn get_interpretation_list(
        &self, 
        ppkt_row: &RowData) 
    -> std::result::Result<Vec<Interpretation>, String> {
        let mut v_interpretation_list: Vec<VariantInterpretation> = Vec::new();
        for (allele, count) in &ppkt_row.allele_count_map {
            let allele_count = *count;
            if  allele_count == 0 {
                return Err(format!("No alleles found in row {:?}", ppkt_row));
            }
            let vinterp = match self.lookup_variant(allele) {
                Some(VariantRef::Hgvs(v)) => self.get_hgvs_variant_interpretation(v, allele_count),
                Some(VariantRef::Sv(v)) => self.get_sv_variant_interpretation(v, allele_count),
                Some(VariantRef::Intergenic(v)) =>
                    self.get_intergenic_variant_interpretation(v, allele_count),
                None => return Err(format!("Could not find validated variant for allele {}", allele)),
            };
            v_interpretation_list.push(vinterp);
        }
        if self.disease_list.is_empty() {
            return Err("No disease objects found".to_string());
        }
       
        let mut g_interpretation_map: HashMap<String, Vec<GenomicInterpretation>> = HashMap::new();
        for vi in v_interpretation_list {
            let gi = GenomicInterpretation{
                subject_or_biosample_id: ppkt_row.individual_data.individual_id.to_string(),
                interpretation_status: InterpretationStatus::Causative.into(),
                call: Some(Call::VariantInterpretation(vi.clone()))
            };
            let symbol = Self::extract_gene_symbol(&vi)?;
            g_interpretation_map
                .entry(symbol)
                .or_default() 
                .push(gi);
        }
        let mut interpretation_list: Vec<Interpretation> = vec![];
        for disease in &self.disease_list {
            let disease_clz = OntologyClass{
                id: disease.disease_id.clone(),
                label: disease.disease_label.clone(),
            };
            let mut diagnosis = Diagnosis{
                disease: Some(disease_clz),
                genomic_interpretations: vec![],
            };
            diagnosis.genomic_interpretations.extend(
                disease.gene_transcript_list.iter()
                    .filter_map(|gene| g_interpretation_map.get(&gene.gene_symbol))
                    .flatten()
                    .cloned()
            );
            let i = Interpretation{
                id: Self::generate_id(),
                progress_status: ProgressStatus::Solved.into(),
                diagnosis: Some(diagnosis),
                summary: String::default(),
            };
            interpretation_list.push(i);
        }
        Ok(interpretation_list)
    }

    /// Create a GeneDescriptor message for the Phenopacket
    /// The elements are used in the gene context field.
    fn gene_descriptor(hgnc_id: impl Into<String>, symbol: impl Into<String>) 
        -> GeneDescriptor {
        GeneDescriptor {
            value_id: hgnc_id.into(),
            symbol: symbol.into(),
            description: String::default(),
            alternate_ids: vec![],
            alternate_symbols: vec![],
            xrefs: vec![],
        }
    }


    fn get_sv_variant_interpretation(
        &self,
        sv: &StructuralVariant,
        allele_count: usize
    ) -> VariantInterpretation {
        let gene_ctxt = Self::gene_descriptor(sv.hgnc_id(), sv.gene_symbol());
        let is_x = sv.is_x_chromosomal();
        let sv_class = sv.get_sequence_ontology_term();
        let allelic_state = self.get_genotype_term(allele_count, sv.is_x_chromosomal());
        let vdesc = VDescBuilder {
            id: sv.variant_key().to_string(),
            gene_context: Some(gene_ctxt),
            expressions: vec![],
            vcf_record: None,
            structural_type: Some(sv_class),
            label: sv.label().to_string(),
            allelic_state,
            extensions: vec![],
        }.build();
        Self::pathogenic_variant(vdesc)
    }


    fn get_hgvs_variant_interpretation(
        &self,
        hgvs: &HgvsVariant,
        allele_count: usize) 
    -> VariantInterpretation {
        let gene_ctxt = Self::gene_descriptor(hgvs.hgnc_id(), hgvs.symbol());
        let vcf_record = Self::get_vcf_record(
            hgvs.assembly(),
            hgvs.chr(),
        hgvs.position() as u64,
            hgvs.ref_allele(),
            hgvs.alt_allele());
        let hgvs_c = Expression{ 
            syntax: "hgvs.c".to_string(),
            value: format!("{}:{}", hgvs.transcript(), hgvs.hgvs()), 
            version: String::default() 
        };
        let mut expression_list = vec![hgvs_c];
        let hgvs_g = Expression{
                    syntax: "hgvs.g".to_string(),
                    value: hgvs.g_hgvs().to_string(),
                    version: String::default(),
                };
        expression_list.push(hgvs_g);
        if let Some(hgsvp) = hgvs.p_hgvs() {
            let hgvs_p = Expression{
                syntax: "hgvs.p".to_string(),
                value: hgsvp,
                version: String::default(),
            };
                expression_list.push(hgvs_p);
        };  
        let allelic_state = self.get_genotype_term(allele_count, hgvs.is_x_chromosomal());
        let vdesc = VDescBuilder { 
            id: hgvs.variant_key(), 
            gene_context: Some(gene_ctxt), 
            expressions: expression_list, 
            vcf_record: Some(vcf_record), 
            structural_type: None, 
            label: String::default(), 
            allelic_state, 
            extensions: vec![] 
            }.build();
        Self::pathogenic_variant(vdesc)
    }


    /// We assign biallelic variants the genotype of HOMOZYGOUS
    /// Monoallelic variants are assigned the genotype of HETEROZYGOUS except for
    /// X-chromosomal genes and males, in which case HEMIZYGOUS is assigned. For girls
    /// with monoallelic variants in X-chromosomal genes, HETEROZYGOUS is assigned.
     fn get_genotype_term(
        &self, 
        allele_count: usize,
        is_x: bool) -> OntologyClass {
        if  allele_count == 2 {
            OntologyClass {
                id: "GENO:0000136".to_string(),
                label: "homozygous".to_string(),
            }         
        } else if is_x && self.is_male {
            OntologyClass {
                id: "GENO:0000134".to_string(),
                label: "hemizygous".to_string(),
            }
        } else {
            OntologyClass {
                id: "GENO:0000135".to_string(),
                label: "heterozygous".to_string(),
            }
        }
    }

    /// Create a Phenopacket Schema (VRSATILE) VCF Record
    fn get_vcf_record(assembly: &str,
        chr: &str,
        pos: u64,
        ref_allele: &str,
        alt_allele: &str,
    ) -> VcfRecord {
         VcfRecord{ 
            genome_assembly: assembly.to_string(), 
            chrom: chr.to_string(), 
            pos, 
            id: String::default(), 
            r#ref: ref_allele.to_string(), 
            alt: alt_allele.to_string(), 
            qual: String::default(), 
            filter: String::default(), 
            info: String::default(), 
        }
    }

    fn get_intergenic_variant_interpretation(
        &self,
        ig: &IntergenicHgvsVariant,
        allele_count: usize
    ) -> VariantInterpretation {
        let vcf_record = Self::get_vcf_record(
            ig.assembly(),
            ig.chr(),
        ig.position() as u64,
            ig.ref_allele(),
            ig.alt_allele());
        let hgvs_g = Expression{ 
            syntax: "hgvs.g".to_string(),
            value: format!("{}", ig.g_hgvs()), 
            version: String::default() 
        };
        let expression_list = vec![hgvs_g];
        let allelic_state = self.get_genotype_term(allele_count, ig.is_x_chromosomal());
      
        let vdesc = VDescBuilder {
            id: ig.variant_key().to_string(),
            gene_context: None,
            expressions: expression_list,
            vcf_record: Some(vcf_record),
            structural_type: None,
            label: String::default(),
            allelic_state,
            extensions: vec![],
            }
            .with_optional_gene(ig.hgnc_id(), ig.symbol())
            .build();
        Self::pathogenic_variant(vdesc)
    }


    fn extract_gene_symbol(vi: &VariantInterpretation) -> Result<String, String> {
        vi
            .variation_descriptor
            .as_ref()
            .and_then(|vd| vd.gene_context.as_ref())
            .map(|gc| gc.symbol.clone())
            .ok_or_else(|| format!(
                "Missing gene symbol for variant interpretation: {:?}",
                vi.variation_descriptor
            ))
    }
    

    /// Generate a random identifier (used in this struct for Interpretation objects).
    fn generate_id() -> String {
        Alphanumeric
            .sample_iter(rand::rng())
            .take(24)
            .map(char::from)
            .collect()
    }


    

}