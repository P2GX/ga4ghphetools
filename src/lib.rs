mod age;
mod allele;
mod curie;
mod excel;
mod header;
mod hpo;
mod hpo_term_template;
mod individual_template;
mod onset;
mod simple_label;
mod simple_term;
mod transcript;

use individual_template::{generate_qc_summary, IndividualTemplateFactory};


pub fn qc_check(hp_json_path: &str, pyphetools_template_path: &str) {
    let list_of_rows     = excel::read_excel_to_dataframe(pyphetools_template_path);
    if list_of_rows.is_err() {
        eprintln!("[ERROR] could not read excel file: '{}", list_of_rows.err().unwrap());
        return;
    }
   
    let result =  IndividualTemplateFactory::new(hp_json_path, list_of_rows.unwrap().as_ref());
    match result {
        Ok(template_factory) => {
            let result = template_factory. get_templates();
            match result {
                Ok(template_list) => {
                    println!("[INFO] We parsed {} templates successfully.", template_list.len());
                },
                Err(errs) => {
                    eprintln!("[ERROR] We encountered errors");
                    for e in errs.messages {
                        eprintln!("[ERROR] {}", e);
                    }
                }
            }
    },
    Err(error) => {
        eprintln!("Could not create template factory! {}", error);
    }
}

}