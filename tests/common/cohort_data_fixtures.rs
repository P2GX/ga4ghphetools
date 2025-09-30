/// Data (fixtures) for tests involving manipulation of CohortData objects

use std::collections::HashMap;

use ga4ghphetools::dto::{cohort_dto::{CohortData, CohortType, DiseaseData, GeneTranscriptData, IndividualData, ModeOfInheritance, RowData}, hpo_term_dto::{CellValue, HpoTermDuplet}};

use rstest::fixture;


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
fn individual_1(hpo_term_pool: Vec<HpoTermDuplet>) -> RowData {
    let hpo_data = vec![
        CellValue::Observed,
        CellValue::Observed,
        CellValue::Observed,
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
fn individual_2(hpo_term_pool: Vec<HpoTermDuplet>) -> RowData {
    let hpo_data = vec![
        CellValue::Observed,
        CellValue::Excluded,
        CellValue::Na,
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
fn individual_3(hpo_term_pool: Vec<HpoTermDuplet>) -> RowData {
    let hpo_data = vec![
        CellValue::Observed,
        CellValue::Excluded,
        CellValue::Observed,
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
fn individual_4(hpo_term_pool: Vec<HpoTermDuplet>) -> RowData {
    let hpo_data = vec![
        CellValue::Observed,
        CellValue::OnsetAge("P2Y".to_string()),
        CellValue::Excluded,
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
        phetools_schema_version: "0.2".to_string(),
        hpo_version: "2024-01-01".to_string(),
        cohort_acronym: Some("COHORT1".to_string()),
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
        phetools_schema_version: "0.2".to_string(),
        hpo_version: "2024-01-01".to_string(),
        cohort_acronym: Some("COHORT2".to_string()),
    }
}

