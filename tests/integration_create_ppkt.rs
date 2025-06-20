mod common;

use std::sync::Arc;

use ontolius::ontology::csr::FullCsrOntology;
use ga4ghphetools::PheTools;
use rstest::rstest;
use common::hpo;
use common::matrix;
use zip::result;

use crate::common::one_case_matrix;




#[rstest]
fn create_ppkt_1(
    one_case_matrix: Vec<Vec<String>>, 
    hpo: Arc<FullCsrOntology>,
) {
    let mut phetools = PheTools::new(hpo);
    assert_eq!(3, one_case_matrix.len()); // original matrix has headers and four data rows
    let original_matrix = one_case_matrix.clone();
    let res = phetools.load_matrix(one_case_matrix);
    assert!(res.is_ok());
    let hpo_version = "2025-05-31";
    /*
    TODO - finish test once the variant validator interface is finished
    let ppkt_list = phetools.export_phenopackets().unwrap();
    assert_eq!(1, ppkt_list.len());
    let ppkt = ppkt_list[0].clone();
    assert_eq!("PMID29198722pArg913TerAffectedIndividual1", &ppkt.id);
     */
    //println!("{:?}", ppkt);
}