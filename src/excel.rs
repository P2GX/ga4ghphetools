//! This module contains utilities to read the contents of a phenopacket-input-formated Excel fule
//!
//! It ingests an Excel file and returns a DataFrame containing the contents of the file.
//! It throws an error if there are syntactic errors in the input file.


use calamine::{open_workbook, Reader, Xlsx, XlsxError};
use std::error::Error;


pub fn read_excel_to_dataframe(file_path: &str) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
    let mut workbook: Xlsx<_> = open_workbook(file_path)
        .map_err(|e: XlsxError| format!("Could not open Excel file at '{}': {}", file_path, e.to_string()))?;
    let range = workbook
        .worksheet_range("Sheet1")
        .map_err(|e: XlsxError| format!("Error reading workbook: {}",e.to_string()))?;
    let mut row_iter = range.rows(); // Create a single iterator over the rows
    let first_row = row_iter.next().ok_or_else(|| calamine::Error::Msg("No data in the worksheet"))?;
    let first_row_headers: Vec<String> = first_row.iter().map(|cell| cell.to_string()).collect();
    let second_row = row_iter.next().ok_or_else(|| calamine::Error::Msg("No data in the worksheet"))?;
    let second_row_headers: Vec<String> = second_row.iter().map(|cell| cell.to_string()).collect();
    let n1 = first_row_headers.len();
    let n2 = second_row_headers.len();
    if n1 != n2 {
        return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidInput, format!("Malformed headers: expected {} fields, got {}", n2, n1))) as Box<dyn Error>);
    }
   
    let mut list_of_rows: Vec<Vec<String>> = vec![];
    list_of_rows.push(first_row_headers);
    list_of_rows.push(second_row_headers);
    // Now, iterate over the remaining rows
    for (_, row) in row_iter.enumerate() {
        let row_data: Vec<String> = row.iter().map(|cell| cell.to_string()).collect();
        //println!("Row {}: {:?}", i + 3, row_data); 
        if row_data.len() != n1 {
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidInput, format!("Malformed line:: expected {} fields, got {}", n1, row_data.len()))) as Box<dyn Error>);
        }
        list_of_rows.push(row_data);
    }
    return Ok(list_of_rows);
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

