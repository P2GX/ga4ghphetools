use ontolius::{io::OntologyLoaderBuilder, ontology::csr::FullCsrOntology};
use rstest::fixture;
use std::{fs::File, io::BufReader, sync::Arc};
use flate2::bufread::GzDecoder;



use once_cell::sync::Lazy;


// Singleton - loads once, shared across all tests
pub static HPO: Lazy<Arc<FullCsrOntology>> = Lazy::new(|| {
    let path = "resources/hp.v2025-03-03.json.gz";
    let reader = GzDecoder::new(BufReader::new(File::open(path).unwrap()));
    let loader = OntologyLoaderBuilder::new().obographs_parser().build();
    let hpo = loader.load_from_read(reader).unwrap();
    Arc::new(hpo)
});

   
#[rstest::fixture]
pub fn hpo() -> Arc<FullCsrOntology> {
    Arc::clone(&HPO)
}




