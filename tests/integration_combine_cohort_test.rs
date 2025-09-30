use std::{clone, collections::HashMap, sync::Arc};

use ga4ghphetools::dto::{cohort_dto::{CohortData, RowData}, hpo_term_dto::{CellValue, HpoTermDuplet}};
use ontolius::ontology::csr::FullCsrOntology;
use rstest::{rstest};
mod common;
use common::cohort_data_fixtures::{cohort_data_1, cohort_data_2, hpo_term_pool};

use crate::common::hpo_fixture::hpo;



// Make sure the CohortData fixtures are working before we do the test
#[rstest]
fn test_two_cohorts_with_hpo_terms(
    cohort_data_1: CohortData,
    cohort_data_2: CohortData,
) {
    assert_eq!(cohort_data_1.rows.len(), 2);
    assert_eq!(cohort_data_1.rows[0].hpo_data.len(), 3);
    assert_eq!(cohort_data_1.rows[1].hpo_data.len(), 3);
    assert_eq!(cohort_data_1.hpo_headers.len(), 3);

    assert_eq!(cohort_data_2.rows.len(), 2);
    assert_eq!(cohort_data_2.rows[0].hpo_data.len(), 3);
    assert_eq!(cohort_data_2.rows[1].hpo_data.len(), 3);
    assert_eq!(cohort_data_2.hpo_headers.len(), 3);

    assert!(cohort_data_1.is_mendelian());
    assert!(cohort_data_2.is_mendelian());
}




// Example test verifying specific HPO term distribution
#[rstest]
fn test_hpo_term_distribution(
    cohort_data_1: CohortData,
    hpo_term_pool: Vec<HpoTermDuplet>,
) {
    // Individual 1 should have terms 0, 1, 2
    let ind1_hpo = &cohort_data_1.rows[0].hpo_data;
    // Individual 2 should have terms 1, 3, 4
    let ind2_hpo = &cohort_data_1.rows[1].hpo_data;

    // Add your specific assertions here based on your CellValue enum structure
    assert_eq!(ind1_hpo.len(), 3);
    assert_eq!(ind2_hpo.len(), 3);
}


#[rstest]
fn test_combine_cohort(
    cohort_data_1: CohortData,
    cohort_data_2: CohortData,
    hpo: Arc<FullCsrOntology>
)  {
    let result = ga4ghphetools::factory::merge_cohort_data_from_etl_dto(cohort_data_1, cohort_data_2, hpo.clone());
    assert!(result.is_ok());
}




fn get_map_current_annotations(hpo_headers: &Vec<HpoTermDuplet>, row: &RowData) -> HashMap<HpoTermDuplet, CellValue> {
    let mut val_map = HashMap::new();
    for (duplet, val) in hpo_headers.iter().zip(row.hpo_data.iter()) {
        val_map.insert(duplet.clone(), val.clone());
    }
    val_map
}


/// Individual 1 has three terms, all observed. When we merge, we expect to see Na for the other two terms.
#[rstest]
fn test_individual_1(
    cohort_data_1: CohortData,
    cohort_data_2: CohortData,
    hpo: Arc<FullCsrOntology>
)  {
    let cohort = ga4ghphetools::factory::merge_cohort_data_from_etl_dto(cohort_data_1.clone(), cohort_data_2, hpo.clone()).unwrap();
    let row_1_val_map = get_map_current_annotations(&cohort_data_1.hpo_headers, &cohort_data_1.rows[0]);
    let hpo_duplets = cohort.hpo_headers.clone();
    let row_1_merged = cohort.rows[0].clone();
    for (duplet, val) in hpo_duplets.iter().zip(row_1_merged.hpo_data.iter()) {
        let merged_val = match row_1_val_map.get(duplet) {
            Some(val) => val.clone(),
            None => CellValue::Na,
        };
        assert_eq!(val.clone(), merged_val);
    }
}

/// Individual 2 has three terms (O, E, na). When we merge, we expect to see Na for the other two terms.
#[rstest]
fn test_individual_2(
    cohort_data_1: CohortData,
    cohort_data_2: CohortData,
    hpo: Arc<FullCsrOntology>
)  {
    let cohort = ga4ghphetools::factory::merge_cohort_data_from_etl_dto(cohort_data_1.clone(), cohort_data_2, hpo.clone()).unwrap();
    let row_2_val_map = get_map_current_annotations(&cohort_data_1.hpo_headers, &cohort_data_1.rows[1]);
    let hpo_duplets = cohort.hpo_headers.clone();
    let row_2_merged = cohort.rows[1].clone();
    for (duplet, val) in hpo_duplets.iter().zip(row_2_merged.hpo_data.iter()) {
        let merged_val = match row_2_val_map.get(duplet) {
            Some(val) => val.clone(),
            None => CellValue::Na,
        };
        assert_eq!(val.clone(), merged_val);
    }
}

/// Individual 3 has three terms (O, E, 0). We are merging this into the previous cohort.
#[rstest]
fn test_individual_3(
    cohort_data_1: CohortData,
    cohort_data_2: CohortData,
    hpo: Arc<FullCsrOntology>
)  {
    let cohort = ga4ghphetools::factory::merge_cohort_data_from_etl_dto(cohort_data_1.clone(), cohort_data_2.clone(), hpo.clone()).unwrap();
    let row_3_val_map = get_map_current_annotations(&cohort_data_2.hpo_headers, &cohort_data_2.rows[0]);
    let hpo_duplets = cohort.hpo_headers.clone();
    let row_3_merged = cohort.rows[2].clone();
    for (duplet, val) in hpo_duplets.iter().zip(row_3_merged.hpo_data.iter()) {
        let merged_val = match row_3_val_map.get(duplet) {
            Some(val) => val.clone(),
            None => CellValue::Na,
        };
        assert_eq!(val.clone(), merged_val);
    }
}

/// Individual 4 has three terms (O, P2Y, E). We are merging this into the previous cohort.
#[rstest]
fn test_individual_4(
    cohort_data_1: CohortData,
    cohort_data_2: CohortData,
    hpo: Arc<FullCsrOntology>
)  {
    let cohort = ga4ghphetools::factory::merge_cohort_data_from_etl_dto(cohort_data_1.clone(), cohort_data_2.clone(), hpo.clone()).unwrap();
    let row_4_val_map = get_map_current_annotations(&cohort_data_2.hpo_headers, &cohort_data_2.rows[1]);
    let hpo_duplets = cohort.hpo_headers.clone();
    let row_4_merged = cohort.rows[3].clone();
    for (duplet, val) in hpo_duplets.iter().zip(row_4_merged.hpo_data.iter()) {
        let merged_val = match row_4_val_map.get(duplet) {
            Some(val) => val.clone(),
            None => CellValue::Na,
        };
        assert_eq!(val.clone(), merged_val);
    }
}