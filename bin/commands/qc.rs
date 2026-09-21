use std::path::PathBuf;
use clap::ArgMatches;

pub fn command() -> clap::Command {
    clap::Command::new("qc")
        .about("Q/C cohort files")
        .arg(clap::Arg::new("dir").short('d').long("directory"))
        .arg(clap::Arg::new("hpo").short('o').long("hpo").required(true))
}

pub fn handle(sub_matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let ppkt_store_directory = sub_matches.get_one::<String>("dir").expect("Could not read phenopacket store directory");
    let hpo_path = sub_matches.get_one::<String>("hpo").expect("Could not retrieve hp.json path");
    let hpo = crate::load_hpo(hpo_path).expect("Could not construct HPO ontology");
    let path: PathBuf = PathBuf::from(ppkt_store_directory);
    let repo_qc = ga4ghphetools::get_repo_qc(&path, hpo)?;    
    println!("Q/C report for {}", repo_qc.repo_path.to_string_lossy());
    println!("Cohorts: n={}; total phenopackets: todo", repo_qc.cohort_count);
    let mut i=0;
    for qc_report in repo_qc.errors {
        i += 1;
        for issue in qc_report.issues.iter() {
            println!("[{}-{}-{:?}] {}", i, qc_report.cohort_name, issue.domain, issue.message);
        }
        
    }
    Ok(())
}

