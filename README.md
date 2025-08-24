# PheTools
Rust implementation of GA4GH Phenopacket Schema Tooling


To run the example program, enter
```bash
cargo run --bin rpt -- --rpt /.../.../phenopacket-store/notebooks/CD28/input/CD28_IMD123_individuals.xlsx --json /.../.../hp.json 
```
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


## Debugging
Clean target and build test binary
```bash
cargo clean
cargo test --no-run
ls target/debug/deps/
````
find a binary file (no ``d``) that looks like this
```bash
target/debug/deps/ga4ghphetools-bf312e28ab5ca020 
```

Create a launch json to test the binars
```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "firedbg",
      "request": "launch",
      "name": "Debug test: test_<NAME>",
      "program": "${workspaceFolder}/target/debug/deps/ga4ghphetools-bf312e28ab5ca020",
      "args": ["--exact", "test_<NAME>"],
      "cwd": "${workspaceFolder}",
      "sourceLanguages": ["rust"]
    }
  ]
}
```
replace ``test_<NAME>``with somthing that makes sense.