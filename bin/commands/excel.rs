use clap::{Arg, ArgMatches};


#[cfg(feature = "excel_export")]
use ga4ghphetools::export::output_excel_comparison;

/// Returns the `clap::Command` for this subcommand
pub fn command() -> clap::Command {
    clap::Command::new("excel")
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
}

/// Handler for the subcommand
#[cfg(feature = "excel_export")]
pub fn handle(sub_matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let cohort_1 = sub_matches.get_one::<String>("cohort1").unwrap();
    let cohort_2 = sub_matches.get_one::<String>("cohort2").unwrap();
    let output = sub_matches.get_one::<String>("output").unwrap();
    let hpo_path = sub_matches.get_one::<String>("hpo").unwrap();
    let threshold: usize = sub_matches.get_one::<String>("threshold").unwrap().parse()?;
    let hpo = crate::load_hpo(hpo_path)?;

    output_excel_comparison(cohort_1, cohort_2, output, hpo, threshold).map_err(|e| e.into())
}

#[cfg(not(feature = "excel_export"))]
pub fn handle(_sub_matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("This binary was built without the `excel_export` feature");
    Ok(())
}
