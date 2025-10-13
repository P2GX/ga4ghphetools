use std::{path::PathBuf};
use tera::{Tera, Context};

pub struct HtmlRenderer {
     tera: Tera,
}

impl HtmlRenderer {
    pub fn new() -> Self {
        // Always resolve relative to the crate directory, not the runtime cwd
        let mut pattern = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        pattern.push("templates/**/*.html");

        let pattern_str = pattern.to_string_lossy();
        let tera = Tera::new(&pattern_str)
            .unwrap_or_else(|e| panic!("Failed to initialize Tera: {e}"));

        Self{ tera }
    }

    pub fn render_html(&self, context: Context) -> String {
       
        let html = self.tera
            .render("cohort_data/cohort_report.html", &context).map_err(|e| {
            eprintln!("❌ Template render failed!");
            eprintln!("Error: {:?}", e); // detailed debug output
            e
        });
        match html {
            Ok(html) => { return html;},
            Err(e) => {
                panic!("{:?}",e);
            }
        }
    }



}


#[cfg(test)]
mod tests {
    use std::{fs::File, io::BufReader, path::Path, sync::Arc};
    use flate2::bufread::GzDecoder;
    use ontolius::{io::OntologyLoaderBuilder, ontology::csr::FullCsrOntology};
    use rstest::{fixture, rstest};
    use super::*;
    use crate::{dto::cohort_dto::CohortData, export::cohort_renderer::CohortRenderer};

    

     #[fixture]
    fn hpo() -> Arc<FullCsrOntology> {
        let path = "resources/hp.v2025-03-03.json.gz";
        let reader = GzDecoder::new(BufReader::new(File::open(path).unwrap()));
        let loader = OntologyLoaderBuilder::new().obographs_parser().build();
        let hpo = loader.load_from_read(reader).unwrap();
        Arc::new(hpo)
    }


    #[rstest]
    #[ignore = "Using local file"]
    fn test_render_html(hpo: Arc<FullCsrOntology>) {
        let template_path = "/Users/robin/GIT/phenopacket-store/notebooks/ADA/ADA_ADA-SCID_individuals.json";
        let file = File::open(template_path).unwrap();
        let reader = BufReader::new(file);
        let cohort: CohortData = serde_json::from_reader(reader).unwrap();
        let cohort_render = CohortRenderer::new(&cohort, hpo.clone()).unwrap();
        let renderer = HtmlRenderer::new();
        let output_path = Path::new("/Users/robin/Downloads/test.html");
        let mut context = Context::new();
        context.insert("cohort", &cohort_render);
        println!("{:?}", cohort_render);
        let html = renderer.render_html(context);
       // assert!(html.contains("ADA-SCID"));
        std::fs::write(output_path, html).unwrap();
        println!("✅ Rendered to {}", output_path.display());
        
    }


}