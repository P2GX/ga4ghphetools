/// Fixtures needed for multiple tests
/// We create a singleton HPO to prevent each test module from loading it anew

#[cfg(test)]
pub mod fixtures {
    use std::{collections::HashMap, sync::LazyLock};
    use ontolius::io::OntologyLoaderBuilder;
    use ontolius::ontology::csr::FullCsrOntology;
    use rstest::fixture;
    use std::sync::Arc;
    use std::fs::File;
    use std::io::BufReader;
    use flate2::read::GzDecoder;
    use std::time::Duration;

    use crate::dto::{cohort_dto::{CohortData, CohortType, DiseaseData, GeneTranscriptData, IndividualData, ModeOfInheritance, RowData}, hpo_term_dto::{CellValue, HpoTermDuplet}};



    pub static HPO: LazyLock<Arc<FullCsrOntology>> = LazyLock::new(|| {
        let path = "resources/hp.v2025-03-03.json.gz";
        let reader = GzDecoder::new(BufReader::new(File::open(path).unwrap()));
        let loader = OntologyLoaderBuilder::new().obographs_parser().build();
        let hpo = loader.load_from_read(reader).unwrap();
        Arc::new(hpo)
    });

   
    #[fixture]
    pub fn hpo() -> Arc<FullCsrOntology> {
        Arc::clone(&HPO)
    }


    #[fixture]
    pub fn http_client() -> Arc<reqwest::blocking::Client> {
        reqwest::blocking::Client::builder()
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(10))
            .build()
            .map(Arc::new)
            .expect("Failed to build HTTP client for test fixture")
    }



    #[fixture]
    pub fn individual_data() -> IndividualData {
        let pmid = "PMID:29482508";
        let title = "Difficult diagnosis and genetic analysis of fibrodysplasia ossificans progressiva: a case report";
        let individual_id = "current case";
        let comment = "na";
        let age_of_onset = "P9Y";
        let age_at_last_encounter = "P16Y";
        let deceased = "no";
        let sex = "M";
        IndividualData::new(pmid, title, individual_id, comment, age_of_onset, age_at_last_encounter, deceased, sex)
    }

    #[fixture]
    pub fn acvr1_disease_data() -> DiseaseData {
        DiseaseData::new(  "diseaseId", "Fibrodysplasia ossificans progressiva")
    }


    #[fixture]
    pub fn hpo_headers_two_terms() -> Vec<HpoTermDuplet> {
        let d1 = HpoTermDuplet::new("Ectopic ossification in muscle tissue", "HP:0011987");
        let d2 = HpoTermDuplet::new("Elevated circulating C-reactive protein concentration", "HP:0011227");
        vec![d1, d2]
    }

    #[fixture]
    pub fn cell_values_two_terms() -> Vec<CellValue> {
        vec![CellValue::observed(), CellValue::observed()]
    }


    #[fixture]
    pub fn acvr1_cohort(
        acvr1_disease_data: DiseaseData,
        hpo_headers_two_terms: Vec<HpoTermDuplet>,
        cell_values_two_terms: Vec<CellValue>,
        individual_data: IndividualData,
        ) -> CohortData {
        let rdata = RowData{ individual_data, disease_id_list: vec![acvr1_disease_data.disease_id.to_string()], allele_count_map: HashMap::new(), hpo_data: cell_values_two_terms };
        let hpo_version = "2025-05-09";
        CohortData::mendelian(acvr1_disease_data, hpo_headers_two_terms, vec![rdata], hpo_version)
    }



// Fixture for HPO terms pool (5 different terms)
#[fixture]
pub fn hpo_term_pool() -> Vec<HpoTermDuplet> {
    vec![
        HpoTermDuplet {
            hpo_id: "HP:0002063".to_string(),
            hpo_label: "Rigidity".to_string(),
        },
        HpoTermDuplet {
            hpo_id: "HP:0004322".to_string(),
            hpo_label: "Short stature".to_string(),
        },
        HpoTermDuplet {
            hpo_id: "HP:0003228".to_string(),
            hpo_label: "Hypernatremia".to_string(),
        },
        HpoTermDuplet {
            hpo_id: "HP:0003774".to_string(),
            hpo_label: "Stage 5 chronic kidney disease".to_string(),
        },
        HpoTermDuplet {
            hpo_id: "HP:0031600".to_string(),
            hpo_label: "P wave inversion".to_string(),
        },
    ]
}


// Fixture for disease data
#[fixture]
fn disease_data() -> DiseaseData {
    DiseaseData {
        disease_id: "OMIM:157000".to_string(),
        disease_label: "Test Disease".to_string(),
        mode_of_inheritance_list: vec![
            ModeOfInheritance {
                hpo_id: "HP:0000006".to_string(),
                hpo_label: "Autosomal dominant inheritance".to_string(),
                citation: "PMID:12345678".to_string(),
            }
        ],
        gene_transcript_list: vec![
            GeneTranscriptData {
                hgnc_id: "HGNC:1100".to_string(),
                gene_symbol: "BRCA1".to_string(),
                transcript: "NM_007294.3".to_string(),
            }
        ],
    }
}


    // Fixture for first individual with HPO terms [0, 1, 2]
    #[fixture]
    fn individual_1() -> RowData {
        let hpo_data = vec![
            CellValue::observed(),
            CellValue::observed(),
            CellValue::observed(),
        ];

        RowData {
            individual_data: IndividualData::new(
                "PMID:11111111",
                "Test Study 1",
                "Individual-1",
                "Test individual 1",
                "P1Y",
                "P10Y",
                "false",
                "MALE",
            ),
            disease_id_list: vec!["OMIM:157000".to_string()],
            allele_count_map: HashMap::new(),
            hpo_data,
        }
    }

    // Fixture for second individual with HPO terms [1, 3, 4]
    #[fixture]
    fn individual_2() -> RowData {
        let hpo_data = vec![
            CellValue::observed(),
            CellValue::excluded(),
            CellValue::na()
        ];

        RowData {
            individual_data: IndividualData::new(
                "PMID:22222222",
                "Test Study 2",
                "Individual-2",
                "Test individual 2",
                "P2Y",
                "P15Y",
                "false",
                "FEMALE",
            ),
            disease_id_list: vec!["OMIM:157000".to_string()],
            allele_count_map: HashMap::new(),
            hpo_data,
        }
    }

    // Fixture for third individual with HPO terms [0, 2, 4]
    #[fixture]
    fn individual_3() -> RowData {
        let hpo_data = vec![
            CellValue::observed(),
            CellValue::excluded(),
            CellValue::observed(),
        ];

        RowData {
            individual_data: IndividualData::new(
                "PMID:33333333",
                "Test Study 3",
                "Individual-3",
                "Test individual 3",
                "P3Y",
                "P20Y",
                "false",
                "MALE",
            ),
            disease_id_list: vec!["OMIM:157000".to_string()],
            allele_count_map: HashMap::new(),
            hpo_data,
        }
    }

    // Fixture for fourth individual with HPO terms [2, 3, 4]
    #[fixture]
    fn individual_4() -> RowData {
        let hpo_data = vec![
            CellValue::observed(),
            CellValue::from_string("P2Y".to_string()).expect("critical: could not compile unit test as expected"),
            CellValue::excluded(),
        ];

        RowData {
            individual_data: IndividualData::new(
                "PMID:44444444",
                "Test Study 4",
                "Individual-4",
                "Test individual 4",
                "P4Y",
                "P25Y",
                "false",
                "FEMALE",
            ),
            disease_id_list: vec!["OMIM:157000".to_string()],
            allele_count_map: HashMap::new(),
            hpo_data,
        }
    }

    // Fixture for first CohortData with individuals 1 and 2
    #[fixture]
    pub fn cohort_data_1(
        disease_data: DiseaseData,
        hpo_term_pool: Vec<HpoTermDuplet>,
        individual_1: RowData,
        individual_2: RowData,
    ) -> CohortData {
        let mut hpo_duplets = Vec::new();
        hpo_duplets.extend_from_slice(&hpo_term_pool[0..3]);
        CohortData {
            cohort_type: CohortType::Mendelian,
            disease_list: vec![disease_data],
            hpo_headers: hpo_duplets,
            rows: vec![individual_1, individual_2],
            hgvs_variants: HashMap::new(),
            structural_variants: HashMap::new(),
            intergenic_variants: HashMap::new(),
            phetools_schema_version: "0.3".to_string(),
            hpo_version: "2024-01-01".to_string(),
            cohort_acronym: Some("COHORT1".to_string()),
            curation_history: vec![]
        }
    }

    // Fixture for second CohortData with individuals 3 and 4
    #[fixture]
    pub fn cohort_data_2(
        disease_data: DiseaseData,
        hpo_term_pool: Vec<HpoTermDuplet>,
        individual_3: RowData,
        individual_4: RowData,
    ) -> CohortData {
        let mut hpo_duplets = Vec::new();
        hpo_duplets.extend_from_slice(&hpo_term_pool[2..5]);
        CohortData {
            cohort_type: CohortType::Mendelian,
            disease_list: vec![disease_data],
            hpo_headers: hpo_duplets,
            rows: vec![individual_3, individual_4],
            hgvs_variants: HashMap::new(),
            structural_variants: HashMap::new(),
            intergenic_variants: HashMap::new(),
            phetools_schema_version: "0.3".to_string(),
            hpo_version: "2024-01-01".to_string(),
            cohort_acronym: Some("COHORT2".to_string()),
            curation_history: vec![]
        }
    }


    /// Create a cohort in which all HPO column entries are "na" at index 1
    #[fixture]
    pub fn cohort_with_na_column(
        disease_data: DiseaseData,
        hpo_term_pool: Vec<HpoTermDuplet>,
        individual_1: RowData,
        individual_2: RowData,
        #[default(1)] na_col_idx: usize,
    ) -> CohortData {
        let mut rows = vec![individual_1, individual_2];
        let na_col_idx: usize = 1;
        for row in rows.iter_mut() {
            row.hpo_data[na_col_idx] = CellValue::na();
        }

        let hpo_headers = hpo_term_pool[0..3].to_vec();

          let cd = CohortData {
            cohort_type: CohortType::Mendelian,
            disease_list: vec![disease_data],
            hpo_headers: hpo_headers,
            rows: rows,
            hgvs_variants: HashMap::new(),
            structural_variants: HashMap::new(),
            intergenic_variants: HashMap::new(),
            phetools_schema_version: "0.3".to_string(),
            hpo_version: "2024-01-01".to_string(),
            cohort_acronym: Some("NA_COHORT".to_string()),
            curation_history: vec![],
        };
        cd
    }
}