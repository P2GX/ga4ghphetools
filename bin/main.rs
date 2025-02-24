// src/main.rs
use clap::Parser;
use std::path::Path;


/// A simple CLI example
#[derive(Parser)]
#[command(name = "rustphen")]
#[command(about = "Process pyphetools Excel template", long_about = None)]
struct Cli {
    /// A required input file
    #[arg(short, long)]
    pyphetools: String,

    #[arg(short, long)]
    json: String,

    /// An optional flag
    #[arg(short, long)]
    verbose: bool,
}

use rustphen::qc_check;




fn main() {
    let cli = Cli::parse();
    if ! Path::new(&cli.pyphetools).exists() {
    } else {
        println!("Could not find pyphetools template at {}.", &cli.pyphetools);
    }
    if ! Path::new(&cli.json).exists() {
    } else {
        println!("Could not find HPO JSON file at {}.", &cli.json);
    }


    let pyphetools_template_path = "/Users/robin/GIT/phenopacket-store/notebooks/CD28/input/CD28_IMD123_individuals.xlsx";
    let hpo_json_path = "/Users/robin/GIT/human-phenotype-ontology/hp.json";
    qc_check(&cli.json, &cli.pyphetools);
   
     
}
