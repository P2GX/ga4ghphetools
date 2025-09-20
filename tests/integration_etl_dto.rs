
use std::collections::HashMap;

use ga4ghphetools::dto::cohort_dto::GeneTranscriptData;
use ga4ghphetools::dto::etl_dto::ColumnDto;
use ga4ghphetools::dto::etl_dto::ColumnTableDto;
use ga4ghphetools::dto::etl_dto::EtlColumnHeader;
use ga4ghphetools::dto::etl_dto::EtlColumnType;
use ga4ghphetools::dto::etl_dto::EtlDto;
use ga4ghphetools::dto::hgvs_variant::HgvsVariant;
use ga4ghphetools::dto::hpo_term_dto::HpoTermDuplet;
use ga4ghphetools::dto::cohort_dto::DiseaseData;
use rstest::fixture;
use rstest::rstest;
use serde_json::json;
use serde_json::to_string;
use serde_json::Value;





#[fixture]
fn column_1_valid() -> ColumnDto {
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
fn column_2_valid() -> ColumnDto {
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
fn column_3_valid() -> ColumnDto {
    ColumnDto {
        id: "09e8235b-d9dd-47e0-9831-68981a24fa93".to_string(),
        transformed: true,
        header: EtlColumnHeader{
            original: "Age at evaluation".to_string(),
            current: None,
            column_type: EtlColumnType::Raw,
            hpo_terms: None,
        },
        values: vec![
           "P3Y6M".to_string(),
           "P25Y".to_string()
        ]
    }
}

#[fixture]
fn column_4_valid() -> ColumnDto {
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
fn column_5_valid() -> ColumnDto {
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
fn column_6_valid() -> ColumnDto {
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
fn column_hypo_hyper() -> ColumnDto {
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
fn valid_columns(
    column_1_valid: ColumnDto,
    column_2_valid: ColumnDto,
    column_3_valid: ColumnDto,
    column_4_valid: ColumnDto,
    column_5_valid: ColumnDto,
    column_6_valid: ColumnDto,
    column_hypo_hyper: ColumnDto,
    column_ptosis: ColumnDto,
    column_strabismus: ColumnDto
) -> Vec<ColumnDto> {
    vec![column_1_valid, column_2_valid, column_3_valid, column_4_valid, column_5_valid, column_6_valid,
        column_hypo_hyper, column_ptosis, column_strabismus]
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


/* 
      "symbol": "WDR83OS",
      "hgncId": "HGNC:30203",
      "hgvs": "",
      "transcript": "NM_016145.4",
      "gHgvs":,
      "variantKey": "c156_1GtoT_WDR83OS_NM_016145v4"
    }
  },*/

#[fixture]
fn etl_dto_valid(
    valid_columns: Vec<ColumnDto>,
    disease_valid: DiseaseData,
    hgvs_map: HashMap<String, HgvsVariant>
) -> EtlDto {
    let table: ColumnTableDto = ColumnTableDto{ 
        file_name: "/Users/name/Desktop/stuff/Demo.xlsx".to_string(), 
        columns: valid_columns
    };
    EtlDto { 
        table: table, 
        disease: Some(disease_valid), 
        pmid: Some("PMID:39471804".to_string()), 
        title: Some("Homozygous variants in WDR83OS lead to a neurodevelopmental disorder with hypercholanemia.".to_string()), 
        hgvs_variants: hgvs_map, 
        structural_variants: HashMap::new() 
    }
}


#[rstest]
fn test_variant_key(
    hgvs_var_1_valid: HgvsVariant
) {
    let expected_key = "c235CtoT_WDR83OS_NM_016145v4";
    assert_eq!(expected_key, hgvs_var_1_valid.variant_key(), "Mismatched variant key");
}

