mod common;

use std::sync::Arc;

use ga4ghphetools::factory::cohort_factory::CohortFactory;
use ontolius::ontology::csr::FullCsrOntology;
use rstest::rstest;
use common::hpo_fixture::hpo;
use common::matrix_fixtures::matrix;


use serde_json::Value;


/// Make sure that our test matrix is valid before we start changing fields to check if we pick up errors
#[rstest]
fn test_valid_input(matrix: Vec<Vec<String>>, hpo: Arc<FullCsrOntology>) {
    let res = CohortFactory::dto_from_mendelian_template(matrix, hpo.clone(), false,  |_p:u32,_q:u32|{/*  no progress bar for test*/});
    assert!(res.is_ok());
}


pub fn strip_phenopacket_defaults(root: &mut Value) {
    // Top-level `subject`
    if let Value::Object(root_map) = root {
        if let Some(Value::Object(subject)) = root_map.get_mut("subject") {
            // Remove karyotypic_sex if it's the unknown/default
            let drop_karyotype = match subject.get("karyotypic_sex") {
                Some(Value::String(s)) if s == "UNKNOWN_KARYOTYPE" => true,
                Some(Value::Number(n)) if n.as_i64() == Some(0) => true,
                _ => false,
            };
            if drop_karyotype {
                subject.remove("karyotypic_sex");
            }

            // If you truly want to drop survival_time_in_days==0 from subject (enable if applicable)
            if let Some(Value::Number(n)) = subject.get("survival_time_in_days") {
                if n.as_i64() == Some(0) {
                    subject.remove("survival_time_in_days");
                }
            }

            // If your schema puts survival time inside a nested object (uncomment as needed)
            if let Some(Value::Object(vs)) = subject.get_mut("vital_status") {
                if let Some(Value::Number(n)) = vs.get("survival_time_in_days") {
                    if n.as_i64() == Some(0) {
                        vs.remove("survival_time_in_days");
                    }
                }
            }
        }
    }
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

fn debug_matrix_comparison(
    mat1: &Vec<Vec<String>>,
    mat2: &Vec<Vec<String>>,
    n: usize,
) {
    for i in 0..mat1.len() {
        let mut row_ok = true;
        for j in 0..n {
            if mat1[i][j] != mat2[i][j] {
                println!("mat1[{}][{}] = '{}' != mat2 = '{}'", i, j,& mat1[i][j], &mat2[i][j] );
                row_ok = false;
            }
        }
        if row_ok {
            println!("Row {} OK", i);
        }
    }
    let n = mat1.len();
    let m = mat2.len();
    if m != n {
        println!("mat1 len = {n} mat2 len ={m}")
    }
    
    for i in 0..n {
        if i < n {
        println!("{}", mat1[i].join("\t"));
        }
        if i < m {
        println!("{}", mat2[i].join("\t"));
        }
        println!("");
    }


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
