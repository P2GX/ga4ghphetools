use std::fs::File;

use clap::{Arg, ArgMatches};
use crate::commands::util::extract_file_name;

pub fn command() -> clap::Command {
    clap::Command::new("remove-term")
        .about("Remove HPO Term and its annotations from Cohort Data file")
        .arg(Arg::new("cohort").short('c').long("cohort").required(true))
        .arg(Arg::new("hpo-id").short('i').long("id").required(true))
}


pub fn handle(sub_matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let input_json = sub_matches.get_one::<String>("cohort").unwrap();
    let hpo_id = sub_matches.get_one::<String>("hpo-id").unwrap();
    println!("Remove HPO Term {hpo_id} from Cohort {input_json}");
    let cohort = ga4ghphetools::factory::load_json_cohort(input_json).expect("Could not load Cohort JSON file");
    let modified_cohort = cohort.remove_hpo_column(hpo_id)?;
    let fname = extract_file_name(input_json);
    let outname = format!("modified-{fname}");
    let json = serde_json::to_string_pretty(&modified_cohort)?;
    println!("{}", json);
    let file = File::create(outname)?;
    let writer = std::io::BufWriter::new(file);

    serde_json::to_writer_pretty(writer, &modified_cohort)?;
    

    Ok(())
}