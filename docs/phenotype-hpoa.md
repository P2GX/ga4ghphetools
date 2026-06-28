# phenotype.hpoa output

The [phenotype.hpoa](https://obophenotype.github.io/human-phenotype-ontology/annotations/phenotype_hpoa/) file is 
is the core download of disease annotation data for the [HPO](https://hpo.jax.org/) project. Internally, the HPO project
uses one so-called "small file" for each disease; information from these files is processed to make the phenotype.hpoa project, which is offered for download. The small files are not offered for download at this time.

GA4GH phenotools has functionality to export a cohort of phenopackets to small file format. For convenience, we explain this format
here and explain the assumptions to code makes to generate the files.

## Format

<style>
table {
  font-size: 12px;
}
</style>

| #diseaseID   | diseaseName        | phenotypeID | phenotypeName                       | onsetID | onsetName | frequency | sex | negation | modifier | description | publication | evidence | biocuration |
|-------------|-------------------|-------------|-------------------------------------|---------|-----------|-----------|-----|----------|----------|-------------|-------------|----------|-------------|
| OMIM:605275 | Noonan syndrome 2 | HP:0011636  | Abnormal coronary artery origin     |         |           | 1/6       |     |          |          |             | PMID:30368668 | PCS      | HPO:probinson\[2021-05-21] |
| OMIM:605275 | Noonan syndrome 2 | HP:0001928  | Abnormality of coagulation          |         |           | 0/4       |     |          |          |             | PMID:30368668 | PCS      | ORCID:0000-0002-0736-9199\[2024-10-02] |
| OMIM:605275 | Noonan syndrome 2 | HP:0000766  | Abnormal sternum morphology         |         |           |           |     |          |          |             | PMID:29469822 | PCS      | HPO:skoehler\[2019-04-18]; HPO:probinson\[2021-05-21] |
| OMIM:605275 | Noonan syndrome 2 | HP:0006721  | Acute lymphoblastic leukemia        |         |           | 3/20      |     |          |          |             | PMID:29469822 | PCS      | ORCID:0000-0002-0736-9199\[2024-04-01] |
| OMIM:605275 | Noonan syndrome 2 | HP:0030674  | Antenatal onset                     |         |           | 11/18     |     |          |          |             | PMID:29469822 | PCS      | ORCID:0000-0002-0736-9199\[2024-04-01] |
| OMIM:605275 | Noonan syndrome 2 | HP:0001134  | Anterior polar cataract             |         |           | 1/20      |     |          |          |             | PMID:29469822 | PCS      | ORCID:0000-0002-0736-9199\[2024-04-01] |
| OMIM:605275 | Noonan syndrome 2 | HP:0000463  | Anteverted nares                    |         |           | 3/20      |     |          |          |             | PMID:29469822 | PCS      | ORCID:0000-0002-0736-9199\[2024-04-01] |
| OMIM:605275 | Noonan syndrome 2 | HP:0002804  | Arthrogryposis multiplex congenita  |         |           | 2/20      |     |          |          |             | PMID:29469822 | PCS      | HPO:skoehler\[2019-04-18]; HPO:probinson\[2021-05-21] |
| OMIM:605275 | Noonan syndrome 2 | HP:0001631  | Atrial septal defect                |         |           | 2/3       |     |          |          |             | PMID:30368668 | PCS      | HPO:probinson\[2021-05-21] |



Note that this file has one line that specifies the frequency of each HPO term in the cohort of individuals described in the PMID. It is
thus possible to have multiple likes for the same HPO with data from different PMIDs.

