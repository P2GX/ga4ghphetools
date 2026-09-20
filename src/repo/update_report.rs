//! A structure to report the results of updating the phenopacket store repository to a new HPO version.
//! That is, we update HPO term identifiers and labels to reflect changes made in the latext HPO version.

use std::path::PathBuf;

pub struct UpdateReport {
    pub directory: String,
    pub processed: usize,
    pub updated: usize,
}

impl UpdateReport {
    pub fn new(path: &PathBuf) -> Self {
        Self {
            directory: path.to_string_lossy().to_string(),
            updated: 0,
            processed: 0
        }
    }

    pub fn updated(&mut self) {
        self.updated += 1;
        self.processed();
    }

    pub fn processed(&mut self) {
        self.processed += 1;
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_new_initializes_zeroed_counts() {
        let path = PathBuf::from("/some/repo/path");
        let report = UpdateReport::new(&path);
        assert_eq!(report.processed, 0);
        assert_eq!(report.updated, 0);
    }

    #[test]
    fn test_new_stores_directory_as_string() {
        let path = PathBuf::from("/some/repo/path");
        let report = UpdateReport::new(&path);
        assert_eq!(report.directory, "/some/repo/path");
    }

    #[test]
    fn test_processed_increments_processed_only() {
        let path = PathBuf::from("/tmp");
        let mut report = UpdateReport::new(&path);
        report.processed();
        assert_eq!(report.processed, 1);
        assert_eq!(report.updated, 0);
    }

    #[test]
    fn test_processed_multiple_calls_accumulate() {
        let path = PathBuf::from("/tmp");
        let mut report = UpdateReport::new(&path);
        report.processed();
        report.processed();
        report.processed();
        assert_eq!(report.processed, 3);
        assert_eq!(report.updated, 0);
    }

    #[test]
    fn test_updated_increments_both_updated_and_processed() {
        // updated() calls processed() internally, so both counters move together.
        let path = PathBuf::from("/tmp");
        let mut report = UpdateReport::new(&path);
        report.updated();
        assert_eq!(report.updated, 1);
        assert_eq!(report.processed, 1);
    }

    #[test]
    fn test_updated_multiple_calls_keep_counts_in_sync() {
        let path = PathBuf::from("/tmp");
        let mut report = UpdateReport::new(&path);
        report.updated();
        report.updated();
        assert_eq!(report.updated, 2);
        assert_eq!(report.processed, 2);
    }

    #[test]
    fn test_mixed_processed_and_updated_calls() {
        let path = PathBuf::from("/tmp");
        let mut report = UpdateReport::new(&path);
        report.processed(); // processed: 1, updated: 0
        report.updated();   // processed: 2, updated: 1
        report.processed(); // processed: 3, updated: 1
        report.updated();   // processed: 4, updated: 2
        assert_eq!(report.processed, 4);
        assert_eq!(report.updated, 2);
    }
}