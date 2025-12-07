mod common;
use std::collections::HashMap;
use std::sync::Arc;

use ga4ghphetools::dto::cohort_dto::GeneTranscriptData;
use ga4ghphetools::dto::etl_dto::ColumnDto;
use ga4ghphetools::dto::etl_dto::ColumnTableDto;
use ga4ghphetools::dto::etl_dto::EtlColumnHeader;
use ga4ghphetools::dto::etl_dto::EtlColumnType;
use ga4ghphetools::dto::etl_dto::EtlDto;
use ga4ghphetools::dto::hgvs_variant::HgvsVariant;
use ga4ghphetools::dto::hpo_term_dto::CellValue;
use ga4ghphetools::dto::hpo_term_dto::HpoTermData;
use ga4ghphetools::dto::hpo_term_dto::HpoTermDuplet;
use ga4ghphetools::dto::cohort_dto::DiseaseData;
use common::hpo_fixture::hpo;
use ontolius::ontology::csr::FullCsrOntology;
use rstest::fixture;
use rstest::rstest;






#[fixture]
fn patient_id_column_valid() -> ColumnDto {
    ColumnDto {
        id: "86fa387b-2d5c-4f84-9437-f3abe3bd7ba3".to_string(),
        transformed: true,
        header: EtlColumnHeader{
            original: "Clinical Features".to_string(),
            current: None,
            column_type: EtlColumnType::PatientId,
            hpo_terms: None,
        },
        values: vec![
            "Family 1 (Turkish) BAB11420".to_string(),
            "Family 2 (Saudi) Proband 1".to_string()
        ]
    }
}

#[fixture]
fn variant_column_valid() -> ColumnDto {
    ColumnDto {
        id: "35f95771-74cd-485d-9289-52de83dbe10d".to_string(),
        transformed: true,
        header: EtlColumnHeader{
            original: "Mutation (NM_016145.4)".to_string(),
            current: Some("Mutation (NM_016145.4)-validated".to_string()),
            column_type: EtlColumnType::Variant,
            hpo_terms: None,
        },
        values: vec![
            "c235CtoT_WDR83OS_NM_016145v4".to_string(),
            "c156_1GtoT_WDR83OS_NM_016145v4".to_string()
        ]
    }
}

#[fixture]
fn age_eval_column_valid() -> ColumnDto {
    ColumnDto {
        id: "09e8235b-d9dd-47e0-9831-68981a24fa93".to_string(),
        transformed: true,
        header: EtlColumnHeader{
            original: "Age at evaluation".to_string(),
            current: None,
            column_type: EtlColumnType::AgeAtLastEncounter,
            hpo_terms: None,
        },
        values: vec![
           "P3Y6M".to_string(),
           "P25Y".to_string()
        ]
    }
}

#[fixture]
fn sex_column_valid() -> ColumnDto {
    ColumnDto {
        id: "194ec5a1-ea0e-429d-a9c0-3af3aaaaadfc".to_string(),
        transformed: true,
        header: EtlColumnHeader{
            original: "Sex".to_string(),
            current: None,
            column_type: EtlColumnType::Sex,
            hpo_terms: None,
        },
        values: vec![
           "M".to_string(),
           "F".to_string()
        ]
    }
}

#[fixture]
fn delayed_sit_column_valid() -> ColumnDto {
 ColumnDto {
        id: "56988503-a04b-4783-ab5c-41afb0eb136f".to_string(),
        transformed: true,
        header: EtlColumnHeader{
            original: "Age of sit".to_string(),
            current: Some("Delayed ability to sit - HP:0025336".to_string()),
            column_type: EtlColumnType::SingleHpoTerm,
            hpo_terms: Some(vec![
                HpoTermDuplet::new("Delayed ability to sit","HP:0025336" )
            ]),
        },
        values: vec![
           "observed".to_string(),
           "na".to_string()
        ]
    }
}

#[fixture]
fn delayed_gross_motor() -> ColumnDto {
    // Delayed gross motor development HP:0002194 -- parent of Delayed ability to sit, this may be redundant
    ColumnDto {
        id: "56988503-a04b-4783-ab5c-41afb0eb131a".to_string(),
        transformed: true,
        header: EtlColumnHeader{
            original: "Age of sit".to_string(),
            current: Some("Delayed gross motor development - HP:0002194".to_string()),
            column_type: EtlColumnType::SingleHpoTerm,
            hpo_terms: Some(vec![
                HpoTermDuplet::new("Delayed gross motor development","HP:0002194" )
            ]),
        },
        values: vec![
           "observed".to_string(),
           "na".to_string()
        ]
    }

}


#[fixture]
fn gdd_column_valid() -> ColumnDto {
 ColumnDto {
        id: "61d07214-abb0-45cf-aa67-3218730416c0".to_string(),
        transformed: true,
        header: EtlColumnHeader{
            original: "Other developmental steps".to_string(),
            current: Some("Multiple HPO terms - Other developmental steps".to_string()),
            column_type: EtlColumnType::MultipleHpoTerm,
            hpo_terms: Some(vec![
                HpoTermDuplet::new("Global developmental delay", "HP:0001263" )
            ]),
        },
        values: vec![
             "HP:0001263-observed".to_string(),
          "HP:0001263-observed".to_string()
        ]
    }
}


#[fixture]
fn hypertelorism_column_valid() -> ColumnDto {
    ColumnDto {
        id: "5697a6ef-4855-4230-9bb1-a8511acadcb3".to_string(),
        transformed: true,
        header: EtlColumnHeader {
            original: "Hypotelorism/hypertelorism".to_string(),
            current: Some("Multiple HPO terms - Hypotelorism/hypertelorism".to_string()),
            column_type: EtlColumnType::MultipleHpoTerm,
            hpo_terms: Some(vec![
                HpoTermDuplet::new("Hypotelorism", "HP:0000601"),
                HpoTermDuplet::new("Hypertelorism", "HP:0000316"),
            ]),
        },
        values: vec![
            "HP:0000601-observed;HP:0000316-excluded".to_string(),
            "HP:0000601-excluded;HP:0000316-observed".to_string(),
        ],
    }
}

#[fixture]
fn column_strabismus() -> ColumnDto {
    ColumnDto {
        id: "0cfed850-7d6f-4344-b52b-a12f86ff0f85".to_string(),
        transformed: true,
        header: EtlColumnHeader {
            original: "Strabismus".to_string(),
            current: Some("Strabismus - HP:0000486".to_string()),
            column_type: EtlColumnType::SingleHpoTerm,
            hpo_terms: Some(vec![HpoTermDuplet::new("Strabismus", "HP:0000486")]),
        },
        values: vec!["observed".to_string(), "excluded".to_string()],
    }
}

#[fixture]
fn column_ptosis() -> ColumnDto {
    ColumnDto {
        id: "c41e1b1c-2610-4414-a754-ca0e2117816d".to_string(),
        transformed: true,
        header: EtlColumnHeader {
            original: "Ptosis".to_string(),
            current: Some("Ptosis - HP:0000508".to_string()),
            column_type: EtlColumnType::SingleHpoTerm,
            hpo_terms: Some(vec![HpoTermDuplet::new("Ptosis", "HP:0000508")]),
        },
        values: vec!["observed".to_string(), "excluded".to_string()],
    }
}


#[fixture]
fn exclude_abn_eye() -> ColumnDto {
    //  HP:
    ColumnDto {
        id: "c41e1b1c-abdc-4414-a754-ca0e2117816d".to_string(),
        transformed: true,
        header: EtlColumnHeader {
            original: "Abnormality of the eye".to_string(),
            current: Some("Abnormality of the eye - HP:0000478".to_string()),
            column_type: EtlColumnType::SingleHpoTerm,
            hpo_terms: Some(vec![HpoTermDuplet::new("Abnormality of the eye", "HP:0000478")]),
        },
        values: vec!["excluded".to_string(), "excluded".to_string()],
    }
}

/// This is invalid because the column type is Raw
#[fixture]
fn column_ptosis_invalid_raw(column_ptosis: ColumnDto) -> ColumnDto {
    let mut col = column_ptosis;
    col.header.column_type = EtlColumnType::Raw;
    col
}

#[fixture]
fn valid_columns(
    patient_id_column_valid: ColumnDto,
    variant_column_valid: ColumnDto,
    age_eval_column_valid: ColumnDto,
    sex_column_valid: ColumnDto,
    delayed_sit_column_valid: ColumnDto,
    gdd_column_valid: ColumnDto,
    hypertelorism_column_valid: ColumnDto,
    column_ptosis: ColumnDto,
    column_strabismus: ColumnDto
) -> Vec<ColumnDto> {
    vec![patient_id_column_valid, variant_column_valid, age_eval_column_valid, sex_column_valid, delayed_sit_column_valid, 
        gdd_column_valid, hypertelorism_column_valid, column_ptosis, column_strabismus]
}

#[fixture]
fn columns_lacking_patient_id(
    variant_column_valid: ColumnDto,
    age_eval_column_valid: ColumnDto,
    sex_column_valid: ColumnDto,
    delayed_sit_column_valid: ColumnDto,
    gdd_column_valid: ColumnDto,
    hypertelorism_column_valid: ColumnDto,
    column_ptosis: ColumnDto,
    column_strabismus: ColumnDto
) -> Vec<ColumnDto> {
    vec![variant_column_valid, age_eval_column_valid, sex_column_valid, delayed_sit_column_valid, 
        gdd_column_valid, hypertelorism_column_valid, column_ptosis, column_strabismus]
}

#[fixture]
fn columns_lacking_hpo(
    patient_id_column_valid: ColumnDto,
    variant_column_valid: ColumnDto,
    age_eval_column_valid: ColumnDto,
    sex_column_valid: ColumnDto,
) -> Vec<ColumnDto> {
    vec![patient_id_column_valid, variant_column_valid, age_eval_column_valid, sex_column_valid]
}

#[fixture]
fn disease_valid() -> DiseaseData {
    DiseaseData {
        disease_id: "OMIM:621016".to_string(),
        disease_label: "Neurodevelopmental disorder with variable familial hypercholanemia".to_string(),
        mode_of_inheritance_list: vec![],
        gene_transcript_list: vec![GeneTranscriptData {
            hgnc_id: "HGNC:30203".to_string(),
            gene_symbol: "WDR83OS".to_string(),
            transcript: "NM_016145.4".to_string(),
        }],
    }
}

#[fixture]
fn columns_with_redudancy(
    patient_id_column_valid: ColumnDto,
    variant_column_valid: ColumnDto,
    age_eval_column_valid: ColumnDto,
    sex_column_valid: ColumnDto,
    delayed_sit_column_valid: ColumnDto,
    gdd_column_valid: ColumnDto,
    hypertelorism_column_valid: ColumnDto,
    column_ptosis: ColumnDto,
    column_strabismus: ColumnDto,
    delayed_gross_motor: ColumnDto,
) -> Vec<ColumnDto> {
    vec![patient_id_column_valid, variant_column_valid, age_eval_column_valid, sex_column_valid, delayed_sit_column_valid, 
        gdd_column_valid, hypertelorism_column_valid, column_ptosis, column_strabismus, delayed_gross_motor]
}


#[fixture]
fn hgvs_var_1_valid() -> HgvsVariant {
    HgvsVariant::new_from_parts(
        "hg38".to_string(), 
        "chr19".to_string(), 
        12668539, 
        "G".to_string(), 
        "A".to_string(), 
        "WDR83OS".to_string(), 
        "HGNC:30203".to_string(), 
        "c.235C>T".to_string(), 
         "NM_016145.4".to_string(), 
         "NC_000019.10:g.12668539G>A".to_string())
}
#[fixture]
fn hgvs_var_2_valid() -> HgvsVariant {
    HgvsVariant::new_from_parts(
        "hg38".to_string(), 
        "chr19".to_string(), 
        12669127, 
        "C".to_string(), 
        "A".to_string(), 
        "WDR83OS".to_string(), 
        "HGNC:30203".to_string(), 
        "c.156+1G>T".to_string(), 
        "NM_016145.4".to_string(), 
        "NC_000019.10:g.12669127C>A".to_string())
}

#[fixture]
fn hgvs_map(
    hgvs_var_1_valid: HgvsVariant,
    hgvs_var_2_valid: HgvsVariant
) -> HashMap<String, HgvsVariant> {
    let mut map: HashMap<String, HgvsVariant> = HashMap::new();
    map.insert(hgvs_var_1_valid.variant_key(), hgvs_var_1_valid.clone());
    map.insert(hgvs_var_2_valid.variant_key(), hgvs_var_2_valid);
    map
}


fn make_table(columns: Vec<ColumnDto>) -> ColumnTableDto {
    ColumnTableDto {
        file_name: "/Users/name/Desktop/stuff/Demo.xlsx".to_string(), 
        columns,
    }
}

fn make_etl(table: ColumnTableDto, disease: DiseaseData) -> EtlDto {
    EtlDto {
        table,
        disease: Some(disease),
        pmid: Some("PMID:39471804".to_string()), 
        title: Some("Homozygous variants in WDR83OS lead to a neurodevelopmental disorder with hypercholanemia.".to_string()), 
        hgvs_variants: Default::default(),
        structural_variants: Default::default(),
    }
}


#[fixture]
fn etl_dto_valid(
    valid_columns: Vec<ColumnDto>,
    disease_valid: DiseaseData,
    hgvs_map: HashMap<String, HgvsVariant>
) -> EtlDto {
    let table: ColumnTableDto = make_table(valid_columns);
    let mut etl_dto = make_etl(table, disease_valid);
    etl_dto.hgvs_variants = hgvs_map;
    etl_dto
}

#[fixture]
fn etl_dto_lacking_patient_id(
    columns_lacking_patient_id: Vec<ColumnDto>,
    disease_valid: DiseaseData,
    hgvs_map: HashMap<String, HgvsVariant>
) -> EtlDto {
    let table: ColumnTableDto = make_table(columns_lacking_patient_id);
    let mut etl_dto = make_etl(table, disease_valid);
    etl_dto.hgvs_variants = hgvs_map;
    etl_dto
}

#[fixture]
fn etl_dto_lacking_hpo(
    columns_lacking_hpo: Vec<ColumnDto>,
    disease_valid: DiseaseData,
    hgvs_map: HashMap<String, HgvsVariant>
) -> EtlDto {
    let table: ColumnTableDto = make_table(columns_lacking_hpo);
    let mut etl_dto = make_etl(table, disease_valid);
    etl_dto.hgvs_variants = hgvs_map;
    etl_dto
}



#[fixture]
fn etl_dto_with_redudancy(
    columns_with_redudancy: Vec<ColumnDto>,
    disease_valid: DiseaseData,
    hgvs_map: HashMap<String, HgvsVariant>
) -> EtlDto {
    let table: ColumnTableDto = make_table(columns_with_redudancy);
    let mut etl_dto = make_etl(table, disease_valid);
    etl_dto.hgvs_variants = hgvs_map;
    etl_dto
}





#[rstest]
fn test_variant_key(
    hgvs_var_1_valid: HgvsVariant
) {
    let expected_key = "c235CtoT_WDR83OS_NM_016145v4";
    assert_eq!(expected_key, hgvs_var_1_valid.variant_key(), "Mismatched variant key");
}

#[rstest]
fn test_valid_etl(
    etl_dto_valid: EtlDto,
    hpo: Arc<FullCsrOntology>) {
    let result = ga4ghphetools::etl::get_cohort_data_from_etl_dto(hpo.clone(), etl_dto_valid);
    assert!(result.is_ok());
    let cohort_dto = result.unwrap();
    let qc = ga4ghphetools::factory::qc_assessment(hpo, &cohort_dto);
}

#[rstest]
fn test_invalid_column_type_raw(
    column_ptosis_invalid_raw: ColumnDto,
    disease_valid: DiseaseData,
    hpo: Arc<FullCsrOntology>
) {
    let table = make_table(vec![column_ptosis_invalid_raw]);
    let etl = make_etl(table, disease_valid);
    let result = ga4ghphetools::etl::get_cohort_data_from_etl_dto(hpo, etl);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err, "'Ptosis' column type not set (Raw)");
}

#[rstest]
fn test_empty_columns(
    disease_valid: DiseaseData,
    hpo: Arc<FullCsrOntology>
) {
    let table: Vec<ColumnDto> = vec![];
    let table_dto = ColumnTableDto{ file_name: "/Users/fakename.xlsx".to_string(), columns: table };
    let etl = make_etl(table_dto, disease_valid);
    let result = ga4ghphetools::etl::get_cohort_data_from_etl_dto(hpo, etl);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err, "EtlDto table with no columns");
}


#[rstest]
fn test_column_type_with_leading_whitespace(
    column_ptosis: ColumnDto,
    disease_valid: DiseaseData,
    hpo: Arc<FullCsrOntology>
) {
    let mut ws_ptosis_col = column_ptosis.clone();
    if let Some(first_val) = ws_ptosis_col.values.get_mut(0) {
        *first_val = format!(" {}", first_val); // prepend leading whitespace
    }
    let table = make_table(vec![ws_ptosis_col]);
    let etl = make_etl(table, disease_valid);
    let result = ga4ghphetools::etl::get_cohort_data_from_etl_dto(hpo, etl);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err, "Ptosis: leading whitespace - ' observed'");
}

#[rstest]
fn test_column_type_with_trailing_whitespace(
    column_ptosis: ColumnDto,
    disease_valid: DiseaseData,
    hpo: Arc<FullCsrOntology>
) {
    let mut ws_ptosis_col = column_ptosis.clone();
    if let Some(first_val) = ws_ptosis_col.values.get_mut(0) {
        *first_val = format!("{} ", first_val); // prepend leading whitespace
    }
    let table = make_table(vec![ws_ptosis_col]);
    let etl = make_etl(table, disease_valid);
    let result = ga4ghphetools::etl::get_cohort_data_from_etl_dto(hpo, etl);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err, "Ptosis: trailing whitespace - 'observed '");
}

/// Insert an invisible whitespace. We find this in HGVS expressions of some external files and need to remove it.
#[rstest]
fn test_column_type_with_invalid_char(
    column_ptosis: ColumnDto,
    disease_valid: DiseaseData,
    hpo: Arc<FullCsrOntology>
) {
    let mut ws_ptosis_col = column_ptosis.clone();
    if let Some(first_val) = ws_ptosis_col.values.get_mut(0) {
        // Insert a ZERO WIDTH SPACE (U+200B) after the 2nd character
        let insert_pos = 2.min(first_val.len()); 
        first_val.insert(insert_pos, '\u{200B}');
    }
    let table = make_table(vec![ws_ptosis_col]);
    let etl = make_etl(table, disease_valid);
    let result = ga4ghphetools::etl::get_cohort_data_from_etl_dto(hpo, etl);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err, "Ptosis: Invalid character: U+200B '\u{200b}'");
}

/// We demand that all alleles (in the Variant columns) are mapped to an HgvsVariant or StructuralVariant.
#[rstest]
fn test_missing_hgvs(
    etl_dto_valid: EtlDto,
    hpo: Arc<FullCsrOntology>
) {
    let mut etl_dto = etl_dto_valid.clone();
    let removed = etl_dto.hgvs_variants
        .remove("c235CtoT_WDR83OS_NM_016145v4");
    assert!(removed.is_some(), "Entry was not found in hgvs_variants"); // make sure we actually remove the variant
    // This should be an error because the Variant row has an allele key that is not in our map
    let result = ga4ghphetools::etl::get_cohort_data_from_etl_dto(hpo, etl_dto);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err, "Unmapped allele: 'c235CtoT_WDR83OS_NM_016145v4'")
}

/// PatientId column is required
#[rstest]
fn test_missing_patient_id(
    etl_dto_lacking_patient_id: EtlDto,
    hpo: Arc<FullCsrOntology>
) {
    let result = ga4ghphetools::etl::get_cohort_data_from_etl_dto(hpo, etl_dto_lacking_patient_id);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err, "No patient identifier column found")
}


/// At least one HPO column required
#[rstest]
fn test_no_hpo_column(
    etl_dto_lacking_hpo: EtlDto,
    hpo: Arc<FullCsrOntology>
) {
    let result = ga4ghphetools::etl::get_cohort_data_from_etl_dto(hpo, etl_dto_lacking_hpo);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err, "No HPO columns found")
}



#[rstest]
fn test_no_pmid(
    etl_dto_valid: EtlDto,
    hpo: Arc<FullCsrOntology>
) {
    let mut etl = etl_dto_valid.clone();
    etl.pmid = None;
    let result = ga4ghphetools::etl::get_cohort_data_from_etl_dto(hpo, etl);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err, "No PMID found")
}



#[rstest]
fn test_malformed_pmid(
    etl_dto_valid: EtlDto,
    hpo: Arc<FullCsrOntology>
) {
    let mut etl = etl_dto_valid.clone();
    etl.pmid = Some("PMID: 123456".to_string());
    let result = ga4ghphetools::etl::get_cohort_data_from_etl_dto(hpo, etl);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err, "Malformed PMID found 'PMID: 123456'")
}

#[rstest]
fn test_no_title(
    etl_dto_valid: EtlDto,
    hpo: Arc<FullCsrOntology>
) {
    let mut etl = etl_dto_valid.clone();
    etl.title = None;
    let result = ga4ghphetools::etl::get_cohort_data_from_etl_dto(hpo, etl);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err, "No title found")
}


#[rstest]
fn test_malformed_title (
    etl_dto_valid: EtlDto,
    hpo: Arc<FullCsrOntology>
) {
    let mut etl = etl_dto_valid.clone();
    etl.title = Some("a".to_string());
    let result = ga4ghphetools::etl::get_cohort_data_from_etl_dto(hpo, etl);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err, "Malformed title: 'a'")
}

/** Tests whether we set a redundant entry to "na" */
#[rstest]
fn test_column_type_with_redundancy(
    etl_dto_with_redudancy: EtlDto,
    hpo: Arc<FullCsrOntology>
) {
    let result = ga4ghphetools::etl::get_cohort_data_from_etl_dto(hpo.clone(), etl_dto_with_redudancy);
    assert!(result.is_ok());
    let cohort_data = result.unwrap();
    let result = ga4ghphetools::factory::qc_assessment(hpo.clone(), &cohort_data);
    assert!(result.is_err()); // expect an error because a term and its ancestor are both obeserved
    let result2 = ga4ghphetools::factory::sanitize_cohort_data(hpo.clone(),  &cohort_data);
    assert!(result2.is_ok());
    let sanitized = result2.unwrap();
    let result3 = ga4ghphetools::factory::qc_assessment(hpo.clone(), &sanitized);
    assert!(result3.is_ok());
}


#[rstest]
fn test_with_excluded_redundancy(
    patient_id_column_valid: ColumnDto,
    column_ptosis: ColumnDto,
    column_strabismus: ColumnDto,
    disease_valid: DiseaseData,
    exclude_abn_eye: ColumnDto,
    hpo: Arc<FullCsrOntology>
) {
    let  columns = vec![patient_id_column_valid, column_ptosis, column_strabismus, exclude_abn_eye];
    let table = make_table(columns);
    let etl = make_etl(table, disease_valid);
    let cohort = ga4ghphetools::etl::get_cohort_data_from_etl_dto(hpo.clone(), etl).unwrap();
    let result = ga4ghphetools::factory::qc_assessment(hpo.clone(), &cohort);
    assert!(result.is_err());
    let result2 = ga4ghphetools::factory::sanitize_cohort_data(hpo.clone(),  &cohort);
    assert!(result2.is_ok());
    let sanitized = result2.unwrap();
    let result3 = ga4ghphetools::factory::qc_assessment(hpo.clone(), &sanitized);
    assert!(result3.is_ok());

}


/// Sanity check that the serde conversion is working for Ultra-low vision with retained light perception (last entry)
#[rstest]
fn test_conversion() {
    let cell_contents = "[{\"termDuplet\":{\"hpoLabel\":\"Nystagmus\",\"hpoId\":\"HP:0000639\"},\"entry\":{\"type\":\"OnsetAge\",\"data\":\"Neonatal onset\"}},{\"termDuplet\":{\"hpoLabel\":\"Secondary microcephaly\",\"hpoId\":\"HP:0005484\"},\"entry\":{\"type\":\"OnsetAge\",\"data\":\"P9M\"}},{\"termDuplet\":{\"hpoLabel\":\"Hypertonia\",\"hpoId\":\"HP:0001276\"},\"entry\":{\"type\":\"OnsetAge\",\"data\":\"P9M\"}},{\"termDuplet\":{\"hpoLabel\":\"Global developmental delay\",\"hpoId\":\"HP:0001263\"},\"entry\":{\"type\":\"OnsetAge\",\"data\":\"P9M\"}},{\"termDuplet\":{\"hpoLabel\":\"Esotropia\",\"hpoId\":\"HP:0000565\"},\"entry\":{\"type\":\"Observed\"}},{\"termDuplet\":{\"hpoLabel\":\"Hypermetropia\",\"hpoId\":\"HP:0000540\"},\"entry\":{\"type\":\"Observed\"}},{\"termDuplet\":{\"hpoLabel\":\"Ataxia\",\"hpoId\":\"HP:0001251\"},\"entry\":{\"type\":\"OnsetAge\",\"data\":\"P2Y\"}},{\"termDuplet\":{\"hpoLabel\":\"Cerebellar atrophy\",\"hpoId\":\"HP:0001272\"},\"entry\":{\"type\":\"OnsetAge\",\"data\":\"P2Y\"}},{\"termDuplet\":{\"hpoLabel\":\"Febrile seizure (within the age range of 3 months to 6 years)\",\"hpoId\":\"HP:0002373\"},\"entry\":{\"type\":\"OnsetAge\",\"data\":\"P9M\"}},{\"termDuplet\":{\"hpoLabel\":\"Slow pupillary light response\",\"hpoId\":\"HP:0030211\"},\"entry\":{\"type\":\"Observed\"}},{\"termDuplet\":{\"hpoLabel\":\"Ultra-low vision with retained light perception\",\"hpoId\":\"HP:0032286\"},\"entry\":{\"type\":\"Observed\"}}]";
    let result = serde_json::from_str::<Vec<HpoTermData>>(cell_contents)
            .map_err(|e| e.to_string());
    assert!(result.is_ok());
    let hpo_term_data_list = result.unwrap();
    assert_eq!(11, hpo_term_data_list.len());
    let last = hpo_term_data_list.last().unwrap();
    assert_eq!(
        last.term_duplet.hpo_label,
        "Ultra-low vision with retained light perception"
    );
    assert_eq!(
        last.term_duplet.hpo_id,
        "HP:0032286"
    );
    assert_eq!(
        last.entry, CellValue::Observed
    )
}


