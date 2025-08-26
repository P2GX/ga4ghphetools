mod common;

use std::sync::Arc;

use ga4ghphetools::dto::cohort_dto::CohortData;
use ga4ghphetools::dto::cohort_dto::CohortType;
use ga4ghphetools::dto::hpo_term_dto::HpoTermData;
use ga4ghphetools::dto::hpo_term_dto::HpoTermDuplet;
use ga4ghphetools::template::cohort_dto_builder::CohortDtoBuilder;
use ontolius::ontology::csr::FullCsrOntology;
use ga4ghphetools::PheTools;
use rstest::rstest;
use common::hpo;
use common::matrix;

use crate::common::acvr1_cohort;



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
        let mut builder = CohortDtoBuilder::new(acvr1_cohort.cohort_type, acvr1_cohort.disease_gene_data.clone(), hpo.clone());
        let result = builder.add_hpo_term_to_cohort(hpo_term.hpo_id(), hpo_term.hpo_label(), acvr1_cohort);
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










