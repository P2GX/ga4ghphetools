# Output

GA4GH Phenools can be used to output data about cohorts as Excel or text files.


## Excel

To compare two cohorts starting with their cohort files, we can enter

```bash
cargo run --features excel_export -- compare \
  --cohort1 cohort_a.json \
  --cohort2 cohort_b.json \
  --output comparison.xlsx \
  --hpo hp.json \
  --threshold 5
```

Note that we need to compile with the excel_export feature activated.