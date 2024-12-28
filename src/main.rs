// src/main.rs

mod excel;
mod dataframe;

fn main() {
    let file_path = "/Users/robin/GIT/phenopacket-store/notebooks/CD28/input/CD28_IMD123_individuals.xlsx";
    
    let _ = excel::read_excel_to_dataframe(file_path);
     
}
