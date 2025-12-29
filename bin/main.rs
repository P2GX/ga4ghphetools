// src/main.rs
use clap::{Arg, ArgMatches, Command};
use ga4ghphetools::dto::etl_dto::EtlDto;
use ontolius::{io::OntologyLoaderBuilder, ontology::csr::FullCsrOntology};
use std::sync::Arc;


#[cfg(feature = "excel_export")]
use ga4ghphetools::export::output_excel_comparison;


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
        .subcommand(
    Command::new("excel")
                .about("Compare two cohorts and export to Excel")
                .arg(Arg::new("cohort1").long("cohort1").required(true))
                .arg(Arg::new("cohort2").long("cohort2").required(true))
                .arg(Arg::new("output").long("output").required(true))
                .arg(Arg::new("hpo").long("hpo").required(true))
                .arg(
                    Arg::new("threshold")
                        .long("threshold")
                        .default_value("1"),
                )
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
        Some(("compare", sub_matches)) => {
            #[cfg(feature = "excel_export")]
            handle_compare(sub_matches).expect("Excel comparison failed");

            #[cfg(not(feature = "excel_export"))]
            eprintln!("This binary was built without the `excel_export` feature");
        }
        _ => println!("No subcommand was used"),
    }
      
}

#[cfg(feature = "excel_export")]
fn handle_compare(sub_matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let cohort_1 = sub_matches
        .get_one::<String>("cohort1")
        .expect("cohort1 is required");

    let cohort_2 = sub_matches
        .get_one::<String>("cohort2")
        .expect("cohort2 is required");

    let output = sub_matches
        .get_one::<String>("output")
        .expect("output is required");

    let hpo_path = sub_matches
        .get_one::<String>("hpo")
        .expect("hpo is required");

    let threshold: usize = sub_matches
        .get_one::<String>("threshold")
        .unwrap()
        .parse()?;

    let hpo = load_hpo(hpo_path)?;

    output_excel_comparison(
        cohort_1,
        cohort_2,
        output,
        hpo,
        threshold,
    )
    .map_err(|e| e.into())
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
    let contents = std::fs::read_to_string(input)
                    .map_err(|e| format!("Failed to read file: {}", e)).unwrap();
    let dto: EtlDto = serde_json::from_str(&contents)
                    .map_err(|e| format!("Failed to deserialize JSON: {}", e)).unwrap(); 
    
    let cohort = ga4ghphetools::etl::get_cohort_data_from_etl_dto(hpo_arc.clone(), dto)?;
    let json = serde_json::to_string_pretty(&cohort).unwrap();
    println!("{}", json);
    Ok(())
}


fn load_hpo(json_path: &str) -> Result<Arc<FullCsrOntology>, Box<dyn std::error::Error>> {
    let loader = OntologyLoaderBuilder::new().obographs_parser().build();
    let hpo: FullCsrOntology = loader.load_from_path(json_path)?;
    Ok(Arc::new(hpo))
}



fn test_load_template(hpo: Arc<FullCsrOntology>, template: &str) {
    match ga4ghphetools::factory::load_pyphetools_excel_template(template, false, hpo,|p,q|{
        println!("{}/{} variants validated", p, q);}) {
        Ok(cohort_dto) => {
           println!("[INFO] No errors identified for {:?}\n\n\n", cohort_dto);
        }
        Err(e) => {
            println!("[ERROR] {:?}", e);
            return;
        }
    }
}
