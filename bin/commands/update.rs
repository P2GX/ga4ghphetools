use std::path::PathBuf;
use clap::ArgMatches;

pub fn command() -> clap::Command {
    clap::Command::new("update")
        .about("Update HPO term ids/labels")
        .arg(clap::Arg::new("dir").short('d').long("directory").required(true))
        .arg(clap::Arg::new("hpo").short('o').long("hpo").required(true))
}

pub fn handle(sub_matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let ppkt_store_directory = sub_matches.get_one::<String>("dir").expect("Could not read phenopacket store directory");
    let hpo_path = sub_matches.get_one::<String>("hpo").expect("Could not retrieve hp.json path");
    let hpo = crate::load_hpo(hpo_path).expect("Could not construct HPO ontology");
    let path: PathBuf = PathBuf::from(ppkt_store_directory);
    let update_report = ga4ghphetools::update_all_ppkt(&path, hpo)?;
    println!("Updated phenopacket store at {}", update_report.directory);
    println!("Processed {} cohorts, of which {} were updated", update_report.processed, update_report.updated);
    Ok(())
}

