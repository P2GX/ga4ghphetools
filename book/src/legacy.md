# Legacy template

The [pypheools](https://github.com/monarch-initiative/pyphetools) project was our first attempt to curate GA4GH Phenopackets
at scale (2021-2024). For this, we designed an Excel template for curation. GA4GH phenotools contains code that enables us
to import daa from these excel files and then serialize the contents to the new JSON format. We plan to update the entire
phenopacket store to use the new JSON serialization format and delete the legacy Excel files. At this point, we will remove
the legacy-related code from the current library as well. We present information about the structure of the Excel files here in order
to help with this process.


# A format for cohort descriptions in excel

The schema of the template consists in two rows that specify the nature of the data. There is a fixed set of columns that capture basic demographic data together with the disease, the source publication, and the variants. The second half of the template should be used to record information about
HPO terms curated from the publications.

Note that the format specifier "CURIE" means "Compact uniform resource identifier", which means the entry should have a prefix (e.g., PMID, a colon, and the identifier, e.g., 3021034, altogether "PMID:3021034"). "str" refers to an arbitrary text (string). "optional" means the cell can
be left empty.

## Fixed columns
The first (leftmost) 11 or 12 columns specify basic demographic data together with the disease, the source publication, and the variants.
The first two rows are used to specify the datatypes and should not be changed. The following tables show the first two rows together with
one example row with data extracted from a publication (We show two tables for better legibility)


|  PMID	| title	| individual_id	| Comment | disease_id	 | disease_label|
|:------|:------|:--------------|:--------|:-------------|:-------------|
| CURIE	| str	| str       	| optional|  CURIE       | 	        str |
| PMID:33087723| 	Early-onset autoimmunity associated with SOCS1 haploinsufficiency| 	A1|	|	OMIM:603597|	Autoinflammatory syndrome, familial, with or without immunodeficiency|


1. **PMID** (CURIE: The PubMed identifier of the publication being curated.
2. **title** (str): The title of the publication being curated.
3. **individual_id** (str): The identifier of the individual being described in the original publication. This field is required. Please add ‘individual’ if the original article does not provide an identifier (if needed, individual 1, individual 2,...).
4. **comment** (str): This field is provided to record additional information that will not be used for creating phenopackets but may be helpful for future reference. It can be left empty.
5. **disease_id** (CURIE). The disease identifer (e.g., ``OMIM:154700`` or  ``MONDO:0007947``).
6. **disease_label** (str). The name of the disease (e.g. ``Marfan syndrome``).




|  HGNC_id   |  gene_symbol|  transcript| 	allele_1| allele_2 | variant.comment|
|:-----------|:------------|:-----------|:----------|:---------|:---------------|
|  CURIE     |  str        |      str   | str       |str       |  str        	|
| HGNC:19383 | SOCS1       | NM_003745.2|c.368C>G   |	    na |p.P123R         |



7. **HGNC_id** (CURIE): Identifier of the [HUGO Gene Nomenclature Committee](https://www.genenames.org/){target="_blank"}.
8. **gene_symbol** (str):: Gene symbol, e.g., SOCS1.
9. **transcript** (str): The identifier of the transcript. NCBI RefSeq or ENSEMBL identifiers are preferred.
10. **allele_1** (str): A string representing the first pathogenic allele (variant) according to [HGVS](https://hgvs-nomenclature.org/stable/background/simple/) nomenclature.
11. **allele_2** (str): This field should not be used for monoallelic diseases (e.g. autosomal dominant, XLR). The column can eigther be omitted or can be filled with "na" to denote "not applicable". It the column is present and is left empty, this will be flagged as an error. For biallelic diseases (autosomal recessive), specific the second allele (which will be the same as the first for homozygous genotypes).
12. **variant.comment** (str): This field is provided to record additional information that will not be used for creating phenopackets but may be helpful for future reference.



| age_of_onset   | age_at_last_encounter |sex	  | HPO	                 |
:----------------|:----------------------|:-------|:-------------------------|
| age      	     | age                   | M:F:O:U| na	                     |
|Infantile onset |    	P21Y	         | F      | na     |

11. **age_of_onset**: The age of onset of disease, recorded using [iso8601 convention](https://en.wikipedia.org/wiki/ISO_8601#Durations) or an HPO [Onset](https://hpo.jax.org/app/browse/term/HP:0003674){:target="_blank"} term.
12. **sex** (M:F:U:O): one of M (male), female (F), other(O), or unknown (U)
13. **HPO** (str): The column marks the end of the data columns and should contain "na".

### HPO Term Columns
All of the following columns denote HPO terms. The first row has the HPO term label. Be sure to use the same label as is shown on the HPO
webpage and do not chance the capitalization. The second row has the corresponding HPO id. The following table shows several examples, whereby
the individual_id column from above is shown for ease of exposition.




|individual_id |  Hepatitis  | 	Pancreatitis| 	Lymphadenopathy| 	Splenomegaly   |
|:-------------|:------------|:-------------|:-----------------|:------------------|
|              | HP:0012115  |HP:0001733    |  HP:0002716      |   HP:0001744      |
| A            | observed    | excluded     |                  | P4Y2M             |
| B            |P3Y          |    na        | observed         | excluded          |

Each table cell can contain either
1. observed: The phenotypic abnormality denoted by the HPO term was present
2. excluded: The phenotypic abnormality denoted by the HPO term was investigated and ruled out.
3. An [iso8601](https://en.wikipedia.org/wiki/ISO_8601#Durations) string denoting the age of onset.
4. na or empty (blank): Information not available or phenotypic feature not measured.

In this example, individual A was observed to have hepatitis (but age of onset is unknown or not available), pancreatitis was ruled out, no information is available about lymphadenopathy, and splenomegaly was first observed at age 4 years and 2 months.

Individual B was found to have hepatitis first observed at age 3 years, no information was available about pancreatitis, lymphadenopathy was observed (but age of onset is unknown or not available), and splenomegaly  was ruled out.











The file should contain at least the following information; see explanations below.




|row_type| id | age | sex | allele |  Tall stature | Abnormal sternum morphology | Potassium |
|:---- |:----|:----|:----|:-----|:---------|:----|:------------- |
| header1 | str | ISO8601 | str | NM_000138.5 | simple  | option| threshold |
| header2 |  |  |  |  | HP:0000098 |  HP:0000767; HP:0000768| 3.5-5.2 mEq/L: High->Hyperkalemia{HP:0002153); Low->Hypokalemia(HP:0002900) |
| individual| patient A | P6Y | male | c.8326C>T | + | Pectus carinatum | n/a |
| individual| patient B | P9Y | female | c.7988G>C | - | Pectus excavatum | 5.8 |



## row_type
Each row must begin with one of the words "header1", "header2" or "individual". There should be one row for each individual in the cohort.

## id
This is an cohort-specific identifier that **must** be anonymized.

## age
This is the age of the individual at the time of the medical encounter at which the phenotypic features were recorded. The format of the column is recorded in the header1 line. Valid options are *ISO8601* for strings such as "P4Y" (four years of age) and "P71Y6M2D" for 71 years, 6 months, and 2 days; *Years* for 5 (5 years of age) or 7.5 (7 years and 6 months).

## sex
Use male, female, other, or unknown.

# HPO columns
The remaining columns contain information about HPO terms observed in the individuals. There are three types of column.

The top row contains the label of the term. The header1 row contains the word "simple". The header2 row contains the HPO id; in the example table, we see [Tall stature; HP:0000098](https://hpo.jax.org/app/browse/term/HP:0000098). If the feature is observed in an indivual, use "+"; if the feature was explicitly excluded, use "-". If the feature was not measured or no information is available, use "n/a".
