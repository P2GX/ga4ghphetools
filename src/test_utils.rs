/// Fixtures needed for multiple tests
/// We create a singleton HPO to prevent each test module from loading it anew

#[cfg(test)]
pub mod fixtures {
    use std::sync::LazyLock;
    use ontolius::io::OntologyLoaderBuilder;
    use ontolius::ontology::csr::FullCsrOntology;
    use std::sync::Arc;
    use std::fs::File;
    use std::io::BufReader;
    use flate2::read::GzDecoder;
    use std::time::Duration;



    pub static HPO: LazyLock<Arc<FullCsrOntology>> = LazyLock::new(|| {
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


    #[rstest::fixture]
    pub fn http_client() -> Arc<reqwest::blocking::Client> {
        reqwest::blocking::Client::builder()
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(10))
            .build()
            .map(Arc::new)
            .expect("Failed to build HTTP client for test fixture")
    }
}