mod common;

use std::sync::Arc;

use ontolius::ontology::csr::FullCsrOntology;
use ga4ghphetools::PheTools;
use rstest::rstest;
use common::hpo;
use common::matrix;
use zip::result;

/// Make sure that our test matrix is valid before we start changing fields to check if we pick up errors
#[rstest]
fn test_valid_input(matrix: Vec<Vec<String>>, hpo: Arc<FullCsrOntology>) {
    let mut phetools = PheTools::new(hpo);
    let res = phetools.load_matrix(matrix);
    assert!(res.is_ok());
}









