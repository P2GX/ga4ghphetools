//! Repository Q/C and Stats
//! 
//! 
//! 

use std::{collections::HashSet, path::Path, sync::Arc};

use ontolius::ontology::csr::FullCsrOntology;
use serde::Serialize;

use crate::repo::{compare_ppkt::{get_hpo_id_set, load_phenopacket_from_path}, gpt_repository::GptRepository, qc_report::UpdateReport, repo_qc::RepoQc};


mod cohort_dir;
mod cohort_qc;
mod disease_qc;
mod gpt_repository;
pub mod qc_report;
pub mod repo_qc;
mod compare_ppkt;


#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComparisonReport {
    pub id_match: bool,
    pub id_a: String,
    pub id_b: String,
    
    pub added_hpo: Vec<String>,
    pub removed_hpo: Vec<String>,
    
    pub added_variants: Vec<String>,
    pub removed_variants: Vec<String>,
}

pub fn compare_two_phenopackets(path1: String, path2: String, hpo: Arc<FullCsrOntology>) 
    ->  Result<ComparisonReport, String> 
 {
      let a = load_phenopacket_from_path(path1.into(), hpo.clone())?;
    let b = load_phenopacket_from_path(path2.into(), hpo.clone())?;
    let id_match = a.id == b.id;

    let hpo_a: HashSet<String> = get_hpo_id_set(&a);
    let hpo_b: HashSet<String> = get_hpo_id_set(&b);

    let added_hpo = hpo_b.difference(&hpo_a).cloned().collect();
    let removed_hpo = hpo_a.difference(&hpo_b).cloned().collect();

    // 3. Compare Variants (simplified example)
    let var_a: HashSet<_> = a.interpretations.iter()
        .flat_map(|i| &i.diagnosis)
        .flat_map(|d| &d.genomic_interpretations)
        .map(|gi| format!("{:?}", gi)) // Or a specific Variant ID/String
        .collect();
        
    let var_b: HashSet<_> = b.interpretations.iter()
        .flat_map(|i| &i.diagnosis)
        .flat_map(|d| &d.genomic_interpretations)
        .map(|gi| format!("{:?}", gi))
        .collect();

    let added_variants = var_b.difference(&var_a).cloned().collect();
    let removed_variants = var_a.difference(&var_b).cloned().collect();

    let cr = ComparisonReport {
        id_match,
        id_a: a.id.clone(),
        id_b: b.id.clone(),
        added_hpo,
        removed_hpo,
        added_variants,
        removed_variants,
    };
    Ok(cr)
 }


/// Perform a general quality check on the entire phenopacket store repository
/// ppkt_store_notebook_path points to the directory in PPKT Store that contains the individual gene folders
pub fn get_repo_qc(
    ppkt_store_notebook_path: &Path,
    hpo: Arc<FullCsrOntology>) -> Result<RepoQc, String> {
    let repo = GptRepository::new(ppkt_store_notebook_path);
    repo.repo_qc()
}

/// Update all HPO ids/labels in the entire phenopacket store
pub fn update_all_ppkt(
    ppkt_store_notebook_path: &Path,
    hpo: Arc<FullCsrOntology>
) -> Result<UpdateReport, String> {
    let repo = GptRepository::new(ppkt_store_notebook_path);
    repo.update_all_ppkt(hpo).map_err(|e|e.to_string())
}