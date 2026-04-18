use std::{any::Any, collections::HashSet, fs, path::PathBuf, sync::Arc};
use ontolius::ontology::csr::{CsrOntology, FullCsrOntology};
use phenopackets::schema::v2::{Phenopacket, core::PhenotypicFeature};
use serde::Deserialize;

use crate::{dto::cohort_dto::CohortData, repo::ComparisonReport};


/// A private helper enum to facilitate "smart" deserialization.
/// 
/// The `#[serde(untagged)]` attribute tells Serde to attempt to match the JSON
/// structure against each variant in order without looking for a "type" tag.
#[derive(Deserialize)]
#[serde(untagged)]
enum PhenopacketSource {
    /// Matches if the JSON is a direct Phenopacket object.
    Direct(Phenopacket),
    /// Matches if the JSON follows the CohortData structure.
    FromCohort(CohortData),
}


///
pub fn load_phenopacket_from_path(path: PathBuf, hpo: Arc<FullCsrOntology>) -> Result<Phenopacket, String> {
    // 1. Read the file to a string
    let content = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    // 2. Try to parse as a direct Phenopacket
    let source: PhenopacketSource = serde_json::from_str(&content)
        .map_err(|e| format!("Validation Error: File does not match Phenopacket or CohortData schemas. Details: {}", e))?;

    // 3. Fallback: Try to parse as CohortData
    match source {
        PhenopacketSource::Direct(ppkt) => Ok(ppkt),
        PhenopacketSource::FromCohort(cohort) => {
            let fake_orcid = "0000-0000-0000-000x".to_string();
            let ppkt_list = crate::ppkt::get_phenopackets(
                cohort, fake_orcid, hpo)?;
            ppkt_list.into_iter().next()
                .ok_or_else(|| "Cohort recognized, but no phenopackets were generated.".to_string())
        }
    }
}

pub fn get_hpo_id_set(ppkt: &Phenopacket) -> HashSet<String> {
    ppkt.phenotypic_features
        .iter()
        .filter_map(|feature| {
            let ontology_class = feature.r#type.as_ref()?;
            Some(ontology_class.id.clone())
        })
        .collect()
}

