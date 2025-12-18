use std::sync::Arc;

use ontolius::ontology::csr::FullCsrOntology;
#[cfg(feature = "excel_export")]
use rust_xlsxwriter::{Format, Workbook};

use crate::{dto::cohort_dto::CohortData, export::table_compare::{RowCounter, TableCompare}};


fn get_header_format() -> Format {
    Format::new()
        .set_bold()
        .set_background_color("#D3D3D3")  // Light gray background
        .set_align(rust_xlsxwriter::FormatAlign::Center)
}

fn get_subheader_format() -> Format {
    Format::new()
        .set_bold()
        .set_background_color("#A3A3A3")  // Light gray background
        .set_align(rust_xlsxwriter::FormatAlign::Center)
}

pub fn create_excel_with_merged_cells(out_filename: &str,
        cohort_1: CohortData,
        cohort_2: CohortData,
        hpo: Arc<FullCsrOntology>,
        threshold: usize) -> Result<(), String> {

    let tcompare = TableCompare::new(cohort_1, cohort_2, hpo)?;
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    worksheet.set_name("Counts").map_err(|e| e.to_string())?;
    let header_format = get_header_format();
    let subheader_format = get_subheader_format();
    // Header row
    let header_fields = RowCounter::get_header();
    for (i, hf) in header_fields.iter().enumerate() {
        worksheet.write_string_with_format(0, i as u16, hf, &header_format).map_err(|e|e.to_string())?;
    }
    let mut row_i: u32 = 1;
    for cat_counter in tcompare.get_category_counters() {
        if !cat_counter.over_threshold(threshold) {
            continue;
        }
        let sub_header = cat_counter.get_subheader().join("\t");
        worksheet.merge_range(row_i, 0, row_i, 4, &sub_header, &subheader_format).map_err(|e| e.to_string())?; 
        row_i += 1;
        for row in cat_counter.row_counter_list {
            if !row.over_threshold(threshold) {
                continue;
            }
            let count = tcompare.get_count(row.duplet());
            if count > threshold {
                let fields = row.get_row();
                for (i, f) in fields.iter().enumerate() {
                        worksheet.write_string(row_i, i as u16, f).map_err(|e|e.to_string())?;
                }
                row_i += 1;
            }
        }
    }
    // Now add a worksheet with all of the publications
    let worksheet2 = workbook.add_worksheet();
    worksheet2.set_name("Publications").map_err(|e| e.to_string())?;
    let citations = tcompare.get_citations();
    let header_fields = vec!["PMID".to_string(), "Title".to_string()];
    for (i, hf) in header_fields.iter().enumerate() {
        worksheet2.write_string_with_format(0, i as u16, hf, &header_format).map_err(|e|e.to_string())?;
    }
    for (i, cite) in citations.iter().enumerate() {
        let j: u32 = (i+1) as u32;
         worksheet2.write_string(j, 0 as u16, cite.pmid()).map_err(|e|e.to_string())?;
         worksheet2.write_string(j, 1 as u16, cite.title()).map_err(|e|e.to_string())?;
    }

    println!("Saving excel file to {out_filename}");
    workbook.save(out_filename).map_err(|e|e.to_string())?;
    Ok(())
}


            







    /*
    
     pub fn output_table(&self, output_path: &str, threshold: usize) -> Result<(), String> {
        let file = File::create(output_path).map_err(|e| e.to_string())?;
        let mut writer = BufWriter::new(file);
        let header = RowCounter::get_header().join("\t");
        writeln!(writer, "{}", header).map_err(|e| e.to_string())?;
        for cat_counter in self.category_map.values() {
            if !cat_counter.over_threshold(threshold) {
                continue;
            }
            let sub_header = cat_counter.get_subheader().join("\t");
            writeln!(writer, "{}", sub_header).map_err(|e| e.to_string())?;
            for row in &cat_counter.row_counter_list {
                if let Some(count) = self.total_term_count_map.get(row.duplet()) {
                    if *count > threshold {
                        let row_str = row.get_row().join("\t");
                        writeln!(writer, "{}", row_str).map_err(|e| e.to_string())?;
                    }
                }
                if !row.over_threshold(threshold) {
                    continue;
                }
            }
        }
        Ok(())
    }
    
     */