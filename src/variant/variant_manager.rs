
use std::collections::HashMap;
use crate::dto::template_dto::CohortDto;

use crate::dto::validation_errors::ValidationErrors;
use crate::dto::variant_dto::{VariantValidationDto, VariantValidationType};
use crate::variant::hgvs_variant::HgvsVariant;
use crate::variant::structural_validator::StructuralValidator;
use crate::{dto::variant_dto::VariantDto, variant::hgvs_variant_validator::HgvsVariantValidator};


use crate::variant::structural_variant::StructuralVariant;

type VariantCache = HashMap<String, HgvsVariant>;
type StructuralCache = HashMap<String, StructuralVariant>;

/// This struct validates variants sent from the front end. It evaluates either HGVS or symbolic (imprecise)
/// structural variants. If a variant is validated, then the CohortTemplate is sent back to the front end
/// with the new variant structure, otherwise errors are returned
pub struct VariantManager {
    hgvs_validator: HgvsVariantValidator,
    structural_validator: StructuralValidator
}




impl VariantManager {
    pub fn new() -> Self {
        Self {
            hgvs_validator: HgvsVariantValidator::hg38(),
            structural_validator: StructuralValidator::hg38()
        }
    }

    pub fn validate_variant(
        &self, 
        vv_dto: VariantValidationDto, 
        mut cohort_dto: CohortDto)
    -> Result<CohortDto, String> {
        match &vv_dto.validation_type {
            VariantValidationType::Hgvs => {
                let hgvs = self.hgvs_validator.validate_hgvs(vv_dto)?;
                cohort_dto.validated_hgvs_variants.insert(hgvs.variant_key(), hgvs);
                return Ok(cohort_dto);
            } 
            VariantValidationType::PreciseSv => {
                return Err("Precise SV validation not implemented".to_string())
            }
            VariantValidationType::Del 
            | VariantValidationType::Inv 
            | VariantValidationType::Transl 
            | VariantValidationType::Dup
            | VariantValidationType::Ins 
            | VariantValidationType::Sv => {
                let sv = self.structural_validator.validate_sv(vv_dto)?;
                cohort_dto.validated_structural_variants.insert(sv.variant_key(), sv);
                return Ok(cohort_dto);
            }
        }
    }

    


   

    /// Extract a list of the variant DTOs sorted such that the HGVS variants come first and are sorted
    /// by gene symbol and then alphanumerbetically by HGVS nomenclature
    /*
    pub fn sorted_variant_dtos(&self) -> Vec<VariantDto> {
        let mut variant_list: Vec<VariantDto> = self.hgvs_validator.values().cloned().collect();
        variant_list.sort_by(|a, b| {
            (
                a.is_structural(), // false < true
                a.gene_symbol(),
                a.numerical_key(),
                a.variant_string(),
            )
            .cmp(&(
                b.is_structural(),
                b.gene_symbol(),
                b.numerical_key(),
                b.variant_string(),
            ))
        });
        variant_list
    }
 */




    pub fn validate_variant_dto_list(&mut self, variant_dto_list: Vec<VariantDto>) -> Result<Vec<VariantDto>, String> {
       /*  let mut evaluated_dto_list: Vec<VariantDto> = Vec::with_capacity(variant_dto_list.len());
        for dto in variant_dto_list {
            let variant = dto.variant_string();
            if dto.is_structural() {
                if self.structural_cache.contains_key(variant ) {
                    evaluated_dto_list.push(dto.clone_validated());
                } else {
                    match self.structural_validator.validate_sv(&dto) {
                        Ok(sv) => {
                            self.structural_cache.insert(variant.to_string(), sv);
                            evaluated_dto_list.push(dto.clone_validated());
                        },
                        Err(e) => {
                            evaluated_dto_list.push(dto.clone_unvalidated());
                        },
                    }
                }
            } else if self.hgvs_cache.contains_key(variant) {
                evaluated_dto_list.push(dto.clone_validated());
            } else {
                match self.validator.validate_hgvs(&dto) {
                    Ok(hgvs) => {
                        self.hgvs_cache.insert(variant.to_string(), hgvs);
                        evaluated_dto_list.push(dto.clone_validated());
                    },
                    Err(e) => {
                        evaluated_dto_list.push(dto.clone_unvalidated());
                    },
                }
            }
        }
        // write variants to cache.
        self.save_hgvs()?;
        self.save_structural()?; 
        VariantDto::sort_variant_dtos(&mut evaluated_dto_list);
        Ok(evaluated_dto_list)*/
        Err("refactoring".to_ascii_lowercase())
    }





}