use tauri_plugin_fs::FilePath;
use ontolius::{io::OntologyLoaderBuilder, ontology::csr::FullCsrOntology};
use std::sync::Arc;

/// Converts a generic Tauri file-picker `FilePath` into a standard `String`.
pub fn get_full_path_as_str(file_path: FilePath) -> Result<String, String> {
    let path = file_path
        .as_path()
        .ok_or_else(|| "Failed to extract system path from FilePath entry".to_string())?;

    Ok(path.to_string_lossy().to_string())
}


pub fn load_ontology(json_path: &str) -> Result<Arc<FullCsrOntology>, Box<dyn std::error::Error>> {
    let loader = OntologyLoaderBuilder::new().obographs_parser().build();
    let onto: FullCsrOntology = loader.load_from_path(json_path)?;
    Ok(Arc::new(onto))
}
