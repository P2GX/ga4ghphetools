
use rstest::fixture;
use ontolius::{io::OntologyLoaderBuilder, ontology::csr::FullCsrOntology};
use std::{fs::File, io::BufReader, sync::Arc};
use flate2::bufread::GzDecoder;

#[fixture]
pub fn hpo() -> Arc<FullCsrOntology> {
    let path = "resources/hp.v2025-03-03.json.gz";
    let reader = GzDecoder::new(BufReader::new(File::open(path).unwrap()));
    let loader = OntologyLoaderBuilder::new().obographs_parser().build();
    let hpo = loader.load_from_read(reader).unwrap();
    Arc::new(hpo)
}

/// A matrix representing a PheTools template for OMIM:617865
#[fixture]
pub fn matrix() -> Vec<Vec<String>> {
    let row1: Vec<String> = vec![ 
        "PMID", "title", "individual_id", "comment", "disease_id", "disease_label", "HGNC_id", "gene_symbol", "transcript", "allele_1", "allele_2", "variant.comment", "age_of_onset", "age_at_last_encounter", "deceased", "sex", "HPO", "Failure to thrive", "Tongue thrusting", "Ataxia", "Hypertonia", "Loss of ambulation", "Happy demeanor", "Seizure"
    ].into_iter().map(|s| s.to_owned()).collect();
    let row2: Vec<String> = vec![
        "CURIE", "str", "str", "optional", "CURIE", "str", "CURIE", "str", "str", "str", "str", "optional", "age", "age", "yes/no/na", "M:F:O:U", "na", "HP:0001508", "HP:0100703", "HP:0001251", "HP:0001276", "HP:0002505", "HP:0040082", "HP:0001250" 
    ].into_iter().map(|s| s.to_owned()).collect();
    let row3: Vec<String> = vec![
        "PMID:29198722", "A Recurrent De Novo Nonsense Variant in ZSWIM6 Results in Severe Intellectual Disability without Frontonasal or Limb Malformations", "p.Arg913Ter Affected Individual 1", "", "OMIM:617865", "Neurodevelopmental disorder with movement abnormalities, abnormal gait, and autistic features", "HGNC:29316", "ZSWIM6", "NM_020928.2", "c.2737C>T", "na", "", "Infantile onset", "P16Y", "", "M", "na", "observed", "observed", "excluded", "observed", "observed", "observed", "observed"
    ].into_iter().map(|s| s.to_owned()).collect();
    let row4: Vec<String> = vec![
        "PMID:29198722", "A Recurrent De Novo Nonsense Variant in ZSWIM6 Results in Severe Intellectual Disability without Frontonasal or Limb Malformations", "p.Arg913Ter Affected Individual 2", "", "OMIM:617865", "Neurodevelopmental disorder with movement abnormalities, abnormal gait, and autistic features", "HGNC:29316", "ZSWIM6", "NM_020928.2", "c.2737C>T", "na", "", "Infantile onset", "P7Y", "", "F", "na", "excluded", "observed", "observed", "excluded", "excluded", "observed", "excluded"
    ].into_iter().map(|s| s.to_owned()).collect();
    let row5: Vec<String> = vec![
        "PMID:29198722", "A Recurrent De Novo Nonsense Variant in ZSWIM6 Results in Severe Intellectual Disability without Frontonasal or Limb Malformations", "p.Arg913Ter Affected Individual 3", "", "OMIM:617865", "Neurodevelopmental disorder with movement abnormalities, abnormal gait, and autistic features", "HGNC:29316", "ZSWIM6", "NM_020928.2", "c.2737C>T", "na", "", "Infantile onset", "P4Y", "", "F", "na", "excluded", "observed", "excluded", "observed", "excluded", "observed", "na"
    ].into_iter().map(|s| s.to_owned()).collect();
    let row6: Vec<String> = vec![
        "PMID:29198722", "A Recurrent De Novo Nonsense Variant in ZSWIM6 Results in Severe Intellectual Disability without Frontonasal or Limb Malformations", "p.Arg913Ter Affected Individual 4", "", "OMIM:617865", "Neurodevelopmental disorder with movement abnormalities, abnormal gait, and autistic features", "HGNC:29316", "ZSWIM6", "NM_020928.2", "c.2737C>T", "na", "", "Infantile onset", "P5Y", "", "F", "na", "excluded", "excluded", "observed", "excluded", "excluded", "na", "excluded"
    ].into_iter().map(|s| s.to_owned()).collect();
    vec![row1, row2, row3, row4, row5, row6]
}

