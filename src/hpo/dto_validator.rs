use std::sync::Arc;

use ontolius::ontology::csr::FullCsrOntology;

use crate::dto::{template_dto::TemplateDto, validation_errors::ValidationErrors};




pub struct TemplateDtoValidator {
    hpo: Arc<FullCsrOntology>
}

impl TemplateDtoValidator {
    pub fn new(hpo: Arc<FullCsrOntology>) -> Self {
        Self { hpo }
    }


    pub fn validate(
        &self, 
        template_dto: TemplateDto
    ) -> Result<(), ValidationErrors> {
        println!("validating");
        let mut verrs = ValidationErrors::new();
        match template_dto.cohort_type.as_str() {
            "mendelian" => self.validate_mendelian(template_dto)?,
            other => { 
                verrs.push_str(
                    format!("not implemented-validation for {other}"));
                return Err(verrs); 
            }
        }

        Ok(())
    }


    fn validate_mendelian(
        &self, 
        template_dto: TemplateDto
    )-> Result<(), ValidationErrors> { 
        todo!();
        Ok(())
    }
}