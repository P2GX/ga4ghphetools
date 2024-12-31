// src/main.rs

use individual_template::IndividualTemplateFactory;

mod age;
mod allele;
mod builder;
mod curie;
mod excel;
mod header;
mod hpo;
mod hpo_term_template;
mod individual_template;
mod onset;
mod ppkt;
mod simple_label;
mod simple_term;
mod transcript;
mod generated {
    pub mod org {
        pub mod ga4gh {
            pub mod vrs {
                pub mod v1;
            }
            pub mod vrsatile {
                pub mod v1;
            }
        }
        pub mod phenopackets {
            pub mod schema {
                pub mod v2 {
                    pub mod core {
                        pub mod core;
                    }
                    pub mod phenopackets;
                }
            }
        }
    }
}

fn main() {
    let file_path = "/Users/robin/GIT/phenopacket-store/notebooks/CD28/input/CD28_IMD123_individuals.xlsx";
    let hpo_json_path = "/Users/robin/GIT/human-phenotype-ontology/hp.json";
    
    let list_of_rows     = excel::read_excel_to_dataframe(file_path);
    if list_of_rows.is_err() {
        eprintln!("[ERROR] could not read excel file: '{}", list_of_rows.err().unwrap());
        return;
    }
    let list_of_termplates = IndividualTemplateFactory::new(hpo_json_path, list_of_rows.unwrap().as_ref());
     
}
