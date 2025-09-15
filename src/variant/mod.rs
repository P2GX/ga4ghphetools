//! A module to work with HGVS (small) and structural variants.

use std::collections::{HashMap, HashSet};

use crate::{dto::hgvs_variant::HgvsVariant, variant::variant_manager::VariantManager};
pub mod acmg;
pub mod structural_validator;
pub mod variant_manager;
pub mod hgvs_variant_validator;
pub mod vcf_var;


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
) -> Result<HashMap<String, HgvsVariant>, String> {
    let mut vmanager = VariantManager::new(symbol, hgnc, transcript);
    vmanager.validate_all_sv(all_alleles, |p,q|{
        println!("{}/{} variants validated", p, q)})?;
    Ok(vmanager.hgvs_map())
}