use ga4ghphetools::dto::hpo_term_dto::HpoTermData;
use ontolius::{io::OntologyLoaderBuilder, ontology::csr::FullCsrOntology};
use rstest::fixture;
use std::{fs::File, io::BufReader, sync::Arc};
use flate2::bufread::GzDecoder;
use std::sync::LazyLock;


// Singleton - loads once, shared across all tests
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





