// src/main.rs
use clap::{Arg, ArgMatches, Command};
use ga4ghphetools::etl::etl_tools::EtlTools;
use ga4ghphetools::PheTools;
use ontolius::{io::OntologyLoaderBuilder, ontology::csr::FullCsrOntology};
use std::sync::Arc;





fn main() {
    let matches = Command::new("phetools")
        .about("GA4GH Phenopacket Schema Curation Library Demo")
        .version(env!("CARGO_PKG_VERSION"))  
        .subcommand(
            Command::new("excel")
                .about("Test loading of legacy Excel template")
                .arg(Arg::new("template").short('t').long("template").required(true))
                .arg(Arg::new("hpo").short('o').long("hpo").required(true))
        )
        .subcommand(
            Command::new("json")
                .about("Test loading of new JSON template")
                .arg(Arg::new("json").short('i').long("input"))
                .arg(Arg::new("hpo").short('o').long("hpo").required(true))
        )
         .subcommand(
            Command::new("etl")
                .about("Test converting an EtlDto to CohortData")
                .arg(Arg::new("input").short('i').long("input").required(true))
                .arg(Arg::new("hpo").short('o').long("hpo").required(true))
        )
        .subcommand(
            Command::new("version")
                .about("Show library version")
                .arg(Arg::new("version").short('v').long("version"))
        )
        .get_matches();
    match matches.subcommand() {
        Some(("excel", sub_matches)) => handle_excel(sub_matches).expect("Could not start excel command"),
        Some(("json", sub_matches)) => {
            let input = sub_matches.get_one::<String>("input").unwrap();
            println!("json: {}", input);
        },
        Some(("etl", sub_matches)) => handle_etl(sub_matches).expect("Could not start ETL command"),
        Some(("version", sub_matches)) => {
             println!("Version: {}", env!("CARGO_PKG_VERSION"));
        },
        _ => println!("No subcommand was used"),
    }
      
}

fn handle_excel(sub_matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let template = sub_matches
        .get_one::<String>("template")
        .expect("template argument is required");
    let hpo = sub_matches
        .get_one::<String>("hpo")
        .ok_or("Missing required --hpo argument")?;

    let hpo_arc = load_hpo(hpo)?;
    test_load_template(hpo_arc, template);
    Ok(())
}

fn handle_etl(sub_matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let input = sub_matches.get_one::<String>("input").unwrap();
    println!("ETL: {}", input);
    let hpo = sub_matches
        .get_one::<String>("hpo")
        .ok_or("Missing required --hpo argument")?;
    let hpo_arc = load_hpo(hpo)?;
   
    let etl_tools = EtlTools::from_json(&input, hpo_arc).unwrap();
    let cohort = etl_tools.get_cohort_data().unwrap();
    let json = serde_json::to_string_pretty(&cohort).unwrap();
    println!("{}", json);
    Ok(())
}


fn load_hpo(json_path: &str) -> Result<Arc<FullCsrOntology>, Box<dyn std::error::Error>> {
    let loader = OntologyLoaderBuilder::new().obographs_parser().build();
    let hpo: FullCsrOntology = loader.load_from_path(json_path)?;
    Ok(Arc::new(hpo))
}



fn test_load_template(hpo_arc: Arc<FullCsrOntology>, template: &str) {
    let mut phetools = PheTools::new(hpo_arc);
    match phetools.load_excel_template(template, false, |p,q|{
        println!("{}/{} variants validated", p, q);}) {
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
    //let etl_tools = EtlTools::(hpo_arc, template_path, false).unwrap();
    println!("Created etl_tools");
    //println!("{}", etl_tools.raw_table());
}