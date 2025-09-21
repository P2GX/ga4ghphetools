mod common;

use std::collections::HashMap;
use std::sync::Arc;

use ga4ghphetools::dto::cohort_dto::CohortData;
use ga4ghphetools::dto::cohort_dto::CohortType;
use ga4ghphetools::dto::cohort_dto::DiseaseData;
use ga4ghphetools::dto::cohort_dto::IndividualData;
use ga4ghphetools::dto::cohort_dto::RowData;
use ga4ghphetools::dto::hpo_term_dto::CellValue;
use ga4ghphetools::dto::hpo_term_dto::HpoTermData;
use ga4ghphetools::dto::hpo_term_dto::HpoTermDuplet;
use ga4ghphetools::factory::cohort_factory::CohortFactory;
use ontolius::ontology::csr::FullCsrOntology;
use ontolius::ontology::MetadataAware;
use rstest::rstest;
use common::hpo;


use crate::common::acvr1_cohort;
use crate::common::acvr1_disease_data;
use crate::common::cell_values_two_terms;
use crate::common::hpo_headers_two_terms;
use crate::common::individual_data;



    #[rstest]
    fn test_add_one_hpo_term(
        acvr1_cohort: CohortData,
        hpo: Arc<FullCsrOntology>
    ) {
        assert_eq!(acvr1_cohort.cohort_type, CohortType::Mendelian);
        assert_eq!(acvr1_cohort.hpo_headers.len(), 2);
        assert_eq!(acvr1_cohort.rows.len(), 1);
        let row0 = acvr1_cohort.rows.get(0).cloned().unwrap();
        assert_eq!(2, row0.hpo_data.len());
        let hpo_term = HpoTermDuplet::new("Long hallux", "HP:0001847");
        let mut factory = CohortFactory::new(hpo.clone());
        let result = factory.add_hpo_term_to_cohort(hpo_term.hpo_id(), hpo_term.hpo_label(), acvr1_cohort);
        if result.is_err() {
            let err = result.err().unwrap();
            println!("{}", err);
        } else {
        assert!(result.is_ok());
        let new_cohort = result.unwrap();
        assert_eq!(3, new_cohort.hpo_headers.len());
        let row0 = new_cohort.rows.get(0).cloned().unwrap();
        assert_eq!(3, row0.hpo_data.len());
        // check that all three expected terms are in the new row. The order does not matter here, but the values do
        let hpo_data = row0.hpo_data.clone();
        let hpo_header_dup = new_cohort.hpo_headers.clone();
        let hpo_data0 = HpoTermData::new(hpo_header_dup[0].clone(), hpo_data[0].clone()).unwrap();
        let hpo_data1 = HpoTermData::new(hpo_header_dup[1].clone(), hpo_data[1].clone()).unwrap();
        let hpo_data2 = HpoTermData::new(hpo_header_dup[2].clone(), hpo_data[2].clone()).unwrap();
        let datavec = vec![hpo_data0, hpo_data1, hpo_data2];
        for htd in datavec {
            if htd.term_id() == "HP:0001847" {
                assert_eq!(htd.label(), "Long hallux");
                assert_eq!(htd.entry(), "na");
            } else if htd.term_id() == "HP:0011987" {
                 assert_eq!(htd.label(), "Ectopic ossification in muscle tissue");
                assert_eq!(htd.entry(), "observed");
            } else if htd.term_id() == "HP:0011227" {
                assert_eq!(htd.label(), "Elevated circulating C-reactive protein concentration");
                assert_eq!(htd.entry(), "observed");
            } else {
                assert!(false, "Could not find {}", htd.term_id());
            }
        }
    }

}



/// Create a Cohort with three terms in which that first and the third terms are identical.
/// The purpose is that we can see if the ingest function throws and error -- this needs to 
/// be cleaned up before further processing.
#[rstest]
pub fn acvr1_cohort_with_repeated_term(
    mut hpo_headers_two_terms: Vec<HpoTermDuplet>,
    mut cell_values_two_terms: Vec<CellValue>,
    individual_data: IndividualData,
    acvr1_disease_data: DiseaseData,
    hpo: Arc<FullCsrOntology>
)  {
    if let Some(first) = hpo_headers_two_terms.first().cloned() {
        hpo_headers_two_terms.push(first);
    } else {
        assert!(false, "could not add HPO term");
    }
    if let Some(first) = cell_values_two_terms.first().cloned() {
        cell_values_two_terms.push(first);
    }
    
     let rdata = RowData{ individual_data, disease_id_list: vec![acvr1_disease_data.disease_id.to_string()], allele_count_map: HashMap::new(), hpo_data: cell_values_two_terms };

    let cohort_data = CohortData::mendelian(acvr1_disease_data, hpo_headers_two_terms, vec![rdata], hpo.version());
    let result = ga4ghphetools::factory::qc_assessment(hpo, &cohort_data);
    assert!(result.is_err());
    let err_str = result.err().unwrap();
    assert_eq!("Duplicate entry in HPO Header: Ectopic ossification in muscle tissue (HP:0011987)", err_str);
    
}







