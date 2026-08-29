# Extracting phenopackets from Cohort files

The ``phenoboard`` app stores data in form of Cohort files (json). We can extract all of the phenopackets from one such file or a directory containing multiple such files using the following command

```bash
phetools extract -i ../mgd-ppkt/cohorts -o <directory> --hpo hp.json
```
Where `<directory>` is an existing directory and hp.json should have the complete path to a downloaded `hp.json` file.