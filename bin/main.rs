// src/main.rs
use clap::Parser;
use ontolius::{io::OntologyLoaderBuilder, ontology::csr::FullCsrOntology};
use std::path::Path;
use std::sync::Arc;

/// A simple CLI example
#[derive(Parser)]
#[command(name = "rpt")]
#[command(about = "Process pyphetools Excel template", long_about = None)]
struct Cli {
    /// A required input file
    #[arg(short, long)]
    template: String,

    #[arg(short, long)]
    json: String,

    /// An optional flag
    #[arg(short, long)]
    verbose: bool,
}

use phetools::PheTools;

fn main() {
    let cli = Cli::parse();
    if !Path::new(&cli.template).exists() {
        println!("Could not find phetools template at {}.", &cli.template);
        return;
    }
    if !Path::new(&cli.json).exists() {
        println!("Could not find HPO JSON file at {}.", &cli.json);
        return;
    }
    // Configure the loader to parse the input as an Obographs file
    let loader = OntologyLoaderBuilder::new().obographs_parser().build();
    let hpo: FullCsrOntology = loader
        .load_from_path(&cli.json)
        .expect("HPO should be loaded");
    let hpo_arc = Arc::new(hpo);
    let mut phetools = PheTools::new(hpo_arc);
    match phetools.load_excel_template(&cli.template) {
        Ok(template) => {
            println!("[INFO] No errors identified for {}", &cli.template);
            println!("{}", &phetools);
            println!("{:?}", phetools.get_string_matrix());
        }
        Err(e) => {
            println!("[ERROR] {}", e);
        }
    }
    let hmap = phetools.get_hpo_data();
    println!("{:?}", hmap);
}
