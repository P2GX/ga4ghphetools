//! Data Transfer Objects (DTOs)
//! 
//! Some of the structures in this module are used purely for structured data transfer, and there are named according to the pattern ``EntityDto``.
//! Others are used for serialization. See especially [`CohortData`](crate::dto::cohort_dto::CohortData), the main structure that represents all of the data about a cohort. 

pub mod case_dto;
pub mod cohort_dto;
pub mod etl_dto;
pub mod hgvs_variant;
pub mod hpo_term_dto;
pub mod intergenic_variant;
pub mod structural_variant;
pub mod validation_errors;
pub mod variant_dto;