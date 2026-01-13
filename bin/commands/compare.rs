use clap::{ArgMatches};

pub fn command() -> clap::Command {
    clap::Command::new("compare")
        .about("Compare two cohorts and export to Excel")
        .arg(clap::Arg::new("cohort1").long("cohort1").required(true))
        .arg(clap::Arg::new("cohort2").long("cohort2").required(true))
        .arg(clap::Arg::new("output").long("output").required(true))
        .arg(clap::Arg::new("hpo").long("hpo").required(true))
        .arg(
            clap::Arg::new("threshold")
                .long("threshold")
                .default_value("1"),
        )
}

#[cfg(feature = "excel_export")]
pub fn handle(sub_matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    crate::commands::excel::handle(sub_matches)
}

#[cfg(not(feature = "excel_export"))]
pub fn handle(_sub_matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("This binary was built without the `excel_export` feature");
    Ok(())
}
