//! This module contains utilities to read the contents of a phenopacket-input-formated Excel fule
//!
//! It ingests an Excel file and returns a DataFrame containing the contents of the file.
//! It throws an error if there are syntactic errors in the input file.

use calamine::{open_workbook, Reader, Xlsx, XlsxError};



use crate::dto::etl_dto::{ColumnDto, ColumnTableDto};



/// Get a matrix of strings from the first worksheet in an Excel file.
fn get_list_of_rows_from_excel(file_path: &str) -> Result<Vec<Vec<String>>, String>  {
    let mut workbook: Xlsx<_> = open_workbook(file_path).map_err(|e: XlsxError| {
        format!(
            "Could not open Excel file at '{}': {}",
            file_path,
            e
        )
    })?;
    let sheet_names = workbook.sheet_names().to_owned();
    let first_sheet_name = match sheet_names.first() {
        Some(first_name) => first_name,
        None => {return Err(format!("Could not get name of first worksheet from {file_path}"));},
    };
    let mut list_of_rows: Vec<Vec<String>> = vec![];
    let range = workbook
        .worksheet_range(first_sheet_name)
        .map_err(|e: XlsxError| format!("Error reading workbook: {}", e))?;
    let row_iter = range.rows(); // Create a single iterator over the rows
    for row in row_iter {
        let row_data: Vec<String> = row.iter()
            .map(|cell| {
                let s = cell.to_string();
                if s.is_empty() {
                    "na".to_string()
                } else {
                    s
                }
            })
            .collect();
        list_of_rows.push(row_data);
    }
    Ok(list_of_rows)
}


/// Reads in data from the initial format version of phenopacket store (up to version 0.1.24)
/// This function replaces any empty cells in the data with na
pub fn read_excel_to_dataframe(file_path: &str) -> Result<Vec<Vec<String>>, String> {
    let matrix = get_list_of_rows_from_excel(file_path)?;
    if matrix.len() < 3 {
        return Err(format!("Input file with insufficient rows ({})", matrix.len()));
    }
    let row0 = &matrix[0];
    let row1 = &matrix[1];
    

    let n1 = row0.len();
    let n2 = row1.len();
    if n1 != n2 {
        return Err(
            format!("Malformed headers: expected {} fields, got {}", n2, n1));
    }
    for row in matrix.iter().skip(2) {
        let row_len = row.len();
        if row_len != n1 {
            return Err(format!(
                    "Malformed line:: expected {} fields, got {}",
                    n1,
                    row_len
                ));
        }
    }
    let matrix_without_na = matrix.into_iter()
        .enumerate()
        .map(|(i, row)| {
            if i < 2 {
                row // keep first two rows as-is
            } else {
                row.into_iter()
                    .map(|cell| if cell.trim().is_empty() { "na".to_string() } else { cell })
                    .collect()
            }
        })
        .collect();
    Ok(matrix_without_na)
}


/// Function to input an external Excel file for ETL purposes.
/// Let's use a separate file for now TODO - consider sharing code with above once API/strategy are clear.
/// We may want to allow some automatic error correction here, we can correct minor errors in the GUI!
pub fn read_external_excel_to_df(file_path: &str, row_based: bool) -> Result<ColumnTableDto, String> {
    let mut matrix = get_list_of_rows_from_excel(file_path)?;
    if matrix.len() < 3 {
        return Err(format!("Input file with insufficient rows ({})", matrix.len()));
    }
    if ! row_based {
        let row_len = matrix[0].len();
        let mut transposed = vec![Vec::with_capacity(matrix.len()); row_len];

        for row in matrix {
            for (i, val) in row.into_iter().enumerate() {
                transposed[i].push(val);
            }
        }
        matrix = transposed;
    }
    let headers = matrix[0].clone();
    let data_rows = &matrix[1..];
    let total_rows = data_rows.len();
    let total_columns = headers.len();
    let mut columns: Vec<ColumnDto> = headers
        .iter()
        .map(|h| ColumnDto {
            column_type: crate::dto::etl_dto::EtlColumnType::Raw,
            transformed: false,
            header: h.clone(),
            values: Vec::with_capacity(total_rows),
        })
        .collect();

    for row in data_rows {
        for (col_index, col) in columns.iter_mut().enumerate() {
            let value = row.get(col_index).cloned().unwrap_or_default();
            col.values.push(value);
        }
    }
    Ok(ColumnTableDto {
        file_name: file_path.to_string(),
        columns,
    })
}

// region:    --- Tests

#[cfg(test)]
mod tests {
    type Error = Box<dyn std::error::Error>;
    type Result<T> = core::result::Result<T, Error>; // For tests.

    use super::*;

    #[test]
    fn test_invalid_file_path() -> Result<()> {
        let fake_path = "wrong/path/template.xlsx";
        let result = read_excel_to_dataframe(fake_path);
        assert!(result.is_err());
        let error_msg = result.err().unwrap().to_string();
        let expected = "Could not open Excel file at 'wrong/path/template.xlsx': I/O error: No such file or directory (os error 2)";
        assert_eq!(expected, error_msg);
        Ok(())
    }
}

// endregion: --- Tests
