//! Export Cohort data to HTML and other formats for further viewing or analysis 


use std::{path::Path, sync::Arc};

use ontolius::ontology::csr::FullCsrOntology;
use tera::Context;

use crate::{dto::cohort_dto::CohortData, export::{cohort_renderer::CohortRenderer, html_renderer::HtmlRenderer}};


mod cohort_renderer;
mod html_renderer;


/// Render a cohort report as an HTML file.
///
/// This function takes a fully populated [`CohortData`] instance together with a
/// reference to the [`FullCsrOntology`] and produces a complete, static HTML
/// representation of the cohort. The HTML report can then be opened in a browser
/// or embedded within a Tauri WebView.
///
/// # Arguments
///
/// * `cohort` — The cohort data structure containing phenotypic and variant information.
/// * `hpo` — A shared [`FullCsrOntology`] instance providing HPO term metadata
///   used for rendering headers and labels.
/// * `output_path` — The filesystem path where the generated HTML file will be written.
///
/// # Returns
///
/// * `Ok(())` on success.
/// * `Err(String)` if rendering fails or if writing the HTML file to disk fails.
///
/// # Notes
///
/// - The output is a standalone HTML document; it does not require external assets.
/// - This can be called directly from a Tauri command to generate reports within the GUI.
/// - The [`HtmlRenderer`] uses a preconfigured template engine (e.g., Tera or Handlebars)
///   and expects a compatible template to be embedded or available at runtime.
///
/// # Errors
///
/// Returns an error if:
/// - Template rendering fails (e.g., missing or invalid template).
/// - Writing the output file fails (e.g., permission or I/O error).
pub fn render_html(
    cohort: CohortData,
    hpo: Arc<FullCsrOntology>, 
    output_path: &Path) -> Result<(), String> {
    let cohort_renderer = CohortRenderer::new(&cohort, hpo.clone())?;
    let mut context = Context::new();
    context.insert("cohort", &cohort_renderer);
    let renderer = HtmlRenderer::new();
    let html = renderer.render_html(context);
    std::fs::write(output_path, html).map_err(|e|e.to_string())?;
    Ok(())
}