//! Phenopacket module
//! 
//! Contains functions to create phenopackets from templates and a public function to
//! export a collection of phenopackets to an indicated directory.

use std::{fs::OpenOptions, path::PathBuf, sync::Arc};

use ontolius::ontology::csr::FullCsrOntology;
use phenopackets::schema::v2::Phenopacket;

use crate::{dto::cohort_dto::CohortData, ppkt::ppkt_exporter::PpktExporter};

mod ppkt_variant_exporter;
pub mod ppkt_exporter;
pub mod ppkt_row;


/// Write all `Phenopacket`s derived from a given `CohortData` to disk.
///
/// This function uses the provided [`CohortData`] and [`FullCsrOntology`]
/// to generate one phenopacket per individual (via [`PpktExporter`]),
/// and writes each packet as a pretty-printed JSON file to the specified output directory.
///
/// # Arguments
///
/// * `cohort_dto` - A `CohortData` object containing the cohort information 
///   to be exported into phenopackets.
/// * `dir` - A [`PathBuf`] representing the output directory where each phenopacket
///   JSON file will be written.
/// * `orcid` - The ORCID identifier of the submitting researcher. This will be 
///   embedded in each phenopacket.
/// * `hpo` - An `Arc<FullCsrOntology>` instance of the HPO ontology used to 
///   annotate the phenopackets.
///
/// # Returns
///
/// On success, returns a [`String`] containing a summary message, e.g.
/// `"Wrote 12 phenopackets to directory /path/to/output"`.
///
/// On error, returns a [`String`] describing the failure (e.g. file I/O or serialization error).
///
/// # Errors
///
/// This function will return an error if:
/// * phenopacket construction fails (via `PpktExporter`)
/// * writing to the target directory fails (e.g. permission issues)
/// * JSON serialization fails
///
/// # Example
///
/// ```ignore
/// use std::sync::Arc;
/// use std::path::PathBuf;
/// use ga4ghphetools::{CohortData, write_phenopackets};
/// use ga4ghphetools::dto::cohort_dto::CohortData;
/// use ontolius::ontology::csr::FullCsrOntology;
/// 
/// let cohort = CohortData::new(); // example
/// let ontology: Arc<FullCsrOntology> = get_hpo_ontology(); // need to get HPO from an appopriate place 
/// let dir = PathBuf::from("./output");
/// let orcid = "0000-0002-1825-0097".to_string();
///
/// match write_phenopackets(cohort, dir, orcid, ontology) {
///     Ok(msg) => println!("{}", msg),
///     Err(e) => eprintln!("Error: {}", e),
/// }
/// ```
pub fn write_phenopackets(
    cohort_dto: CohortData, 
    dir: PathBuf,
    orcid: String,
    hpo: Arc<FullCsrOntology>) 
-> Result<String, String> {
    let exporter = PpktExporter::new(hpo.clone(), &orcid, cohort_dto);
    let ppkt_list: Vec<Phenopacket> = exporter.get_all_phenopackets()?;
    let n_phenopackets = ppkt_list.len();
    for ppkt in ppkt_list {
        let title = ppkt.id.clone() + ".json";
        let mut file_path = dir.clone();
        file_path.push(title);
        write_ppkt(&ppkt, file_path)?;
    }
    let success_message = format!("Wrote {} phenopackets to directory {}", n_phenopackets, dir.to_string_lossy());
    Ok(success_message)
}

/// Write a single [`Phenopacket`] to a JSON file on disk.
///
/// The file will be overwritten if it already exists.
///
/// # Arguments
///
/// * `ppkt` - Reference to the phenopacket to serialize.
/// * `file_path` - Path to the target file (including filename, typically `*.json`).
///
/// # Errors
///
/// Returns an error if the file cannot be opened/created, or if JSON serialization fails.
fn write_ppkt(ppkt: &Phenopacket, file_path: PathBuf) -> Result<(), String> {
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&file_path)
        .map_err(|e| e.to_string())?;

    let mut json_value = serde_json::to_value(ppkt).map_err(|e| e.to_string())?;
    PpktExporter::strip_phenopacket_defaults(&mut json_value);
    
    serde_json::to_writer_pretty(file, &json_value)
        .map_err(|e| e.to_string())?; 
    Ok(())
}

/// Generate a list of `Phenopacket`s from a given `CohortData`.
///
/// This function converts the provided cohort into phenopackets using the
/// [`PpktExporter`], embedding the provided ORCID and annotating with the
/// specified HPO ontology.
///
/// Unlike [`write_phenopackets`], this function does **not** write anything
/// to disk. It only returns the constructed phenopackets in memory.
///
/// # Arguments
///
/// * `cohort_dto` - A `CohortData` object containing the cohort information 
///   to be converted into phenopackets.
/// * `orcid` - The ORCID identifier of the submitting researcher.
/// * `hpo` - An `Arc<FullCsrOntology>` instance of the HPO ontology used for
///   annotation.
///
/// # Returns
///
/// On success, returns a vector of `Phenopacket` objects generated from
/// the cohort.
///
/// On error, returns a [`String`] describing the failure (e.g. if the exporter fails).
///
/// # Errors
///
/// This function will return an error if:
/// * phenopacket construction fails within [`PpktExporter`]
pub fn get_phenopackets( 
    cohort_dto: CohortData, 
    orcid: String,
    hpo: Arc<FullCsrOntology>) 
-> Result<Vec<Phenopacket>, String> { 
    let exporter = PpktExporter::new(hpo.clone(), &orcid, cohort_dto);
    let ppkt_list: Vec<Phenopacket> = exporter.get_all_phenopackets()?;
    Ok(ppkt_list)
}