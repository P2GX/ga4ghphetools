# Demo

# Example program
To run the example program (whose code is available at bin/main.rs), enter
```bash
cargo run --bin rpt -- --rpt /.../.../phenopacket-store/notebooks/CD28/input/CD28_IMD123_individuals.xlsx --json /.../.../hp.json 
```
Adjust the paths to an Excel legacy template and the hp.json file as needed.

For faster performance, enter
```bash
cargo run --features="cli"  --release --bin rpt -- --template ./../phenopacket-store/notebooks/CD28/input/CD28_IMD123_individuals.xlsx --json ./../../data/hpo/hp.json 
```



## To build the binary demo (with clap)
```bash
cargo build --release --features cli
```
(the binary is then in ``./target/release/rpt``)
to run it
```bash
cargo run --features cli --bin rpt
```

## To see private features in documentation
```bash
cargo doc --document-private-items --open
```