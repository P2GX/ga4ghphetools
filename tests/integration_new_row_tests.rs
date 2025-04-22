mod common;

use std::sync::Arc;

use ontolius::ontology::csr::FullCsrOntology;
use rphetools::dto::case_dto::CaseDto;
use rphetools::dto::hpo_term_dto::HpoTermDto;
use rphetools::PheTools;
use rstest::rstest;
use common::hpo;
use common::matrix;
use common::hpo_dto_list_1;
use common::case_5_dto;
use zip::result;

/// Make sure that our test matrix is valid before we start changing fields to check if we pick up errors
#[rstest]
fn test_valid_input(matrix: Vec<Vec<String>>, hpo: Arc<FullCsrOntology>) {
    let mut phetools = PheTools::new(hpo);
    let res = phetools.load_matrix(matrix);
    assert!(res.is_ok());
}

/// We expect that the first 17 columns (non-HPO columns) remain equal for the first 6 rows
fn first_n_columns_equal(
    mat1: &Vec<Vec<String>>,
    mat2: &Vec<Vec<String>>,
    n: usize,
) -> bool {
    mat1.iter()
        .zip(mat2.iter())
        .all(|(row1, row2)| row1.iter().take(n).eq(row2.iter().take(n)))
}

/// New HPO columns should be all "na" except for the last entry, which is taken from the DTO
fn validity_of_new_column(
    header1: &str, 
    header2: &str, 
    new_entry: &str, 
    matrix: &Vec<Vec<String>>) -> Result<(), String> {
    let col = get_index_of_column(&matrix, header1)?;
    if matrix[0][col] != header1 || matrix[1][col] != header2 {
        return Err(format!("Expected header1={}/header2={} but got {}/{}",
            header1, header2, matrix[0][col], matrix[1][col]));  
    }
    if matrix[2..matrix.len() - 1].iter().any(|row| row[col] != "na") {
        return Err(format!("Expected na but go other value"));
    }
    if matrix[matrix.len() - 1][col] != new_entry {
        return Err(format!("Last column entry{} but expected {}",
            matrix[matrix.len() - 1][col], new_entry));
    }
    Ok(())
}


fn get_index_of_column(matrix: &Vec<Vec<String>>, colname: &str) 
    -> Result<usize, String> {
        for i in 0..matrix[0].len() {
            if matrix[0][i] == colname {
                return Ok(i);
            }
        }
        Err(format!("Could not find column'{}' in matrix", colname))
    }

fn validity_of_previous_column(
    prev_matrix: &Vec<Vec<String>>, 
    new_matrix: &Vec<Vec<String>>, 
    colname: &str, 
    new_entry: &str) -> Result<(), String> 
{
    let new_col = get_index_of_column(&new_matrix, colname)?;
    let prev_col = get_index_of_column(&prev_matrix, colname)?;
    for i in 0..prev_matrix.len() {
        if prev_matrix[i][prev_col] != new_matrix[i][new_col] {
            return Err(format!("Expected prev_matrix[{}][{}]={}  but got new_matrix[{}][{}]={}",
                i, prev_col, prev_matrix[i][prev_col], i, new_col, new_matrix[i][new_col]));  
        }
    }
    if new_matrix[new_matrix.len() - 1][new_col] != new_entry {
        return Err(format!("Expected new_matrix[{}][{}]={} but got {}",
            new_matrix.len() - 1, new_col, new_matrix[new_matrix.len() - 1][new_col], new_entry));  
    }
    return Ok(());
}


#[rstest]
fn add_new_row_test_1(
    matrix: Vec<Vec<String>>, 
    hpo: Arc<FullCsrOntology>,
    case_5_dto: CaseDto,
    hpo_dto_list_1: Vec<HpoTermDto>
) {
    let mut phetools = PheTools::new(hpo);
    assert_eq!(6, matrix.len()); // original matrix has headers and four data rows
    let original_matrix = matrix.clone();
    let res = phetools.load_matrix(matrix);
    assert!(res.is_ok());
    let dto_cloned = case_5_dto.clone(); // needed only for testing
    let res = phetools.add_row_with_hpo_data(case_5_dto, hpo_dto_list_1);
    assert!(res.is_ok());
    // Check that the constant items are what we want
    let new_matrix = phetools.get_string_matrix().expect("Could not unwrap matrix with added row");
    assert_eq!(7, new_matrix.len());
    // Check that the first six rows are identical to the original matrix
    // Note that we can only do this for the constant columns, because we have added new HPO columns
    let first_six_new_rows: Vec<Vec<String>> = new_matrix.iter().take(6).cloned().collect();
    let are_equal = first_n_columns_equal(&original_matrix, &first_six_new_rows, 17);
    assert!(are_equal);
    // now check that the non-HPO entries in the new line are OK
    let seventh_row = new_matrix[6].clone();
    assert_eq!(dto_cloned.pmid, seventh_row[0]);
    assert_eq!(dto_cloned.title, seventh_row[1]);
    assert_eq!(dto_cloned.individual_id, seventh_row[2]);
    assert_eq!(dto_cloned.comment, seventh_row[3]);
    assert_eq!(dto_cloned.allele_1, seventh_row[9]);
    assert_eq!(dto_cloned.allele_2, seventh_row[10]);
    assert_eq!(dto_cloned.variant_comment, seventh_row[11]);
    assert_eq!(dto_cloned.age_of_onset, seventh_row[12]);
    assert_eq!(dto_cloned.age_at_last_encounter, seventh_row[13]);
    assert_eq!(dto_cloned.deceased, seventh_row[14]);
    assert_eq!(dto_cloned.sex, seventh_row[15]);
    assert_eq!("na", seventh_row[16]); // constant HPO separator column
    // Now check the HPO columns
    // The DTO added: thick_eye_brow: excluded; grand_mal: observed; strabismus: observed; esotropia_observed
    validity_of_new_column("Strabismus", "HP:0000486", "observed", &new_matrix).expect("Strabismus observed in DTO");
    validity_of_new_column("Esotropia", "HP:0000565", "observed", &new_matrix).expect("Esotropia observed in DTO"); 
    validity_of_previous_column(&original_matrix, &new_matrix, "Loss of ambulation", "na").expect("Loss of ambulation, no info in DTO thus - na");
    validity_of_previous_column(&original_matrix, &new_matrix, "Seizure", "na").expect("Seizure, no info in DTO thus - na");
    validity_of_new_column("Bilateral tonic-clonic seizure", "HP:0002069", "observed", &new_matrix).expect("Bilateral tonic-clonic seizure observed in DTO");
    validity_of_previous_column(&original_matrix, &new_matrix, "Ataxia", "na").expect("Ataxia, no info in DTO thus - na");
    validity_of_previous_column(&original_matrix, &new_matrix, "Tongue thrusting", "na").expect("Tongue thrusting, no info in DTO thus - na");
    validity_of_previous_column(&original_matrix, &new_matrix, "Happy demeanor", "na").expect("Happy demeanor, no info in DTO thus - na");
    validity_of_previous_column(&original_matrix, &new_matrix, "Hypertonia", "na").expect("Hypertonia, no info in DTO thus - na");
    validity_of_previous_column(&original_matrix, &new_matrix, "Failure to thrive", "na").expect("Failure to thrive, no info in DTO thus - na");
    validity_of_new_column("Thick eyebrow", "HP:0000574", "excluded", &new_matrix).expect("Thick eyebrow excluded in DTO");
    // If we get here, we have passed all tests!
    assert!(true);
}


///Check that all entries in a column are the same
fn check_data_entries_unique(
    new_matrix: &Vec<Vec<String>>, 
    colname: &str, 
    new_entry: &str) -> Result<(), String>
{
    let col = get_index_of_column(&new_matrix, colname)?;
    for i in 2..new_matrix.len() {
        if new_matrix[i][col] != new_entry {
            return Err(format!("Expected new_matrix[{}][{}]={}  but got {}",
                i, col, new_entry, new_matrix[i][col]));  
        }
    }
    Ok(())
}


/// Check that all entries in the constant, disease-gene-bundle block are identical
/// If so, then the new row contains the same values for
/// disease_id, disease_label, HGNC_id, gene_symbol, and transcript
#[rstest]
fn add_new_row_check_disease_gene_bundle(
    matrix: Vec<Vec<String>>, 
    hpo: Arc<FullCsrOntology>,
    case_5_dto: CaseDto,
    hpo_dto_list_1: Vec<HpoTermDto>
) {
    let mut phetools = PheTools::new(hpo);
    assert_eq!(6, matrix.len()); // original matrix has headers and four data rows
    let original_matrix = matrix.clone();
    let res = phetools.load_matrix(matrix);
    assert!(res.is_ok());
    let dto_cloned = case_5_dto.clone(); // needed only for testing
    let res = phetools.add_row_with_hpo_data(case_5_dto, hpo_dto_list_1);
    assert!(res.is_ok());
    // Check that the constant items are what we want
    let new_matrix = phetools.get_string_matrix().expect("Could not unwrap matrix with added row");
    assert_eq!(7, new_matrix.len());
    check_data_entries_unique(&new_matrix, "disease_id", "OMIM:617865").expect("Expected all entries to be 'OMIM:617865'");
    check_data_entries_unique(&new_matrix, "disease_label", "Neurodevelopmental disorder with movement abnormalities, abnormal gait, and autistic features")
        .expect("Expected all entries to be 'Neurodevelopmental disorder with movement abnormalities, abnormal gait, and autistic features'");
    check_data_entries_unique(&new_matrix, "HGNC_id", "HGNC:29316").expect("Expected all entries to be 'HGNC:29316'");
    check_data_entries_unique(&new_matrix, "gene_symbol", "ZSWIM6").expect("Expected all entries to be 'ZSWIM6'");
    check_data_entries_unique(&new_matrix, "transcript", "NM_020928.2").expect("Expected all entries to be 'NM_020928.2'");
    // if we get here, all tests were OK!
    assert!(true);
}

