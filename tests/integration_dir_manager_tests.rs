mod common;

use std::sync::Arc;

use common::case_5_dto;
use common::hpo_dto_list_1;
use ontolius::ontology::csr::FullCsrOntology;
use rphetools::dto::case_dto::CaseDto;
use rphetools::dto::hpo_term_dto::HpoTermDto;
use rphetools::PheTools;
use rstest::rstest;
use common::hpo;
use common::matrix;
use zip::result;


#[rstest]
#[ignore = "testing requires API"]
fn test_variant_manager_cache(
        matrix: Vec<Vec<String>>, 
        hpo: Arc<FullCsrOntology>,
        case_5_dto: CaseDto,
        hpo_dto_list_1: Vec<HpoTermDto>
    ) {
    let mut phetools = PheTools::new(hpo);
    let original_matrix = matrix.clone();
    let res = phetools.load_matrix(matrix);
    let dir_path = "/tmp/vman";
    let result = phetools.set_cache_location(dir_path);
    assert!(result.is_ok());
    let result = phetools.validate_all_variants();

    

/*
    let result = DirManager::new(dir_path);
    assert!(result.is_ok());
    let mut manager = result.unwrap();
    // the manager may have variants from previous tests, therefore we need to clear the cacher
    manager.clear_cache();
   assert_eq!(0, manager.n_hgvs());
   assert_eq!(0, manager.n_sv());
    // NM_002834.5(PTPN11):c.178G>T (p.Gly60Cys)
    let result = manager.validate_hgvs("c.178G>T", "NM_002834.5");

    assert_eq!(1, manager.n_hgvs());
    // should get from cache
    manager.validate_hgvs("c.178G>T", "NM_002834.5");
    assert_eq!(1, manager.n_hgvs());
    // NM_002834.5(PTPN11):c.181G>A (p.Asp61Asn)
    manager.validate_hgvs("c.181G>A","NM_002834.5" );
    assert_eq!(2, manager.n_hgvs());
    */
}