//! Repository Q/C and Stats
//! 
//! 
//! 
//! 
pub(crate) mod gpt_repository;
mod compare_ppkt;
pub mod update_report;
mod ppkt_wrapper;
mod cohort_wrapper;
pub mod repo_qc;

use std::{collections::HashSet, path::Path, sync::Arc};
use ontolius::ontology::csr::FullCsrOntology;
use serde::Serialize;

use crate::{RepoQc, cohort_qc::{cohort_qc::CohortDataQc, qc_report::QcReport}, repo::{compare_ppkt::{get_hpo_id_set, load_phenopacket_from_path}, gpt_repository::GptRepository, update_report::UpdateReport}};




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




/// Update all HPO ids/labels in the entire phenopacket store
pub fn update_all_ppkt(
    ppkt_store_notebook_path: &Path,
    hpo: Arc<FullCsrOntology>
) -> Result<UpdateReport, String> {
    let repo = GptRepository::new(ppkt_store_notebook_path);
    repo.update_all_ppkt(hpo).map_err(|e|e.to_string())
}


/// Perform a general quality check on the entire phenopacket store repository
/// ppkt_store_notebook_path points to the directory in PPKT Store that contains the individual gene folders
pub fn get_repo_qc(
    ppkt_store_notebook_path: &Path,
    hpo: Arc<FullCsrOntology>) -> Result<RepoQc, String> {
    let repo = GptRepository::new(ppkt_store_notebook_path);
    let cohort_wrapper_list = repo.get_all_cohort_wrappers().map_err(|e|e.to_string())?;
    let mut qc_report_list: Vec<QcReport> = Vec::new();
    let cohort_qc = CohortDataQc::new(hpo.clone());
    for cohort_wrap in cohort_wrapper_list.into_iter() {
        let cohort_data = cohort_wrap.cohort_data();
        let qc = cohort_qc.qc_check(cohort_data).map_err(|e|e.to_string())?;
        qc_report_list.push(qc);        
    }
    let repo_qc = RepoQc::new(ppkt_store_notebook_path, qc_report_list);
    Ok(repo_qc)
}