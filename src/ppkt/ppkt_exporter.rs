//! Module to export GA4GH Phenopackets from the information in the template.


use std::sync::Arc;

use ontolius::ontology::csr::FullCsrOntology;
use ontolius::ontology::MetadataAware;
use phenopacket_tools::builders::time_elements::time_element_from_str;
use phenopackets::ga4gh::vrsatile::v1::{Expression, GeneDescriptor, MoleculeContext, VariationDescriptor, VcfRecord};
use phenopackets::schema::v2::core::genomic_interpretation::{Call, InterpretationStatus};
use phenopackets::schema::v2::core::interpretation::ProgressStatus;
use phenopackets::schema::v2::core::{Diagnosis, KaryotypicSex, OntologyClass};
use phenopackets::schema::v2::core::vital_status::Status;
use phenopackets::schema::v2::core::{AcmgPathogenicityClassification, Disease, ExternalReference, GenomicInterpretation, Individual, Interpretation, MetaData, PhenotypicFeature, Sex, TherapeuticActionability, VariantInterpretation, VitalStatus};
use phenopackets::schema::v2::Phenopacket;

use rand::Rng;
use regex::Regex;
use crate::dto::cohort_dto::{CohortData, RowData};

use crate::dto::hgvs_variant::HgvsVariant;
use crate::dto::structural_variant::StructuralVariant;
use phenopacket_tools;
use phenopacket_tools::builders::builder::Builder;


const DEFAULT_HGNC_VERSION: &str =  "06/01/25";
const DEFAULT_OMIM_VERSION: &str =  "06/01/25";
const DEFAULT_SEQUENCE_ONTOLOGY_VERSION: &str =  "2024-11-18";
const DEFAULT_GENO_VERSION: &str =  "2025-07-25";

pub struct PpktExporter {
    /// Reference to the Ontolius Human Phenotype Ontology Full CSR object
    hpo: Arc<FullCsrOntology>,
    so_version: String,
    geno_version: String,
    omim_version: String,
    hgnc_version: String,
    orcid_id: String,
    cohort_dto: CohortData,
}

impl PpktExporter {


    pub fn new( 
        hpo: Arc<FullCsrOntology>,
        creator_orcid: &str,
        cohort: CohortData
    ) -> Self {
        Self::from_versions(
            hpo,
            DEFAULT_SEQUENCE_ONTOLOGY_VERSION,
            DEFAULT_GENO_VERSION,
            DEFAULT_OMIM_VERSION,
            DEFAULT_HGNC_VERSION,
            creator_orcid,
            cohort)
    }

    pub fn from_versions(
        hpo: Arc<FullCsrOntology>,
        so_version: &str, 
        geno_version: &str,
        omim_version: &str, 
        hgnc_version: &str ,
        creator_orcid: &str,
        cohort: CohortData
    ) -> Self {
        Self{ 
            hpo, 
            so_version: so_version.to_string(), 
            geno_version: geno_version.to_string(),
            omim_version: omim_version.to_string(), 
            hgnc_version: hgnc_version.to_string(),
            orcid_id: creator_orcid.to_string(),
            cohort_dto: cohort
        }
    }


    /// Create a GA4GH Individual message
    pub fn extract_individual(&self, ppkt_row: &RowData) -> Result<Individual, String> {
        let individual_dto = &ppkt_row.individual_data;
        let mut idvl = Individual{ 
            id: individual_dto.individual_id.clone(), 
            alternate_ids: vec![], 
            date_of_birth: None, 
            time_at_last_encounter: None, 
            vital_status: None, 
            sex: Sex::UnknownSex.into(), 
            karyotypic_sex: KaryotypicSex::UnknownKaryotype.into(), 
            gender: None, 
            taxonomy: None };
        match individual_dto.sex.as_ref() {
            "M" => idvl.sex = Sex::Male.into(),
            "F" => idvl.sex = Sex::Female.into(),
            "O" => idvl.sex = Sex::OtherSex.into(),
            "U" => idvl.sex = Sex::UnknownSex.into(),
            _ => { return Err(format!("Did not recognize sex string '{}'", idvl.sex)); }
        };
        let last_enc = &individual_dto.age_at_last_encounter;
        if last_enc != "na" {
            let age = time_element_from_str(last_enc)
                .map_err(|e| format!("malformed_time_element {}",e.to_string()))?;
            idvl.time_at_last_encounter = Some(age);
        }
        if individual_dto.deceased == "yes" {
            idvl.vital_status = Some(VitalStatus{ 
                status: Status::Deceased.into(), 
                time_of_death: None, 
                cause_of_death: None, 
                survival_time_in_days: 0 
            });
        } 
        Ok(idvl)

    }

    pub fn hpo_version(&self) -> &str {
        &self.hpo.version()
    } 

    pub fn so_version(&self) -> &str {
        &self.so_version
    } 

    pub fn geno_version(&self) -> &str {
        &self.geno_version
    } 

    pub fn omim_version(&self) -> &str {
        &self.omim_version
    } 

    pub fn hgnc_version(&self) -> &str {
        &self.hgnc_version
    } 

    /// Create GA4GH MetaData object from version numbers using functions from phenopacket_tools
    pub fn get_meta_data(&self, row_dto: &RowData) -> Result<MetaData, String> {
        let created_by = self.orcid_id.clone();
        let mut meta_data = Builder::meta_data_now(created_by);
        let hpo = phenopacket_tools::builders::resources::Resources::hpo_version(self.hpo_version());
        let geno = phenopacket_tools::builders::resources::Resources::geno_version(self.geno_version());
        let so = phenopacket_tools::builders::resources::Resources::so_version(self.so_version());
        let omim = phenopacket_tools::builders::resources::Resources::omim_version(self.omim_version());
        let hgnc = phenopacket_tools::builders::resources::Resources::hgnc_version(&self.hgnc_version());
        let indvl_dto = row_dto.individual_data.individual_id.clone();
        let ext_res = ExternalReference{ 
            id: row_dto.individual_data.pmid.clone(), 
            reference: String::default(), 
            description: row_dto.individual_data.title.clone()
        };
        meta_data.resources.push(hpo);
        meta_data.resources.push(geno);
        meta_data.resources.push(so);
        meta_data.resources.push(omim);
        meta_data.resources.push(hgnc);
        meta_data.external_references.push(ext_res);
        Ok(meta_data)
    }


    /// Generate the phenopacket identifier from the PMID and the individual identifier
    pub fn get_phenopacket_id(&self, ppkt_row: &RowData) -> String {
        let individual_dto = &ppkt_row.individual_data;
        let pmid = ppkt_row.individual_data.pmid.replace(":", "_");
        let individual_id = individual_dto.individual_id.replace(" ", "_");
        let ppkt_id = format!("{}_{}", pmid, individual_id);
        let ppkt_id = ppkt_id.replace("__", "_");
        // Replace any non-ASCII characters with _, but remove trailing "_" if it exists.
        let mut sanitized: String = ppkt_id.chars()
            .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
            .clone().collect();
         // Collapse multiple underscores, if any
        let re = Regex::new(r"_+").unwrap();
        sanitized = re.replace_all(&sanitized, "_").to_string();
        if let Some(stripped) = sanitized.strip_suffix('_') {
            sanitized = stripped.to_string();
        }
        sanitized
    }

    /// TODO extend for multiple diseases
    pub fn get_disease(&self, ppkt_row: &RowData) -> Result<Disease, String> {
        let disease_list = &ppkt_row.disease_data_list;
        if disease_list.is_empty() {
            return Err(format!("todo empty disease"));
        }
        let dto = disease_list[0].clone();
        let dx_id = Builder::ontology_class(dto.disease_id, dto.disease_label)
            .map_err(|e| format!("malformed disease id: {:?}", disease_list))?;
        let mut disease = Disease{ 
            term: Some(dx_id), 
            excluded: false, 
            onset: None, 
            resolution: None, 
            disease_stage: vec![], 
            clinical_tnm_finding: vec![], 
            primary_site: None, 
            laterality: None 
        };
        let idl_dto = ppkt_row.individual_data.individual_id.clone();
        let onset = &ppkt_row.individual_data.age_of_onset;
        if onset != "na" {
            let age = time_element_from_str(onset)
                .map_err(|e| format!("malformed_time_element {}",e.to_string()))?;
            disease.onset = Some(age);
        };
        Ok(disease)
    }

    fn allele_not_contained(allele: &str) -> String {
        format!("'{allele}' must be validated before exporting to Phenopacket Schema")
    }



    fn get_sv_variant_interpretation(
        sv: &StructuralVariant,
        allele_count: usize
    ) -> VariantInterpretation {
        let gene_ctxt = GeneDescriptor{ 
            value_id: sv.hgnc_id().to_string(), 
            symbol: sv.gene_symbol().to_string(), 
            description: String::default(), 
            alternate_ids: vec![] , 
            alternate_symbols: vec![] , 
            xrefs: vec![] 
            };
        let is_x = sv.is_x_chromosomal();
        let sv_class = sv.get_sequence_ontology_term();
        let allelic_state = Self::get_allele_term(allele_count, sv.is_x_chromosomal());
        
        let vdesc = VariationDescriptor {
            id: sv.variant_key().to_string(),
            variation: None,
            label: sv.label().to_string(),
            description: String::default(),
            gene_context: Some(gene_ctxt),
            expressions: vec![],
            vcf_record: None,
            xrefs: vec![],
            alternate_labels: vec![],
            extensions: vec![],
            molecule_context: MoleculeContext::Genomic.into(),
            structural_type: Some(sv_class),
            vrs_ref_allele_seq: String::default(),
            allelic_state: Some(allelic_state),
        };
        let vi = VariantInterpretation{ 
            acmg_pathogenicity_classification: AcmgPathogenicityClassification::Pathogenic.into(), 
            therapeutic_actionability: TherapeuticActionability::UnknownActionability.into(), 
            variation_descriptor: Some(vdesc) 
        };
        vi
    }

    fn get_allele_term(allele_count: usize, is_x: bool) -> OntologyClass {
        if  allele_count == 2 {
            return OntologyClass {
                id: "GENO:0000136".to_string(),
                label: "homozygous".to_string(),
            };            
        } else if is_x {
            return OntologyClass {
                id: "GENO:0000134".to_string(),
                label: "hemizygous".to_string(),
            }; 
        } else {
            return OntologyClass {
                id: "GENO:0000135".to_string(),
                label: "heterozygous".to_string(),
            }; 
        }
    }
      
    fn get_hgvs_variant_interpretation(
        hgvs: &HgvsVariant,
        allele_count: usize) 
    -> VariantInterpretation {
        let gene_ctxt = GeneDescriptor{ 
            value_id: hgvs.hgnc_id().to_string(), 
            symbol: hgvs.symbol().to_string(), 
            description: String::default(), 
            alternate_ids: vec![] , 
            alternate_symbols: vec![] , 
            xrefs: vec![] 
            };
        let vcf_record = VcfRecord{ 
            genome_assembly: hgvs.assembly().to_string(), 
            chrom: hgvs.chr().to_string(), 
            pos: hgvs.position() as u64, 
            id: String::default(), 
            r#ref: hgvs.ref_allele().to_string(), 
            alt: hgvs.alt_allele().to_string(), 
            qual: String::default(), 
            filter: String::default(), 
            info: String::default(), 
        };

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
         
        let allelic_state = Self::get_allele_term(allele_count, hgvs.is_x_chromosomal());
        let vdesc = VariationDescriptor{ 
            id: hgvs.variant_key().to_string(), 
            variation: None, 
            label: String::default(), 
            description: String::default(), 
            gene_context: Some(gene_ctxt), 
            expressions: expression_list, 
            vcf_record: Some(vcf_record), 
            xrefs: vec![], 
            alternate_labels: vec![], 
            extensions: vec![], 
            molecule_context: MoleculeContext::Genomic.into(), 
            structural_type: None, 
            vrs_ref_allele_seq: String::default(), 
            allelic_state: Some(allelic_state) 
        };
        let vi = VariantInterpretation{ 
            acmg_pathogenicity_classification: AcmgPathogenicityClassification::Pathogenic.into(), 
            therapeutic_actionability: TherapeuticActionability::UnknownActionability.into(), 
            variation_descriptor: Some(vdesc) 
        };
        vi
    }

    /// Generate a random identifier (used in this struct for Interpretation objects).
    pub fn generate_id() -> String {
        rand::rng()
            .sample_iter(&rand::distr::Alphanumeric)
            .take(24)
            .map(char::from)
            .collect()
    }
    
    /// TODO, for melded, we need to assign genes to diseases
    pub fn get_interpretation_list(
        &self, 
        ppkt_row: &RowData) 
    -> std::result::Result<Vec<Interpretation>, String> {
        let mut v_interpretation_list: Vec<VariantInterpretation> = Vec::new();
        for (allele, count) in &ppkt_row.allele_count_map {
            let allele_count = *count;
            if allele_count > 2 || allele_count < 1 {
                return Err(format!("Invalid count ({}) for allele '{}'", count, allele));
            }
            if let Some(hgvs) = self.cohort_dto.hgvs_variants.get(allele) {
                let vinterp = Self::get_hgvs_variant_interpretation( hgvs, allele_count);
                v_interpretation_list.push(vinterp);
            } else if let Some(sv) = self.cohort_dto.structural_variants.get(allele) {
                let vinterp = Self::get_sv_variant_interpretation(sv, allele_count);
                v_interpretation_list.push(vinterp);
            } else {
                return Err(format!("Could not find validated variant for allele {}", allele));
            }
        }
        if self.cohort_dto.disease_gene_data.disease_dto_list.len() != 1 {
            return Err(format!("Melded disease interpretation not implemented yet: {:?}", self.cohort_dto.disease_gene_data.disease_dto_list));
        }
        let d_dto = &self.cohort_dto.disease_gene_data.disease_dto_list[0];
    
        let disease_clz = OntologyClass{
            id: d_dto.disease_id.clone(),
            label: d_dto.disease_label.clone(),
        };
        let mut g_interpretations: Vec<GenomicInterpretation> = Vec::new();
        for vi in v_interpretation_list {
            let gi = GenomicInterpretation{
                subject_or_biosample_id: ppkt_row.individual_data.individual_id.to_string(),
                interpretation_status: InterpretationStatus::Causative.into(),
                call: Some(Call::VariantInterpretation(vi))
            };
            g_interpretations.push(gi);
        }
        let diagnosis = Diagnosis{
            disease: Some(disease_clz),
            genomic_interpretations: g_interpretations,
        };
        let i = Interpretation{
            id: Self::generate_id(),
            progress_status: ProgressStatus::Solved.into(),
            diagnosis: Some(diagnosis),
            summary: String::default(),
        };
        let interpretation_list: Vec<Interpretation> = vec![i];
        Ok(interpretation_list)
    }

    


    pub fn get_phenopacket_features(&self, ppkt_row: &RowData) -> Result<Vec<PhenotypicFeature>, String> {
        let hpo_term_list = &self.cohort_dto.hpo_headers;
        let hpo_data = &ppkt_row.hpo_data;
        if hpo_data.len() != hpo_term_list.len() {
            return Err(format!("Length of HPO headers ({}) does not match length of HPO values {}",
            hpo_term_list.len(), hpo_data.len()));
        }
        let mut ppkt_feature_list: Vec<PhenotypicFeature> = Vec::with_capacity(hpo_data.len());
        for (term, cell_contents) in hpo_term_list.iter().zip(hpo_data.iter()) {
            if ! cell_contents.is_ascertained() {
                continue;
            }
            let hpo_term = Builder::ontology_class(term.hpo_id(), term.hpo_label())
                .map_err(|e| format!("termid_parse_error '{:?}'", term))?;
            let mut pf = PhenotypicFeature{ 
                description: String::default(), 
                r#type: Some(hpo_term), 
                excluded: cell_contents.is_excluded(), 
                severity: None, 
                modifiers: vec![], 
                onset: None,
                resolution: None, 
                evidence: vec![]
            };
            if cell_contents.has_onset() {
                let ost = time_element_from_str(&cell_contents.to_string())
                    .map_err(|e| format!("malformed_time_element{}", cell_contents))?;
                pf.onset = Some(ost);
            }
            ppkt_feature_list.push(pf);
        }
        Ok(ppkt_feature_list)
    }


 pub fn extract_phenopacket_from_dto(
        &self, 
        ppkt_row_dto: &RowData, 
    ) -> Result<Phenopacket, String> {
        if self.cohort_dto.disease_gene_data.gene_transcript_dto_list.len() != 1 {
            panic!("NEED TO EXTEND MODEL TO NON MEND. NEED TO EXTEND CACHE KEY FOR GENE-TRANSCRIPT-NAME");
        }
        let interpretation_list = self.get_interpretation_list(ppkt_row_dto)?;
        let ppkt = Phenopacket{ 
            id: self.get_phenopacket_id(ppkt_row_dto), 
            subject:  Some(self.extract_individual(ppkt_row_dto)?), 
            phenotypic_features: self.get_phenopacket_features(ppkt_row_dto)?, 
            measurements: vec![], 
            biosamples: vec![], 
            interpretations: interpretation_list, 
            diseases: vec![self.get_disease(ppkt_row_dto)?], 
            medical_actions: vec![], 
            files: vec![], 
            meta_data: Some(self.get_meta_data(ppkt_row_dto)?) 
        };
    
        Ok(ppkt)


    }

    pub fn get_all_phenopackets(&self) -> Result<Vec<Phenopacket>, String> {
        let mut ppkt_list: Vec<Phenopacket> = Vec::new();
        for row in &self.cohort_dto.rows {
           let ppkt = self.extract_phenopacket_from_dto(row)?;
           ppkt_list.push(ppkt);
        }

        Ok(ppkt_list)
    }


}