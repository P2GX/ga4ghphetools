//! Export Cohort data to HTML and other formats for further viewing or analysis 


use std::{path::Path, sync::Arc};

use ontolius::ontology::csr::FullCsrOntology;
use tera::Context;

use crate::{dto::cohort_dto::CohortData, export::{cohort_renderer::CohortRenderer, html_renderer::HtmlRenderer, table_compare::TableCompare}, factory};


mod cohort_renderer;
mod html_renderer;
mod table_compare;
#[cfg(feature = "excel_export")]
mod excel_export;


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


/// Generate a tab-separated comparison table summarizing HPO term frequencies between two cohorts.
///
/// This function compares two cohorts annotated with HPO terms and produces
/// a table summarizing how often each term was observed or excluded in each cohort.
/// The table is organized by top-level HPO categories (e.g. “Abnormality of the cardiovascular system”),
/// and only includes terms that have been measured at least `threshold` times in total.
///
/// # Arguments
///
/// * `cohort_1_path` - Path to the first cohort file (JSON format).
/// * `cohort_2_path` - Path to the second cohort file (JSON format).
/// * `output_path` - Path to write the output table (tab-separated text file).
/// * `hpo` - An [`Arc<FullCsrOntology>`] containing the HPO ontology used to determine term hierarchy.
/// * `threshold` - Minimum total number of measurements (across both cohorts)
///                 required for a term to be included in the output.
///
/// # Output Format
///
/// The output is a tab-separated table with the following columns:
///
/// | HPO | HPO.id | Cohort 1 | Cohort 2 | Total |
/// |------|---------|-----------|-----------|--------|
/// | *term label* | *term ID* | `obs/measured (percent)` | `obs/measured (percent)` | `obs/measured (percent)` |
///
/// The table is grouped by top-level HPO categories. Each group starts with a subheader row
/// identifying the organ system or top-level category.
///
/// # Returns
///
/// Returns `Ok(())` if the table was successfully written to `output_path`,
/// or an `Err(String)` describing the error (for example, if one of the input files
/// could not be loaded or an HPO term lookup failed).
///
/// # Errors
///
/// * If either cohort JSON file cannot be read or parsed.
/// * If any HPO terms in the cohorts cannot be resolved in the provided ontology.
/// * If the output file cannot be created or written.
///
/// # Notes
///
/// This function internally:
/// - Loads both cohorts from JSON using [`factory::load_json_cohort`].
/// - Uses [`TableCompare`] to aggregate and compare per-term counts.
/// - Writes only rows where the total number of measurements (`observed + excluded`)
///   across both cohorts meets or exceeds `threshold`.
///
/// See also: [`TableCompare`], [`TermCounter`], [`RowCounter`].
pub fn output_comparison_table(
    cohort_1_path: &str,
    cohort_2_path: &str,
    output_path: &str,
    hpo: Arc<FullCsrOntology>,
    threshold: usize) -> Result<(), String> {
        let cohort_1 = factory::load_json_cohort(cohort_1_path)?;
        let cohort_2 = factory::load_json_cohort(cohort_2_path)?;
        let table_compare = TableCompare::new(cohort_1, cohort_2, hpo)?;
        table_compare.output_table(output_path, threshold)?;
        Ok(())
    }



/// Export a comparison of two cohorts to an Excel file.
///
/// This function loads two cohorts from JSON files, compares them using the
/// provided HPO ontology, and writes the results to an Excel workbook.
/// The output may include merged cells and threshold-based filtering,
/// depending on the implementation of the Excel exporter.
///
/// # Feature flags
///
/// This function is only available when the `excel_export` feature is enabled.
///
/// # Parameters
///
/// * `cohort_1_path` - Path to the first cohort JSON file.
/// * `cohort_2_path` - Path to the second cohort JSON file.
/// * `output_path` - Path where the Excel file will be written.
/// * `hpo` - Shared reference to the full HPO ontology used for comparison.
/// * `threshold` - Minimum count or cutoff used to filter reported terms.
///
/// # Errors
///
/// Returns an error if:
/// * Either cohort cannot be loaded from disk
/// * The Excel file cannot be created or written
///
/// # Examples
///
/// ```ignore
/// # use std::sync::Arc;
/// # use your_crate::output_excel_comparison;
/// # use your_crate::FullCsrOntology;
/// # let hpo = Arc::new(FullCsrOntology::default());
/// output_excel_comparison(
///     "cohort_a.json",
///     "cohort_b.json",
///     "comparison.xlsx",
///     hpo,
///     5,
/// )?;
/// # Ok::<(), String>(())
/// ```
///
/// # See also
///
/// * [`factory::load_json_cohort`]
/// * [`excel_export::create_excel_with_merged_cells`]
#[cfg(feature = "excel_export")]
pub fn output_excel_comparison(
    cohort_1_path: &str,
    cohort_2_path: &str,
    output_path: &str,
    hpo: Arc<FullCsrOntology>,
    threshold: usize
)-> Result<(), String> {
        let cohort_1 = factory::load_json_cohort(cohort_1_path)?;
        let cohort_2 = factory::load_json_cohort(cohort_2_path)?;
        excel_export::create_excel_with_merged_cells(output_path, cohort_1, cohort_2, hpo, threshold)?;
        Ok(())
}


#[cfg(test)]
mod tests {
    use std::{fs::File, io::BufReader, sync::Arc};
    use flate2::bufread::GzDecoder;
    use ontolius::{io::OntologyLoaderBuilder, ontology::csr::FullCsrOntology};
    use rstest::{fixture, rstest};
    use super::*;


     #[fixture]
    fn hpo() -> Arc<FullCsrOntology> {
        let path = "resources/hp.v2025-03-03.json.gz";
        let reader = GzDecoder::new(BufReader::new(File::open(path).unwrap()));
        let loader = OntologyLoaderBuilder::new().obographs_parser().build();
        let hpo = loader.load_from_read(reader).unwrap();
        Arc::new(hpo)
    }

    #[rstest]
    #[ignore = "local file, just for testing"]
    fn write_compare(hpo: Arc<FullCsrOntology>) {
        let cohort_1 = "/Users/robin/Desktop/HPOstuff/Netherton/NL-cohort/SPINK5_NETH_individuals-NL.json";
        let cohort_2 = "/Users/robin/GIT/phenopacket-store/notebooks/SPINK5/SPINK5_NETH_individuals.json";
        let output_path = "/Users/robin/Desktop/HPOstuff/Netherton/NL-cohort/comparison.txt";
        let threshold = 20;
        output_comparison_table(cohort_1, cohort_2, output_path, hpo, threshold).unwrap();
    }


    #[cfg(feature = "excel_export")]
    #[rstest]
    //#[ignore = "local file, just for testing"]
    fn write_compare_excel(hpo: Arc<FullCsrOntology>) {
        let cohort_1 = "/Users/robin/Desktop/HPOstuff/Netherton/NL-cohort/SPINK5_NETH_individuals-DEC19.json";
        let cohort_2 = "/Users/robin/GIT/phenopacket-store/notebooks/SPINK5/SPINK5_NETH_individuals.json";
        let output_path = "/Users/robin/Desktop/HPOstuff/Netherton/NL-cohort/comparisonDEC19.xlsx";
        let threshold = 20;
        output_excel_comparison(cohort_1, cohort_2, output_path, hpo, threshold).unwrap();
    }

}

