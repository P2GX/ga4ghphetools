use std::{fs, io::{self, Write}, path::{Path, PathBuf}, sync::Arc};

use clap::{Arg, ArgMatches, value_parser};
use ontolius::ontology::csr::FullCsrOntology;


pub fn command() -> clap::Command {
    clap::Command::new("extract")
        .about("Extract phenopackets from Cohort files")
        .arg(Arg::new("input").short('i').long("input").required(true).value_parser(value_parser!(PathBuf)))
        .arg(Arg::new("output").short('o').long("output").required(true).value_parser(value_parser!(PathBuf)))
        .arg(clap::Arg::new("hpo").long("hpo").required(true).value_parser(value_parser!(PathBuf)))
}


pub fn handle(sub_matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let input = sub_matches.get_one::<PathBuf>("input").unwrap();
    let output = sub_matches.get_one::<PathBuf>("output").unwrap();
  
    if ! output.is_dir() {
        eprintln!("[ERROR] '{}' is not a directory. Pass an existing directory with the -o/--output argument.", output.to_string_lossy());
        return Ok(());
    }
    let hpo_path = sub_matches.get_one::<PathBuf>("hpo").unwrap();
    let hpo = crate::load_hpo(&hpo_path.to_string_lossy())?;
    let path = Path::new(input);
    if path.is_dir() {
        process_cohort_dir(path, output, hpo.clone())?;
    } else if path.is_file() {
        process_cohort_file(input, output, hpo.clone()).unwrap();
    } else {
        eprintln!("[ERROR] '{}' does not exist. Pass the path to a file or directory with Cohort files with the -i/--input argument.", input.to_string_lossy());
        return Ok(());
    }
    Ok(())
}



fn process_cohort_file(input_file: &PathBuf, output_dir: &Path, hpo: Arc<FullCsrOntology>) -> Result<usize, String> {
    let cohort = ga4ghphetools::factory::load_json_cohort(&input_file.to_string_lossy()).expect("Could not load Cohort JSON file");
    let orcid = cohort.get_latest_biocurator_id()?;
    let overwrite = true;
    let n_processed = ga4ghphetools::ppkt::write_phenopackets(cohort, output_dir.to_path_buf(), orcid, hpo.clone(), overwrite).map_err(|e|e.to_string())?;
    Ok(n_processed)
}

fn process_cohort_dir(input_dir: &Path, output_dir: &Path, hpo: Arc<FullCsrOntology>) -> Result<(), String> {
    let entries = fs::read_dir(input_dir).map_err(|e|e.to_string())?;
    let mut total_processed = 0 as usize;
    print!("\rProcessed {} phenopackets...", total_processed);
            io::stdout().flush().map_err(|e| e.to_string())?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            // If one file fails, we stop and return the error
            let n = process_cohort_file(&path, output_dir, hpo.clone())?;
            total_processed += n;
            print!("\rProcessed {} phenopackets.", total_processed);
            io::stdout().flush().map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}