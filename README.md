# PheTools
Rust implementation of GA4GH Phenopacket Schema Tooling


To run the example program, enter
```bash
cargo run --bin rpt -- --rpt /.../.../phenopacket-store/notebooks/CD28/input/CD28_IMD123_individuals.xlsx --json /.../.../hp.json 
```
For faster performance, enter
```bash
cargo run --release --bin rpt -- --pyphetools ./../phenopacket-store/notebooks/CD28/input/CD28_IMD123_individuals.xlsx --json ./../../data/hpo/hp.json 
```

## Generating rust code from phenopackets (experiment)

1. Download google's protoc compiler
2. Copy code from phenopacket schema (only v2) to a local directory
3. Copy versionof VRS and VRSATILE
4. Run cargo build with the following code in build.rs

## To build the binary demo (with clap)
```bash
cargo build --release --features cli
```
(the binary is then in ``./target/release/rpt``)
to run it
```bash
cargo run --features cli --bin rpt
```
