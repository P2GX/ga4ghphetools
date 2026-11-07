//! PpktExporter -- One row together with Header information, all that is needed to export a GA4GH phenopacket
//!
//! Each table cell is modelled as having the ability to return a datatype and the contents as a String
//! If a PpktExporter instance has no error, then we are ready to create a phenopacket.

use std::collections::HashMap;

use std::str::FromStr;
use std::sync::Arc;
use ontolius::TermId;
use crate::dto::hpo_term_dto::CellValue;
use crate::dto::hpo_term_dto::CellValueInner;
use crate::dto::hpo_term_dto::HpoTermData;
use crate::dto::cohort_data::{DiseaseData, GeneVariantData, IndividualData};

use crate::cohort_qc::field_checks;

use crate::error::ontology_error::OntologyError;
use crate::factory::header_duplet_row::HeaderDupletRow;


/// The index where the Mendelian demographic part starts
const DEMOGRAPHIC_IDX:usize = 12;


#[derive(Clone, Debug)]
pub struct PpktRow {
    header: Arc<HeaderDupletRow>,
    individual_data: IndividualData,
    disease_list: Vec<DiseaseData>,
    gene_var_list: Vec<GeneVariantData>,
    hpo_content: Vec<String>
}



impl PpktRow {

    /// Create a new PpktRow. This is used when we create a row (phenopacket) with terms that
    /// may not be included in the previous phenopackets and which may not have values for all of the
    /// terms in the previous phenopackets. 
    ///  # Arguments
    ///
    /// * `header` - Header with all HPO terms in previous cohort and new phenopacket, ordered by DFS
    /// * `individual_dto` - DTO with demographic information about the new individual
    /// * `gene_variant_list` - genotypes
    /// * `tid_to_value_map` - this has values (e.g., observed, na, P32Y2M) for which we have information in the new phenopacket
    /// * `disease_data`- DTO for cohort disease data
    pub fn from_dtos(
        header: Arc<HeaderDupletRow>, 
        individual_dto: IndividualData,
        gene_variant_list: Vec<GeneVariantData>,
        tid_to_value_map: HashMap<TermId, String>, 
        disease_data: DiseaseData) -> std::result::Result<Self, String> {
        let mut items = Vec::with_capacity(header.hpo_count());
        for hduplet in header.hpo_duplets() {
            let tid = hduplet.to_term_id().map_err(|e|e.to_string())?;
            let value: String =  tid_to_value_map.get(&tid).map_or("na", |v| v).to_string();
            items.push(value);
        }
        Ok(Self { header, 
            individual_data: individual_dto, 
            disease_list: vec![disease_data], 
            gene_var_list: gene_variant_list, 
            hpo_content: items
        })
    }


    pub fn get_individual_dto(&self) -> IndividualData {
        self.individual_data.clone()
    }

    /// Get a list of disease identifiers, e.g., OMIM:157000
    /// For Mendelian and digenic diseases, we will have but one id
    /// For melded, we will have two or more
    pub fn get_disease_id_list(&self) -> Vec<String> {
        let mut dto_list: Vec<String> = Vec::new();
        for disease in &self.disease_list {
            dto_list.push(disease.disease_id.clone())
        }
        dto_list
    }

    pub fn get_gene_var_dto_list(&self) -> Vec<GeneVariantData> {
        self.gene_var_list.clone()
    }

    pub fn get_hpo_value_list(&self) -> Result<Vec<CellValue>, String> {
        let mut cell_dto_list: Vec<CellValue> = Vec::new();
        for hpo_val in &self.hpo_content {
            cell_dto_list.push(CellValue::from_str(hpo_val).map_err(|e|e.to_string())?);
        }
        Ok(cell_dto_list)
    }

    pub fn hpo_count(&self) -> usize {
        self.hpo_content.len()
    }

    pub fn get_hpo_term_dto_list(&self) -> std::result::Result<Vec<HpoTermData>, String> {
        self.header.get_hpo_term_dto_list(&self.hpo_content).map_err(|e| e.to_string())
    }

    /// This function checks the current PpktRow for syntactical errors
    pub fn check_for_errors(&self) -> std::result::Result<(), String> {
        field_checks::check_individual_data(&self.individual_data)?;
        for db in &self.disease_list {
            field_checks::check_disease_id(&db.disease_id)?;
            field_checks::check_disease_label(&db.disease_label)?;
        }
        for gvb in &self.gene_var_list {
            field_checks::check_gene_variant_data(gvb)?;
        }
        for item in &self.hpo_content {
            if ! CellValueInner::is_valid_cell_value(item){
                return Err(format!("Invalid HPO cell contents '{}'", item));
            }
        }
        Ok(())
    }

    /// Update current HPO values according to a new header.
    /// The new header may contain HPO terms that the current PpktRow does not have
    /// in this case, we must add 'na' as the value for these terms.
    pub fn update_header(
        &self, 
        updated_hdr: Arc<HeaderDupletRow>
    ) -> std::result::Result<Self, OntologyError> {
        let updated_hpo_id_list = updated_hdr.get_hpo_id_list()?;
        let previous_header = &self.header;
        let hpo_map = previous_header.get_hpo_content_map(&self.hpo_content)?;
        let mut content = Vec::new();
        for tid in updated_hpo_id_list {
            let item: String = hpo_map
                .get(&tid)
                .cloned() // converts Option<&String> to Option<String>
                .unwrap_or_else(|| "na".to_string()); // this will pertain to the new HPO term we are adding
            content.push(item);
        }

        Ok(Self { 
            header: updated_hdr, 
            individual_data: self.individual_data.clone(), 
            disease_list: self.disease_list.clone(), 
            gene_var_list: self.gene_var_list.clone(), 
            hpo_content: content 
        })
    }

    /// Given a previous list of `TermId`s and an updated list, this function
    /// returns a vector of indices representing where each element of the
    /// `previous_hpo_list` now appears in the `updated_hpo_list`.
    ///
    /// This is useful for tracking how terms from an earlier template are
    /// rearranged after updating the template (e.g., after inserting or reordering terms).
    /// The returned vector can be used to remap associated data (e.g., column values)
    /// to their new positions.
    ///
    /// # Arguments
    /// - `previous_hpo_list`: The list of HPO term IDs before the update.
    /// - `updated_hpo_list`: The reordered or expanded list of HPO term IDs after the update.
    ///                       It must contain all terms from `previous_hpo_list`.
    ///
    /// # Returns
    /// A `Vec<usize>` where each element `i` gives the index in `updated_hpo_list`
    /// of the `i`-th term in `previous_hpo_list`.
    ///
    /// # Panics
    /// This function will panic if any term from `previous_hpo_list` is not found in `updated_hpo_list`.
    ///
    pub fn get_update_vector(
        previous_hpo_list: &[TermId],
        updated_hpo_list: &[TermId])
    -> Vec<usize> {
        let id_to_new_index: HashMap<TermId, usize> = updated_hpo_list
            .iter()
            .enumerate()
            .map(|(i, tid)| (tid.clone(), i))
            .collect();
        let new_indices: Vec<usize> = previous_hpo_list
            .iter()
            .map(|tid| id_to_new_index[tid])
            .collect();
        new_indices
    }

   
}



#[cfg(test)]
mod test {
    use super::*;
    use rstest::fixture;



    #[fixture]
    pub fn hpo_dtos() -> Vec<HpoTermData> {
        vec![HpoTermData::from_str("HP:0001382", "Joint hypermobility", "observed").unwrap(),
        HpoTermData::from_str("HP:0000574", "Thick eyebrow", "observed").unwrap()]
    }



}
