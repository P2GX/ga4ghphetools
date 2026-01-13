use clap::{Arg, ArgMatches};
use ga4ghphetools::dto::etl_dto::EtlDto;

/// Returns the `clap::Command` for ETL
pub fn command() -> clap::Command {
    clap::Command::new("etl")
        .about("Test converting an EtlDto to CohortData")
        .arg(Arg::new("input").short('i').long("input").required(true))
        .arg(Arg::new("hpo").short('o').long("hpo").required(true))
}

/// Handler for the subcommand
pub fn handle(sub_matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let input = sub_matches.get_one::<String>("input").unwrap();
    let hpo_path = sub_matches.get_one::<String>("hpo").unwrap();
    let hpo = crate::load_hpo(hpo_path)?;

    let contents = std::fs::read_to_string(input)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    let dto: EtlDto = serde_json::from_str(&contents)
        .map_err(|e| format!("Failed to deserialize JSON: {}", e))?;

    let cohort = ga4ghphetools::etl::get_cohort_data_from_etl_dto(hpo.clone(), dto)?;
    let json = serde_json::to_string_pretty(&cohort)?;
    println!("{}", json);

    Ok(())
}
