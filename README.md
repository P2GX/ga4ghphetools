# rustphen
Rust implementation of GA4GH Phenopacket Schema Tooling


To run the example program, enter
```bash
cargo run --bin rustphen -- --pyphetools /.../.../phenopacket-store/notebooks/CD28/input/CD28_IMD123_individuals.xlsx --json /.../.../hp.json 
```
For faster performance, enter
```bash
cargo run --release --bin rustphen -- --pyphetools ./../phenopacket-store/notebooks/CD28/input/CD28_IMD123_individuals.xlsx --json ./../../data/hpo/hp.json 
```

## Generating rust code from phenopackets (experiment)

1. Download google's protoc compiler
2. Copy code from phenopacket schema (only v2) to a local directory
3. Copy versionof VRS and VRSATILE
4. Run cargo build with the following code in build.rs

```rust

fn main() {
    let protofiles: &[&str; 14] = &[
        "phenopackets/schema/v2/phenopackets.proto",
        "phenopackets/schema/v2/core/meta_data.proto",
        "phenopackets/schema/v2/core/individual.proto",
        "phenopackets/schema/v2/core/base.proto",
        "phenopackets/schema/v2/core/biosample.proto",
        "phenopackets/schema/v2/core/phenotypic_feature.proto",
        "phenopackets/schema/v2/core/medical_action.proto",
        "phenopackets/schema/v2/core/genome.proto",
        "phenopackets/schema/v2/core/interpretation.proto",
        "phenopackets/schema/v2/core/pedigree.proto",
        "phenopackets/schema/v2/core/measurement.proto",
        "phenopackets/schema/v2/core/disease.proto",
        "phenopackets/vrs/v1/vrs.proto",
        "phenopackets/vrsatile/v1/vrsatile.proto",
    ];


    prost_build::compile_protos(protofiles, &[".",
    "/Users/robin/Downloads/protoc-29.2-osx-aarch_64/protobuf/src"])
        .expect("Failed to compile Protobuf files");
}
```

Cargo.toml must look like this:
```
[package]
name = "cpb"
version = "0.1.0"
edition = "2021"

[dependencies]
prost = "0.13.4"  # Protobuf serialization/deserialization library for Rust
prost-types = "0.13.4"  # Well-known Protobuf types

[build-dependencies]
prost-build = "0.13.4"  # Build tool f
```