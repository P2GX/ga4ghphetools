mod common;

use std::sync::Arc;

use ontolius::ontology::csr::FullCsrOntology;
use rphetools::PheTools;
use rstest::rstest;
use common::hpo;
use common::matrix;

/// Make sure that our test matrix is valid before we start changing fields to check if we pick up errors
#[rstest]
fn test_valid_input(matrix: Vec<Vec<String>>, hpo: Arc<FullCsrOntology>) {
    let mut phetools = PheTools::new(hpo);
    let res = phetools.load_matrix(matrix);
    assert!(res.is_ok());
}


/// Make sure we do not inadvertently change anything by loading the matrix
#[rstest]
fn check_round_trip(matrix: Vec<Vec<String>>, hpo: Arc<FullCsrOntology>) {
    let mut phetools = PheTools::new(hpo);
    let original_matrix = matrix.clone();
    let res = phetools.load_matrix(matrix);
    assert!(res.is_ok());
    let loaded_matrix = phetools.get_string_matrix().unwrap();
    assert_eq!(original_matrix, loaded_matrix);
}


/// Check whether trying to set an invalid value leads to an Error
#[rstest]
#[case(0,17, "+", "Malformed entry for Failure to thrive (HP:0001508): '+'")]
fn check_setting_invalid_value_single_test(
    matrix: Vec<Vec<String>>, 
    hpo: Arc<FullCsrOntology>,
    #[case] i: usize, 
    #[case] j: usize, 
    #[case] value: &str,
    #[case] error_msg: &str) {
    let mut phetools = PheTools::new(hpo);
    let original_matrix = matrix.clone();
    let res = phetools.load_matrix(matrix);
    assert!(res.is_ok());
    let result = phetools.set_value(i, j, value);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(error_msg, err);
}



/// Check whether trying to set an invalid value leads to an Error
#[rstest]
#[case(0,0, "PMD123", "Invalid CURIE with no colon: 'PMD123'")]
#[case(0,1, "A Variant in ZSWIM6 ", "Trailing whitespace in 'A Variant in ZSWIM6 '")]
#[case(0,1, "A Variant  in ZSWIM6", "Consecutive whitespace in 'A Variant  in ZSWIM6'")]
#[case(0,2, "Individual/A", "Forbidden character '/' found in label 'Individual/A'")]
#[case(0,2, "Individual.A", "Forbidden character '.' found in label 'Individual.A'")]
#[case(0,2, "Individual(A)", "Forbidden character '(' found in label 'Individual(A)'")]
#[case(0,4, "OMIM617865", "Invalid CURIE with no colon: 'OMIM617865'")]
#[case(0,4, "OMIM:17865", "OMIM identifiers must have 6 digits: 'OMIM:17865'")]
#[case(0,4, "MONDO76617865", "Invalid CURIE with no colon: 'MONDO76617865'")]
#[case(0,5, "Neurodevelopmental disorder ", "Trailing whitespace in 'Neurodevelopmental disorder '")]
#[case(0,5, "Neurodevelopmental  disorder", "Consecutive whitespace in 'Neurodevelopmental  disorder'")]
#[case(0,5, "", "Value must not be empty")]
#[case(0,6, "", "Empty CURIE")]
#[case(0,6, "HGNC:29316 ", "Contains stray whitespace: 'HGNC:29316 '")]
#[case(0,6, "HGNY:29316", "HGNC id has invalid prefix: 'HGNY:29316'")]
#[case(0,7, "", "Value must not be empty")]
#[case(0,7, "ZSWIM6 ", "Trailing whitespace in 'ZSWIM6 '")]
#[case(0,8, "", "Value must not be empty")]
#[case(0,8, "NM_020928", "Transcript 'NM_020928' is missing a version")]
#[case(0,9, "", "Value must not be empty")]
#[case(0,9, "c.2737C >T", "Malformed allele 'c.2737C >T'")]
#[case(0,9, "c.2737C > T", "Malformed allele 'c.2737C > T'")]
#[case(0,9, "2737C>T", "Malformed allele '2737C>T'")]
#[case(0,9, "c2737C>T", "Malformed allele 'c2737C>T'")]
#[case(0,9, "c.2737CT", "Malformed allele 'c.2737CT'")]
#[case(0,10, "", "Value must not be empty")]
#[case(0,10, "nan", "Malformed allele_2 field: 'nan'")]
#[case(0,10, "2737C>T", "Malformed allele_2 field: '2737C>T'")]
#[case(0,12, "Infantileonset", "Malformed age_of_onset 'Infantileonset'")]
#[case(0,12, "", "age_of_onset must not be empty")]
#[case(0,13, "Infantileonset", "Malformed age_at_last_encounter 'Infantileonset'")]
#[case(0,13, "", "age_at_last_encounter must not be empty")]
#[case(0,14, "", "deceased must not be empty")]
#[case(0,14, "Yes", "Malformed deceased entry: 'Yes'")]
#[case(0,15, "", "sex must not be empty")]
#[case(0,15, "male", "Malformed entry in sex field: 'male'")]
#[case(0,16, "", "HPO (separator) must not be empty")]
#[case(0,16, "nan", "Malformed HPO (separator) entry: 'nan'")]
#[case(0,17, "+", "Malformed entry for Failure to thrive (HP:0001508): '+'")]
#[case(0,17, "Observed", "Malformed entry for Failure to thrive (HP:0001508): 'Observed'")]
#[case(0,17, "-", "Malformed entry for Failure to thrive (HP:0001508): '-'")]
#[case(0,17, "exc", "Malformed entry for Failure to thrive (HP:0001508): 'exc'")]
fn check_setting_invalid_value(
    matrix: Vec<Vec<String>>, 
    hpo: Arc<FullCsrOntology>,
    #[case] i: usize, 
    #[case] j: usize, 
    #[case] value: &str,
    #[case] error_msg: &str) {
    let mut phetools = PheTools::new(hpo);
    let original_matrix = matrix.clone();
    let res = phetools.load_matrix(matrix);
    assert!(res.is_ok());
    let result = phetools.set_value(i, j, value);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(error_msg, err);
}


/// Check that setting a valid value does not lead to an Error
#[rstest]
#[case(0,17, "")]
fn check_setting_valid_value_single_test(
    matrix: Vec<Vec<String>>, 
    hpo: Arc<FullCsrOntology>,
    #[case] i: usize, 
    #[case] j: usize, 
    #[case] value: &str) {
    let mut phetools = PheTools::new(hpo);
    let original_matrix = matrix.clone();
    let res = phetools.load_matrix(matrix);
    assert!(res.is_ok());
    let result = phetools.set_value(i, j, value);
    assert!(result.is_ok());

}



/// Check that setting a valid value does not lead to an Error
#[rstest]
#[case(0,0, "PMID:123")]
#[case(0,1, "A Variant in ZSWIM6")]
#[case(0,2, "Individual:A")]
#[case(0,2, "Individual A")]
#[case(0,2, "Individual_A")]
#[case(0,4, "OMIM:617865")]
#[case(0,4, "MONDO:76617865")]
#[case(0,5, "Neurodevelopmental disorder")]
#[case(0,6, "HGNC:29316")]
#[case(0,7, "ZSWIM6")]
#[case(0,8, "NM_020928.42")]
#[case(0,8, "NM_020928.1")]
#[case(0,9, "c.2737C>T")]
#[case(0,9, "DEL: deletion exon 5")]
#[case(0,10, "INV: inversion whole gene")]
#[case(0,10, "TRANSL: trans(chr2q1, chr4p2")]
#[case(0,10, "na")]
#[case(0,12, "Infantile onset")]
#[case(0,13, "Infantile onset")]
#[case(0,14, "yes")]
#[case(0,14, "no")]
#[case(0,15, "U")]
#[case(0,15, "F")]
#[case(0,16, "na")]
#[case(0,17, "")]
#[case(0,17, "observed")]
#[case(0,17, "excluded")]
#[case(0,17, "P24Y")]
#[case(0,17, "")]
fn check_setting_valid_value(
    matrix: Vec<Vec<String>>, 
    hpo: Arc<FullCsrOntology>,
    #[case] i: usize, 
    #[case] j: usize, 
    #[case] value: &str) {
    let mut phetools = PheTools::new(hpo);
    let original_matrix = matrix.clone();
    let res = phetools.load_matrix(matrix);
    assert!(res.is_ok());
    let result = phetools.set_value(i, j, value);
    assert!(result.is_ok());
}