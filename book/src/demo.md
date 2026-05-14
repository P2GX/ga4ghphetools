# Command-Line Application

Some features of ga4ghphetools are available as a command-line application called **phetools**.


## Building the app
To compile (build) the command-line application, enter the following command
```bash
cargo build --release --features cli
```
The `phetools` binary is created here: ``./target/release/phetools``:

```bash
./target/release/phetools 
GA4GH Phenopacket Schema Curation Library Demo

Usage: phetools [COMMAND]

Commands:
  excel        Compare two cohorts and export to Excel
  extract      Extract phenopackets from Cohort files
  etl          Test converting an EtlDto to CohortData
  compare      Compare two cohorts and export to Excel
  json         Q/C Cohort JSON file
  remove-term  Remove HPO Term and its annotations from Cohort Data file
  help         Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

Alternative, the application can be run without build using the following command
```bash
cargo run --features cli --bin phetools
```

In the rest of the documentation, we will assume that the application is on the path and just show `phetools`.

## Extracting phenopackets from Cohort files

The ``phenoboard`` app stores data in form of Cohort files (json). We can extract all of the phenopackets from one such file or a directory containing multiple such files using the following command

```bash
phetools extract -i ../mgd-ppkt/cohorts -o <existing output directory> --hpo ../../data/hpo/hp.json
```

## Compare
Compare two cohorts and export to Excel

## ETL
Test converting an EtlDto to CohortData (only useful for debugging/development!).

## json
Q/C Cohort JSON file (only useful for debugging/development!).

## remove-term
Remove HPO Term and its annotations from Cohort Data file. This can be useful if an HPO term has been  added to a Cohort in error.

## To see private features in documentation
```bash
cargo doc --document-private-items --open
```