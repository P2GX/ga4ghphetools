# Update

This command is intended to be applied to the entire phenopacket store notebook repository. The goal of the command is to update 
the [Human Phenotype Ontology](https://hpo.jax.org/) term identifiers and labels to the latest versions.

During editing of the HPO, occasionally, two terms are merged because they are determined to be synonymous (that is, one of the two terms was added redundantly because the other term was not recognized as a synonym during the curation process). In this case, one of the terms is chosen to remain, and the identifier of the other term is added as an `alternative_id` to the other term. If the merged term was used for annotation, then we need to change the identifier and label to those of the other term. This command does so across all cohorts of phenopacket store.

The code first reads each CohortData file and checks the HPO headers. If all HPO headers are up to date, no further action is taken. If one or more headers contain an outdated term identifier or label, then these are changed and the file is overwritten with the new data. Additionally, the code outputs the correspondingly updated phenopackets when the cohort file was updated.

If the code hits an error, it stops at the first error and returns. This usually means there is an error that needs manual (human) intervention, and this should be fixed with phenoboard or as required and then the command can be rerun. If the command does not encounter errors, it returns an `UpdateReport` object with a summary of how many cohorts were updated.

## Usage

```bash
./target/release/phetools update -h
Update HPO term ids/labels

Usage: phetools update [OPTIONS] --hpo <hpo>

Options:
  -d, --directory <dir>
  -o, --hpo <hpo>
  -h, --help             Print help
```

The `-d` argument specifies the path to the phenopacket store data which contains the individual gene folders, e.g.,
```bash
ls ../phenopacket-store/notebooks
CRELD1                HMGCS2                NRAP
(...)
```

The `-o` argument is the path to the `hp.json` file. Be sure to use the latest version.

## API

Client code can run the equivalent functionality by calling the following function (in `ga4ghphetools::repo`).

```rust
pub fn update_all_ppkt(
    path: &PathBuf,
    hpo: Arc<FullCsrOntology>
) -> Result<UpdateReport, String> {
    let repo = GptRepository::new(path);
    repo.update_all_ppkt(hpo).map_err(|e|e.to_string())
}
```

