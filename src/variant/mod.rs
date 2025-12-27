//! A module to work with HGVS (small) and structural variants.

use std::collections::{HashMap, HashSet};

use crate::{dto::{cohort_dto::{CohortData, GeneTranscriptData}, etl_dto::EtlDto, hgvs_variant::HgvsVariant, structural_variant::{StructuralVariant, SvType}, variant_dto::VariantDto}, variant::variant_manager::VariantManager};
mod acmg;
mod structural_validator;
pub mod variant_manager;
pub(crate) mod hgvs_variant_validator;
pub(crate) mod intergenic_hgvs_validator;
pub(crate) mod variant_validation_handler;
pub(crate) mod vcf_var;


/// Validates all HGVS variants in the given set of allele strings.
///
/// This function creates a [`VariantManager`] for the provided gene symbol, HGNC identifier,
/// and transcript. It attempts to parse and validate every allele in `all_alleles` that
/// appears to be an HGVS variant (strings beginning with `c.` or `n.`).
///
/// During validation, progress is printed to standard output in the form:
///
/// ```text
/// {validated}/{total} variants validated
/// ```
///
/// # Arguments
///
/// * `symbol`     – Gene symbol (e.g. `"BRCA1"`).
/// * `hgnc`       – HGNC identifier for the gene.
/// * `transcript` – Transcript identifier against which the variants should be validated.
/// * `all_alleles` – A set of allele strings that may contain HGVS-formatted variants.
///
/// # Returns
///
/// * `Ok(HashMap<String, HgvsVariant>)` – A map from the original allele string to its
///   successfully parsed [`HgvsVariant`].
/// * `Err(String)` – If validation fails, returns an error message describing the problem.
///
/// # Side Effects
///
/// * Prints progress updates to `stdout`.
///
/// # Examples
///
/// ```ignore
/// use std::collections::HashSet;
///
/// let alleles: HashSet<String> = ["c.123A>G".into(), "n.456C>T".into()].into();
///
/// let result = validate_all_hgvs("BRCA1", "HGNC:1100", "NM_007294.3", &alleles);
///
/// match result {
///     Ok(map) => println!("Validated {} variants", map.len()),
///     Err(e) => eprintln!("Validation failed: {}", e),
/// }
/// ```
pub fn validate_all_hgvs(
    symbol: &str, 
    hgnc: &str, 
    transcript: &str,
    all_alleles: &HashSet<String>
) -> Result<HashMap<String, HgvsVariant>, String> {
    let mut vmanager = VariantManager::new(symbol, hgnc, transcript);
    vmanager.validate_all_hgvs(all_alleles, |p,q|{
        println!("{}/{} variants validated", p, q)})?;
    Ok(vmanager.hgvs_map())
}

/// Validates a single HGVS variant from one allele string (e.g., c.123A>C).
/// # Arguments
///
/// * `symbol`     – Gene symbol (e.g. `"BRCA1"`).
/// * `hgnc`       – HGNC identifier for the gene.
/// * `transcript` – Transcript identifier against which the variants should be validated.
/// * `allele` – A string that should contain an HGVS-formatted variant.
///
/// # Returns
///
/// * `Ok(HgvsVariant)` – The successfully parsed [`HgvsVariant`].
/// * `Err(String)` – If validation fails, returns an error message describing the problem.
pub fn validate_one_hgvs_variant(
    symbol: &str,
    hgnc: &str,
    transcript: &str,
    allele: &str) 
-> Result<HgvsVariant, String> {
    let mut vmanager = VariantManager::new(symbol, hgnc, transcript);
    vmanager.get_validated_hgvs(allele)
}

/* 
/// Validates a structural variant in the given string.
///
/// This function is intended for use with symbol structural variant strings such as DEL ex 5
/// # Arguments
///
/// * `symbol`     – Gene symbol (e.g. `"BRCA1"`).
/// * `hgnc`       – HGNC identifier for the gene.
/// * `transcript` – Transcript identifier against which the variants should be validated.
/// * `allele` – A string that represents a SV, e.g., DUP ex 9-10.
///
/// # Returns
///
/// * `Ok(StructuralVariant)` – The successfully parsed [`StructuralVariant`].
/// * `Err(String)` – If validation fails, returns an error message describing the problem.
pub fn validate_one_structural_variant(
    symbol: &str,
    hgnc: &str,
    transcript: &str,
    allele: &str) 
-> Result<StructuralVariant, String> {
    let vmanager = VariantManager::new(symbol, hgnc, transcript);
    vmanager.get_validated_sv(allele)
}

*/
pub fn validate_structural_variant(
    variant_dto: VariantDto
) -> Result<StructuralVariant, String> {
    if variant_dto.is_hgvs() {
        return Err(format!("Expecting to validate structural variant, but got {:?}", variant_dto));
    }
    let symbol = variant_dto.gene_symbol;
    let transcript = variant_dto.transcript;
    let hgnc = variant_dto.hgnc_id;
    let allele = variant_dto.variant_string;
    let var_type = variant_dto.variant_type;
    let mut vmanager = VariantManager::new(&symbol, &hgnc, &transcript);
    vmanager.get_validated_structural_variant(&allele, var_type)
}


/// Validates all structural variants in the given set of allele strings.
///
/// This function creates a [`VariantManager`] for the provided gene symbol, HGNC identifier,
/// and transcript. It attempts to parse and validate every allele in `all_alleles` that
/// appears not to be an HGVS variant (strings that do not begin with `c.` or `n.`).
///
/// During validation, progress is printed to standard output in the form:
///
/// ```text
/// {validated}/{total} variants validated
/// ```
///
/// # Arguments
///
/// * `symbol`     – Gene symbol (e.g. `"BRCA1"`).
/// * `hgnc`       – HGNC identifier for the gene.
/// * `transcript` – Transcript identifier against which the variants should be validated.
/// * `all_alleles` – A set of allele strings that may contain HGVS-formatted variants.
///
/// # Returns
///
/// * `Ok(HashMap<String, HgvsVariant>)` – A map from the original allele string to its
///   successfully parsed [`HgvsVariant`].
/// * `Err(String)` – If validation fails, returns an error message describing the problem.
///
/// # Side Effects
///
/// * Prints progress updates to `stdout`.
pub fn validate_all_sv(
    symbol: &str, 
    hgnc: &str, 
    transcript: &str,
    all_alleles: &HashSet<String>
) -> Result<HashMap<String, StructuralVariant>, String> {
    let mut vmanager = VariantManager::new(symbol, hgnc, transcript);
    vmanager.validate_all_sv(all_alleles, |p,q|{
        println!("{}/{} variants validated", p, q)})?;
    Ok(vmanager.sv_map())
}

pub fn validate_etl_dto(
    etl_dto: EtlDto,
    transcript: String,
    hgnc: String,
    symbol: String) -> Result<EtlDto, String> {
    let mut vmanager = VariantManager::new(&symbol, &hgnc, &transcript);  

    Err("Not implemented yet".to_string())
}


/// Analyze and summarize genetic variants for a Mendelian cohort.
///
/// This function extracts the relevant disease and gene–transcript context
/// from the provided [`CohortData`] and delegates variant analysis to
/// [`VariantManager`]. The result is a list of [`VariantDto`] values suitable
/// for downstream display or reporting.
///
/// # Requirements
///
/// * The cohort **must** represent a Mendelian disease model.
/// * At least one [`DiseaseData`] entry must be present in the cohort.
/// * The first disease must contain at least one [`GeneTranscriptData`] entry.
///
/// # Arguments
///
/// * `cohort_dto` – Input cohort containing disease and gene–transcript data.
///
/// # Returns
///
/// * `Ok(Vec<VariantDto>)` on successful analysis.
/// * `Err(String)` if:
///   * the cohort is not Mendelian,
///   * no disease data is available, or
///   * no gene–transcript data can be extracted.
///
/// # Errors
///
/// This function returns a string-based error if required structural data
/// is missing or if the cohort model is unsupported.
///
/// # Notes
///
/// Currently, only Mendelian cohorts are supported
pub fn analyze_variants(cohort_dto: CohortData) -> Result<Vec<VariantDto>, String> {
    if ! cohort_dto.is_mendelian() {
        return Err(format!("analyze_variants is only implemented for Mendelian"));
    }
    let disease_data = match cohort_dto.disease_list.first() {
        Some(data) => data.clone(),
        None =>  { return Err(format!("Unable to extract DiseaseData")); },
    };
    
    let gt_data: GeneTranscriptData = match disease_data.gene_transcript_list.first() {
        Some(data) => data.clone(),
        None =>  { return Err(format!("Unable to extract GeneTranscriptData")); }
    };
    let vmanager = VariantManager::from_gene_transcript_dto(&gt_data);
    vmanager.analyze_variants(cohort_dto)
}