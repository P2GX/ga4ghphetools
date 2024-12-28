//! This module contains utilities to read the contents of a phenopacket-input-formated Excel fule
//!
//! It ingests an Excel file and returns a DataFrame containing the contents of the file.
//! It throws an error if there are syntactic errors in the input file.


use calamine::{Reader, open_workbook, Xlsx};
use std::error::Error;
use std::fmt;


/// There are two header lines. For the static fields, the information is only needed from the
/// first header. For the HPO columns, the label is shown in the first header and the HPO id is
/// shown in the second field. The purpose of this struct is simply to record the strings in
/// both rows so that we can do some Q/C prior to starting to create the DataFrame object.
struct HeaderDuplet {
    h1: String,
    h2: String,
}

impl HeaderDuplet {

    pub fn new(header1: &str ,  header2: &str) -> Self {
        HeaderDuplet {
            h1: header1.to_string(),
            h2: header2.to_string(),
        }
    }
}

impl fmt::Display for HeaderDuplet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "HeaderDuplet(h1: {}, h2: {})", self.h1, self.h2)
    }
}


pub fn read_excel_to_dataframe(file_path: &str) -> Result<(), Box<dyn Error>> {
    let mut workbook: Xlsx<_> = open_workbook(file_path).expect("Cannot open file at {file_path}");
    let range = workbook
        .worksheet_range("Sheet1")
        .map_err(|_| calamine::Error::Msg("Cannot find Sheet1")).expect("Could not find Excel Sheet 1");
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
    let mut header_duplets: Vec<HeaderDuplet> = vec![];


    println!("Headers:");
    for i in 0..(n1-1) {
        header_duplets.push(HeaderDuplet::new(&first_row_headers[i], &second_row_headers[i]));
        println!("{} ", header_duplets[i]); // Print each column name (header)
    }
    if let Err(res) = qc_list_of_header_items(&header_duplets) {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, res).into());
    }
    let mut list_of_rows: Vec<Vec<String>> = vec![];
    // Now, iterate over the remaining rows
    for (i, row) in row_iter.enumerate() {
        let row_data: Vec<String> = row.iter().map(|cell| cell.to_string()).collect();
        println!("Row {}: {:?}", i + 3, row_data); 
        if row_data.len() != n1 {
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidInput, format!("Malformed line:: expected {} fields, got {}", n1, row_data.len()))) as Box<dyn Error>);
        }
        list_of_rows.push(row_data);
    }
    // When we get here, we can create our dataframe object as a list of IndividualRows.
    // Each individual row has a series of constant items as well as a list of HPO-related cells
   
   
    Ok(())
}



fn qc_list_of_header_items(header_duplets: &Vec<HeaderDuplet>) -> Result<(), String> {
    // check each of the items in turn
    let expected_h1_fields = vec!["PMID", "title", "individual_id", "comment", "disease_id", 
    "disease_label", "HGNC_id", "gene_symbol", "transcript", "allele_1", "allele_2", 
    "variant.comment", "age_of_onset", "age_at_last_encounter", "deceased", "sex", "HPO"];
    let expected_h2_fields = vec!["CURIE", "str", "str", "optional", "CURIE", "str", "CURIE", 
     "str", "str", "str", "str", "optional", "age", "age", "yes/no/na", "M:F:O:U", "na"];
    let h1_len = expected_h1_fields.len();
    if h1_len != expected_h2_fields.len() {
        // should never happen
        return Err("header field definitions malformend".to_string());
    }
    let mut errors: Vec<String> = vec![];
    for (i, duplet) in header_duplets.into_iter().enumerate() {
        if i < h1_len && duplet.h1 != expected_h1_fields[i] {
            errors.push(format!("Malformed header: expected {}, got {}", 
                            expected_h1_fields[i], 
                            duplet.h1))
        } else {
            // if i > h1_len, then we have the HPO fields
            // todo check HPO validity
        }
        if i < h1_len && duplet.h2 != expected_h2_fields[i] {
            errors.push(format!("Malformed header (row 2): expected {}, got {}", 
                            expected_h1_fields[i], 
                            duplet.h1))
        } 

    }
    if errors.len() > 0 {
        let s = format!("Could not parse headers: {}", errors.join(", "));
        return Err(s);
    }
    Ok(())
}



#[cfg(test)]
mod test {
    use super::*;


    #[test]
    fn test_header_duplet_ctor() {
        let hd = HeaderDuplet::new("Arachnodactly", "HP:0001166");
        let expected_header1 = String::from("Arachnodactly");
        let expected_header2 = String::from("HP:0001166");
        assert_eq!(expected_header1, hd.h1);
        assert_eq!(expected_header2, hd.h2);
    }

}
