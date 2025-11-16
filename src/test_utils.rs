/// Fixtures needed for multiple tests
/// We create a singleton HPO to prevent each test module from loading it anew

#[cfg(test)]
pub mod fixtures {
    use once_cell::sync::Lazy;
    use ontolius::io::OntologyLoaderBuilder;
    use ontolius::ontology::csr::FullCsrOntology;
    use std::sync::Arc;
    use std::fs::File;
    use std::io::BufReader;
    use flate2::read::GzDecoder;
    // ... other imports

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
}