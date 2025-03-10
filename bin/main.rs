// src/main.rs
use clap::Parser;
use std::path::Path;


/// A simple CLI example
#[derive(Parser)]
#[command(name = "rpt")]
#[command(about = "Process pyphetools Excel template", long_about = None)]
struct Cli {
    /// A required input file
    #[arg(short, long)]
    pyphetools: String,

    #[arg(short, long)]
    json: String,

    /// An optional flag
    #[arg(short, long)]
    verbose: bool,
}

use rustphen::qc_check;




fn main() {
    let cli = Cli::parse();
    if ! Path::new(&cli.pyphetools).exists() {
        println!("Could not find pyphetools template at {}.", &cli.pyphetools);
        return;
    }
    if ! Path::new(&cli.json).exists() {
        println!("Could not find HPO JSON file at {}.", &cli.json);
        return;
    }

    qc_check(&cli.json, &cli.pyphetools);
   
     
}
