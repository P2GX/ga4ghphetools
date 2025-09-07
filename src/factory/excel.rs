//! This module contains utilities to read the contents of Excel files.
//! 
//! The Phenopacket Store project (PMID:39394689) was initially developed using
//! pyphetools, a Python package that used templated Excel files as serialization medium.
//! We are transitioning away from this format and replacing these Excel files with phetool JSON templates (CohortData), but
//! since there are several hundred cohorts, the migration will take some time. 
//! 
//! Additionally, we provide code to help transform an Excel file from other sources into phenopacket data.
//! The strategy is to first ingest the Excel file oriented such that the individuals are in rows and the
//! specific data is in columns. Then, the Phenoboard GUI can be used to transform this data one column at a time.
//!


use calamine::{open_workbook, Reader, Xlsx, XlsxError};

use crate::dto::etl_dto::{ColumnDto, ColumnTableDto};


/// Reads the **first worksheet** from an Excel `.xlsx` file and returns its contents
/// as a 2D vector of strings (`Vec<Vec<String>>`).
///
/// # Behavior
/// - Opens the Excel file at the given `file_path`.
/// - Finds the first worksheet in the workbook.
/// - Iterates over all rows and cells in the sheet.
/// - Converts each cell to a string:
///   - Empty cells are replaced with the literal string `"na"`.
///   - Non-empty cells are converted with `to_string()`.
/// - Collects each row into a `Vec<String>`, and all rows into a `Vec<Vec<String>>`.
///
/// # Errors
/// Returns `Err(String)` in the following cases:
/// - The file cannot be opened as an Excel workbook (I/O error or invalid file format).
/// - The workbook has no worksheets.
/// - The first worksheet cannot be read successfully.
///
/// # Returns
/// On success, returns `Ok(rows)` where `rows` is a matrix of strings, one per row
/// in the worksheet. Each inner `Vec<String>` represents the values in one row.
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


/// Reads an Excel file in the **legacy phenopacket-store format** 
/// and validates it into a structured 2D string matrix (`Vec<Vec<String>>`).
///
/// This function is specialized for early phenopacket-store input files,
/// which are expected to have:
/// - At least **3 rows** total (2 header rows + data).
/// - The **first two rows** (`row0`, `row1`) defining the column headers.
/// - Each data row having the **same number of fields** as the headers.
///
/// # Behavior
/// - Delegates to [`get_list_of_rows_from_excel`] to extract raw cell values.
/// - Ensures the matrix has at least 3 rows; otherwise returns an error.
/// - Ensures the first two rows have the same number of fields (consistent headers).
/// - Ensures each subsequent row has the same number of fields as the headers.
/// - Replaces any **empty cells** (only in data rows) with the literal `"na"`.
///
/// # Errors
/// Returns `Err(String)` in the following cases:
/// - The Excel file cannot be opened or parsed (from [`get_list_of_rows_from_excel`]).
/// - The file has fewer than 3 rows.
/// - The two header rows have different numbers of fields.
/// - Any data row has a different number of fields than the headers.
///
/// # Returns
/// On success, returns `Ok(matrix)` where:
/// - `matrix[0]` is the first header row (unchanged).
/// - `matrix[1]` is the second header row (unchanged).
/// - `matrix[2..]` are data rows, with empty cells normalized to `"na"`.
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


/// Reads an **external Excel file** for ETL purposes and converts it into
/// a `ColumnTableDto` suitable for further transformation in the ETL pipeline.
///
/// This function is intended for external Excel files (**Not the internal phenopacket-store templates**).
/// The output separates columns into DTOs that include:
/// - `column_type` (initially set to `Raw`)
/// - `transformed` flag (initially `false`)
/// - `header` string
/// - `values` vector for each column
///
/// # Parameters
/// - `file_path`: Path to the Excel file to read.
/// - `row_based`: Determines whether the Excel file is already **row-based** (`true`) or **column-major** (`false`).
///   - If `false`, the function will **transpose** the matrix so that each vector represents a column.
///
/// # Behavior
/// 1. Reads the first worksheet from the Excel file via [`get_list_of_rows_from_excel`].
/// 2. Ensures the file has at least 3 rows; otherwise returns an error.
/// 3. Optionally transposes the matrix if `row_based` is `false`.
/// 4. Uses the first row as headers.
/// 5. Remaining rows are treated as data, mapped into `ColumnDto` structs.
/// 6. Each column gets a `Vec<String>` of values, maintaining order.
///
/// # Returns
/// On success, returns a `ColumnTableDto` containing:
/// - `file_name`: the input file path as string
/// - `columns`: vector of `ColumnDto`, each representing a column with header and values.
///
/// # Errors
/// Returns `Err(String)` if:
/// - The file cannot be opened or read (from `get_list_of_rows_from_excel`)
/// - The file has fewer than 3 rows
pub fn read_external_excel_to_df(
    file_path: &str, 
    row_based: bool) 
-> Result<ColumnTableDto, String> {
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
        .map(|h| ColumnDto::new_raw(h, total_rows))
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
