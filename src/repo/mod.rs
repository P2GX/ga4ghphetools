//! Repository Q/C and Stats
//! 
//! 
//! 

use std::path::PathBuf;

use crate::repo::{gpt_repository::GptRepository, repo_qc::RepoQc};


mod cohort_dir;
mod cohort_qc;
pub mod dashboard_data;
mod disease_qc;
mod gpt_repository;
pub mod qc_report;
pub mod repo_qc;



pub fn get_repo_qc(path: &PathBuf) -> Result<RepoQc, String> {
     let repo = GptRepository::new(path);
     repo.repo_qc()
}