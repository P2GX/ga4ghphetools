mod commands;

use clap::Command;
use ontolius::{io::OntologyLoaderBuilder, ontology::csr::FullCsrOntology};
use std::sync::Arc;
use log4rs::init_config;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    log4rs::init_file("log4rs.toml", Default::default()).expect("Failed to initialize log4rs");

     let mut cmd = Command::new("phetools")
        .about("GA4GH Phenopacket Schema: phetools")
        .version(env!("CARGO_PKG_VERSION"))
        .subcommand(commands::extract::command())
        .subcommand(commands::etl::command())
        .subcommand(commands::compare::command())
        .subcommand(commands::json::command())
        .subcommand(commands::qc::command())
        .subcommand(commands::removeterm::command())
        .subcommand(commands::update::command());

    let matches = cmd.clone().get_matches();
    
    match matches.subcommand() {
        Some(("compare", sub_matches)) => commands::compare::handle(sub_matches)?,
        Some(("extract", sub_matches)) => commands::extract::handle(sub_matches)?,
        Some(("etl", sub_matches)) => commands::etl::handle(sub_matches)?,
        Some(("json", sub_matches)) => commands::json::handle(sub_matches)?,
        Some(("qc", sub_matches)) => commands::qc::handle(sub_matches)?,
        Some(("remove-term", sub_matches)) => commands::removeterm::handle(sub_matches)?,
        Some(("update", sub_matches)) => commands::update::handle(sub_matches)?,
        _ => cmd.print_help()?,
    }
    Ok(())
}

/// Load HPO JSON
pub fn load_hpo(json_path: &str) -> Result<Arc<FullCsrOntology>, Box<dyn std::error::Error>> {
    let loader = OntologyLoaderBuilder::new().obographs_parser().build();
    let hpo: FullCsrOntology = loader.load_from_path(json_path)?;
    Ok(Arc::new(hpo))
}
