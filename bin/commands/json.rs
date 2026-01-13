use crate::commands::util::extract_file_name;

use clap::ArgMatches;

pub fn command() -> clap::Command {
    clap::Command::new("json")
        .about("Q/C Cohort JSON file")
        .arg(clap::Arg::new("cohort").short('c').long("cohort"))
        .arg(clap::Arg::new("hpo").short('o').long("hpo").required(true))
}

pub fn handle(sub_matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let json_input_path = sub_matches.get_one::<String>("input").expect("Could not read JSON input");
    let hpo_path = sub_matches.get_one::<String>("hpo").expect("Could not retrieve hp.json path");
    let hpo = crate::load_hpo(hpo_path).expect("Could not construct HPO ontology");
    let cohort = ga4ghphetools::factory::load_json_cohort(json_input_path).expect("Could not load Cohort JSON file");
    let cohort_file_name = extract_file_name(json_input_path);
    match ga4ghphetools::factory::qc_assessment(hpo, &cohort) {
        Ok(_) => println!("No Q/C issues identified for {cohort_file_name}."),
        Err(e) => eprint!("Error for {cohort_file_name}: {e}"),
    }
    
    Ok(())
}

