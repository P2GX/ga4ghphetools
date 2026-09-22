use std::path::{Path, PathBuf};

use crate::cohort_qc::cohort_dir::CohortDir;

pub struct CohortDirIter {
    dirs: std::vec::IntoIter<PathBuf>,
}

impl CohortDirIter {
    pub fn new(store_path: &Path) -> std::io::Result<Self> {
        let mut dirs: Vec<PathBuf> = std::fs::read_dir(store_path)?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_dir())
            .collect();
        dirs.sort(); // deterministic ordering
        Ok(Self { dirs: dirs.into_iter() })
    }
}

impl Iterator for CohortDirIter {
    type Item = CohortDir;

    fn next(&mut self) -> Option<Self::Item> {
        let path = self.dirs.next()?;
        Some(CohortDir::process_gene_directory(&path))
    }
}