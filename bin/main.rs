// src/main.rs
use clap::{command, Parser};
use ga4ghphetools::etl::etl_tools::EtlTools;
use ga4ghphetools::PheTools;
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

    if true {
        test_load_template(hpo_arc, &cli.template);
    } else {
        test_load_etl(hpo_arc);
    }
   
}



fn test_load_template(hpo_arc: Arc<FullCsrOntology>, template: &str) {
    let mut phetools = PheTools::new(hpo_arc);
    match phetools.load_excel_template(template, false, |p,q|{
        println!("{}/ {} variants validated", p, q);}) {
        Ok(cohort_dto) => {
           println!("[INFO] No errors identified for {:?}\n\n\n", template);
        }
        Err(e) => {
            println!("[ERROR] {:?}", e);
            return;
        }
    }
}


fn test_load_etl(hpo_arc: Arc<FullCsrOntology>) {
    let template_path = "/Users/robin/data/hpo/etlTest.xlsx";
    let etl_tools = EtlTools::new(hpo_arc, template_path, false).unwrap();
    println!("Created etl_tools");
    println!("{}", etl_tools.raw_table());
}