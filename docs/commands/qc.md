# qc

This command is intended to be applied to the entire phenopacket store notebook repository. The goal of the command is to perform some basic quality control measures and report problems.

## Usage

```bash
phetools qc -h
Q/C cohort files

Usage: phetools qc [OPTIONS] --hpo <hpo>

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

## Example

```bash
phetools qc -o ../../data/hpo/hp.json -d ../phenopacket-store/notebooks 
Processed 732 gene directories.
Did not recognize MOI: ModeOfInheritance { hpo_id: "HP:0001427", hpo_label: "Mitochondrial inheritance", citation: "PMID:39468830" }
```
