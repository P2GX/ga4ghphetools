

use ga4ghphetools::dto::hpo_term_dto::HpoTermData;
use rstest::fixture;


#[fixture]
pub fn thick_eye_brow_excluded_dto() -> HpoTermData {
    HpoTermData::from_str("HP:0000574", "Thick eyebrow", "excluded").unwrap()
}

#[fixture]
pub fn thick_eye_brow_observed_dto() -> HpoTermData {
    HpoTermData::from_str("HP:0000574", "Thick eyebrow", "observed").unwrap()
}

#[fixture]
pub fn thick_eye_brow_na_dto() -> HpoTermData {
    HpoTermData::from_str("HP:0000574", "Thick eyebrow", "na").unwrap()
}


#[fixture]
pub fn flat_occiput_observed_dto() -> HpoTermData {
    HpoTermData::from_str("HP:0005469", "Flat occiput", "observed").unwrap()
}
#[fixture]
pub fn flat_occiput_excluded_dto() -> HpoTermData {
    HpoTermData::from_str("HP:0005469", "Flat occiput", "excluded").unwrap()
}
#[fixture]
pub fn flat_occiput_na_dto() -> HpoTermData {
    HpoTermData::from_str("HP:0005469", "Flat occiput", "na").unwrap()
}
#[fixture]
pub fn join_hypermobility_observed_dto() -> HpoTermData {
    HpoTermData::from_str("HP:0001382", "Joint hypermobility", "observed").unwrap()
}

#[fixture]
pub fn joint_hypermobility_excluded_dto() -> HpoTermData {
    HpoTermData::from_str("HP:0001382", "Joint hypermobility", "excluded").unwrap()
}
#[fixture]
pub fn joint_hypermobility_na_dto() -> HpoTermData {
    HpoTermData::from_str("HP:0001382", "Joint hypermobility", "na").unwrap()
}


#[fixture]
pub fn grand_mal_observed_dto() -> HpoTermData {
    HpoTermData::from_str("HP:0002069", "Bilateral tonic-clonic seizure", "observed").unwrap()
}
#[fixture]
pub fn grand_mal_excluded_dto() -> HpoTermData {
    HpoTermData::from_str("HP:0002069", "Bilateral tonic-clonic seizure", "excluded").unwrap()
}

#[fixture]
pub fn strabismus_observed_dto() -> HpoTermData {
    HpoTermData::from_str("HP:0000486", "Strabismus", "observed").unwrap()
}

#[fixture]
pub fn esotropia_observed_dto() -> HpoTermData {
    HpoTermData::from_str("HP:0000565", "Esotropia", "observed").unwrap()
}

