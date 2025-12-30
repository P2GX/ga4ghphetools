use crate::repo::qc_report::QcReport;




pub struct DashboardData {
    pub all_reports: Vec<QcReport>,
    pub total_errors: usize,
}

impl DashboardData {
    pub fn new(reports: Vec<QcReport>) -> Self {
        let total_errors = reports.iter().filter(|r| !r.is_ok).count();
        Self { all_reports: reports, total_errors }
    }
}