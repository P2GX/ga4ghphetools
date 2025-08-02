//! PptTemplate2
//!
//! The struct that contains all data needed to create or edit a cohort of phenopackets
//! in "pyphetools" format, and to export GA4GH Phenopackets.
//! Each template has the following main members
//! - HeaderDupletRow (defines each of the columns)
//! - A list of PpktRow (one per phenopacket)
use std::{collections::{HashMap, HashSet}, str::FromStr, sync::Arc, vec};
use ontolius::{
    ontology::{csr::FullCsrOntology, MetadataAware, OntologyTerms},
    term::{simple::{SimpleMinimalTerm}, MinimalTerm},
    Identified, TermId,
};
use phenopackets::schema::v2::Phenopacket;
use serde::{Deserialize, Serialize};

use crate::{dto::{hpo_term_dto::HpoTermDto, template_dto::{DiseaseDto, DiseaseGeneDto, GeneTranscriptDto, GeneVariantBundleDto, IndividualBundleDto, RowDto, TemplateDto}, validation_errors::ValidationErrors}, error::{Error, Result}, header::hpo_term_duplet::HpoTermDuplet, hpo::hpo_util::HpoUtil, ppkt::{ppkt_exporter::PpktExporter, ppkt_row::PpktRow}, template::header_duplet_row::HeaderDupletRow, variant::{hgvs_variant::HgvsVariant, structural_variant::StructuralVariant}};
use crate::{
    hpo::hpo_term_arranger::HpoTermArranger
};


/// Phetools can be used to curate cases with Mendelian disease or with melded phenotypes
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum TemplateType {
    Mendelian,
    Melded,
    Digenic
}

impl FromStr for TemplateType {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, String> {
        match s.to_ascii_lowercase().as_str() {
            "mendelian" => Ok(TemplateType::Mendelian),
            "melded" => Ok(TemplateType::Melded),
            "digenic" => Ok(TemplateType::Digenic),
            _ => Err(format!("Unrecognized template type {s}")),
        }
    }
}

/// All data needed to edit a cohort of phenopackets or export as GA4GH Phenopackets
pub struct PheToolsTemplate {
    header: Arc<HeaderDupletRow>,
    template_type: TemplateType,
    /// Data structure used to seed new entries in the template (info re: gene[s], disease[s])
    disease_gene_dto: DiseaseGeneDto,
     /// Reference to the Ontolius Human Phenotype Ontology Full CSR object
    hpo: Arc<FullCsrOntology>,
     /// One row for each individual (phenopacket) in the cohort
    ppkt_rows: Vec<PpktRow>
}

const PMID_COL: usize = 0;
const TITLE_COL: usize = 1;
const INDIVIDUAL_ID_COL: usize = 2;
const INDIVIDUAL_COMMENT: usize = 3;
const EMPTY_STRING: &str = "";

impl PheToolsTemplate {
    /// Create the initial pyphetools template using HPO seed terms
    pub fn create_pyphetools_template_mendelian(
        hpo_term_ids: Vec<TermId>,
        // Reference to the Ontolius Human Phenotype Ontology Full CSR object
        hpo: Arc<FullCsrOntology>,
        disease_gene_dto: DiseaseGeneDto,
    ) -> std::result::Result<Self, String> {
        let mut hp_header_duplet_list: Vec<HpoTermDuplet> = Vec::new();
        for hpo_id in hpo_term_ids {
            match hpo.term_by_id(&hpo_id) {
                Some(term) => {
                    let hpo_duplet = HpoTermDuplet::new(term.name(), term.identifier().to_string());
                    hp_header_duplet_list.push(hpo_duplet);
                }
                None => {
                    return Err(format!("Could not find HPO identifier '{}'", hpo_id.to_string()));
                }
            }
        }
        let header_dup_row = HeaderDupletRow::from_hpo_duplets(hp_header_duplet_list, TemplateType::Mendelian);
        let hdr_arc = Arc::new(header_dup_row);

        Ok(Self {
            header: hdr_arc,
            template_type: TemplateType::Mendelian,
            disease_gene_dto,
            hpo: hpo.clone(),
            ppkt_rows: vec![]
        })
    }

    /**
     * Transform a DTO received from the frontend into a PheToolsTemplate object.
     */
    pub fn from_dto(
        hpo: Arc<FullCsrOntology>,
        cohort_dto: &TemplateDto) 
    -> std::result::Result<Self, String>  {
        let tt: TemplateType = cohort_dto.cohort_type;
        let hpo_duplets: Vec<HpoTermDuplet> = cohort_dto
            .hpo_headers
            .iter()
            .map(|dto| dto.clone().to_hpo_duplet())
            .collect();
        let header_duplet_row: HeaderDupletRow = HeaderDupletRow::from_hpo_duplets(hpo_duplets, tt);
        let arc_header = Arc::new(header_duplet_row);
        let ppkt_rows = cohort_dto.rows.iter()
            .map(|dto| PpktRow::from_dto(dto, arc_header.clone())).collect();
        println!("{}{}-from dto.", file!(), line!());
        Ok(Self { 
            header: arc_header.clone(), 
            template_type: tt, 
            disease_gene_dto: cohort_dto.disease_gene_dto.clone(),
            hpo: hpo.clone(), 
            ppkt_rows 
        })
    }
    

    pub fn get_template_dto(&self) -> Result<TemplateDto> {
        let header_dto = self.header.get_hpo_header_dtos();
        let row_dto_list: Vec<RowDto> = self.ppkt_rows
            .iter()
            .map(RowDto::from_ppkt_row)
            .collect();
        Ok(TemplateDto::mendelian(self.disease_gene_dto.clone(), header_dto, row_dto_list, ))
    }

    pub fn from_template_dto(
        template_dto: &TemplateDto, 
        hpo: Arc<FullCsrOntology>) 
    -> std::result::Result<Self, ValidationErrors> {
        let header_duplet_row = match template_dto.cohort_type {
            TemplateType::Mendelian => HeaderDupletRow::new_mendelian_ppkt_from_dto(&template_dto.hpo_headers),
            other => {
                return Err(ValidationErrors::from_string(format!("Only Mendelian implemented. We cannot yet handle '{:?}'", other)));
            }
        };
        let header_arc = Arc::new(header_duplet_row);
        let mut ppkt_rows: Vec<PpktRow> = Vec::new();
        for row_dto in &template_dto.rows {
            let ppkt_row = PpktRow::from_dto(row_dto, header_arc.clone());
            ppkt_rows.push(ppkt_row);
        }
        let template = PheToolsTemplate {
            header: header_arc.clone(),
            template_type: TemplateType::Mendelian,
            disease_gene_dto: template_dto.disease_gene_dto.clone(),
            hpo,
            ppkt_rows,
        };

        template.check_for_errors()?;
        Ok(template)
    }

    /// Get a list of all HPO identifiers currently in the template
    pub fn get_hpo_term_ids(&self) -> std::result::Result<Vec<TermId>, Vec<String>> {
        self.header.get_hpo_id_list().map_err(|verr|verr.errors().clone())
    }

    pub fn create_pyphetools_template(
        template_type: TemplateType,
        disease_gene_dto: DiseaseGeneDto,
        hpo_term_ids: Vec<TermId>,
        hpo: Arc<FullCsrOntology>,
    ) -> std::result::Result<PheToolsTemplate, String> {
        let mut smt_list: Vec<SimpleMinimalTerm> = Vec::new();
        for hpo_id in &hpo_term_ids {
            match hpo.term_by_id(hpo_id) {
                Some(term) => {
                    let smt =
                        SimpleMinimalTerm::new(term.identifier().clone(), term.name(), vec![], false);
                    smt_list.push(smt);
                }
                None => {
                    return Err(format!("Could not find HPO identifier '{}'", hpo_id.to_string()));
                }
            }
        }
        if template_type == TemplateType::Mendelian {
            let result = Self::create_pyphetools_template_mendelian( hpo_term_ids, hpo, disease_gene_dto)?;
            Ok(result)
        } else {
            Err(format!("Creation of template of type {:?} not supported", template_type))
        }
        
    }

    /// We are extract a DiseaseGeneDto from the Excel files (version 1), all of which are
    /// Mendelian. We know the columns are
    /// (0) "PMID", (1) "title", (2) "individual_id", (3)"comment", 
    /// (4*) "disease_id", (5*) "disease_label", (6*) "HGNC_id", (7*) "gene_symbol", 
    ///  (8*)  "transcript", (9) "allele_1", (10) "allele_2", (11) "variant.comment", 
    ///    (12) "age_of_onset", (13)"age_at_last_encounter", (14)  "deceased", (15) "sex", (16) "HPO", 
    /// The columns with asterisk are what we need
    /// Note: This function should be deleted after the Excel files have been converted.
    fn get_disease_dto_from_excel(matrix: &Vec<Vec<String>>) -> std::result::Result<DiseaseGeneDto, String> {
        let rows: Vec<&Vec<String>> = matrix.iter().skip(2).collect();
        if rows.is_empty() {
            return Err("Could not extract DTO because less than three rows found".to_string());
        };
        let mut extracted_data: Vec<(String, String, String, String, String)> = Vec::new();
        for (row_idx, row) in rows.iter().enumerate() {
            if row.len() <= 16 {
                return Err(format!("Row {} (after skipping 2) has only {} columns, need at least 16", 
                                row_idx, row.len()));
            }
            extracted_data.push((row[4].clone(), row[5].clone(), row[6].clone(), row[7].clone(), row[8].clone()));
        }
        let first = &extracted_data[0];
        let all_identical = extracted_data.iter().all(|tuple| tuple == first);
        if ! all_identical {
            return Err("DiseaseGeneDto-related columns are not equal in all rows - requires manual check".to_string());
        }
        let disease_dto = DiseaseDto{
            disease_id: first.0.clone(),
            disease_label: first.1.clone(),
        };
        let gtr_dto = GeneTranscriptDto{
            hgnc_id: first.2.clone(),
            gene_symbol: first.3.clone(),
            transcript: first.4.clone(),
        };
        /// Note we will need to manually fix the cohort acronym for legacy files TODO possibly refactor
        let dg_dto = DiseaseGeneDto{
            template_type: TemplateType::Mendelian,
            disease_dto_list: vec![disease_dto],
            gene_transcript_dto_list: vec![gtr_dto],
            cohort_acronym:  "TODO".to_string(),
        };
        Ok(dg_dto)
    }

    /// Extract a template from the version 1 Excel files (that will be deprecated)
    /// These files do not have a disease_gene_dto element, but we know that all the
    /// excel files share the fields we will need to extract this data. 
    /// Note: This function should be deleted after the Excel files have been converted.
    pub fn from_mendelian_template(
        matrix: Vec<Vec<String>>,
        hpo: Arc<FullCsrOntology>,
        fix_errors: bool
    ) -> std::result::Result<Self, ValidationErrors> {
        let verrs = ValidationErrors::new();
        let header = HeaderDupletRow::mendelian(&matrix, hpo.clone())?;

        const HEADER_ROWS: usize = 2; // first two rows of template are header
        let hdr_arc = Arc::new(header);
        let mut ppt_rows: Vec<PpktRow> = Vec::new();
        let dg_dto = Self::get_disease_dto_from_excel(&matrix)
            .map_err(|e| ValidationErrors::from_one_err(e))?;
        for row in matrix.into_iter().skip(HEADER_ROWS) {
            let hdr_clone = hdr_arc.clone();
            let ppkt_row = PpktRow::from_row(hdr_clone, row)?;
            ppt_rows.push(ppkt_row);
        }
        
        if verrs.has_error() {
            return Err(verrs);
        }
        Ok(Self { 
                header: hdr_arc, 
                template_type: TemplateType::Mendelian,
                disease_gene_dto: dg_dto,
                hpo: hpo.clone(),
                ppkt_rows: ppt_rows
            })

    }

    fn check_duplet(&self, duplet: &HpoTermDuplet) -> std::result::Result<(), String> {
        let term_id = match TermId::from_str(duplet.hpo_id()) {
            Ok(tid) => tid,
            Err(e) => { return Err(format!("Could not create TermId from '{}': {}", duplet.hpo_id(), e)); },
        };
        match self.hpo.term_by_id(&term_id) {
            Some(term) => {
                if term.identifier().to_string() != duplet.hpo_id() {
                    return Err(format!("Identifier '{}' did not match primary ID {}", duplet.hpo_id(), term.identifier()));
                } else if term.name() != duplet.hpo_label() {
                    return Err(format!("HPO Label '{}' did not match expected label {}", duplet.hpo_label(), term.name()));
                }
            },
            None => { return Err(format!("Could not retrieve Term from TermId '{:?}'", term_id)); },
        }

        Ok(())
    }

    /// This function can be used after we have converted a DTO to a PhetoolsTemplate
    /// to check for syntactic errors in all of the fields (corresponding to all of the columns of the template)
    /// It does not check for ontology errors, e.g., a term is excluded and a child of that term is observed
    pub fn check_for_errors(&self) -> std::result::Result<(), ValidationErrors> {
        let mut verrs = ValidationErrors::new();
        for duplet in &self.header.get_hpo_duplets() {
            verrs.push_result(self.check_duplet(duplet));
        }
        for ppkt_row in &self.ppkt_rows {
            verrs.push_verr_result(ppkt_row.check_for_errors());
        }

        verrs.ok()
    }


    /// Validate the current template
    ///
    ///  * Returns
    ///
    /// - a vector of errors (can be empty)
    ///
    pub fn qc_check(&self) -> Result<()> {

        Ok(())
    }

    pub fn is_mendelian(&self) -> bool {
        self.template_type == TemplateType::Mendelian
    }

    pub fn phenopacket_count(&self) -> usize {
        self.ppkt_rows.len()
    }



    /// Delete a row. We expect this to come from a GUI where the rows include
    /// the headers (two rows) and adjust here. TODO - Consider
    /// adjusting the count in the GUI
    pub fn delete_row(&mut self, row: usize) -> Result<()> {
        if row > self.ppkt_rows.len() {
            return Err(Error::TemplateError { msg: format!("Attempt to delete row {row} but there are only {} rows", self.ppkt_rows.len()) });
        }
        self.ppkt_rows.remove(row);
        Ok(())
    }

    pub fn get_variant_dto_list(&self) {
        
    }

    /// Arranges the given HPO terms into a specific order for curation.
    ///
    /// # Arguments
    ///
    /// * `pmid` - The PubMed identifier for the new phenopacket row.
    /// * `title` - The title of the article corresponding to the pmid.
    /// * `individual_id` - The identifier of an individual described in the PMID
    /// * `hpo_items` - List of [`HpoTermDto`](struct@crate::dto::hpo_term_dto::HpoTermDto) instances describing the observed HPO features
    ///
    /// # Returns
    ///
    /// - A ``Ok(())`` upon success, otherwise ``Err(String)`.
    ///
    /// # Notes
    ///
    /// - Client code should retrieve HpoTermDto objects using the function [`Self::get_hpo_term_dto`]. This function will
    /// additionally rearrange the order of the HPO columns to keep them in "ideal" (DFS) order. Cells for HPO terms (columns) not included
    /// in the list of items but present in the columns of the previous matrix will be set to "na"
    pub fn add_row_with_hpo_data(
        &mut self,
        individual_dto: IndividualBundleDto,
        hpo_dto_items: Vec<HpoTermDto>,
        gene_variant_list: Vec<GeneVariantBundleDto>,
        cohort_dto: TemplateDto
    ) -> std::result::Result<(), ValidationErrors> {
        let mut verrs = ValidationErrors::new();
        let hpo_util = HpoUtil::new(self.hpo.clone());
        // === STEP 1: Extract all HPO TIDs from DTO and classify ===
        let dto_map: HashMap<TermId, String> = hpo_util.term_label_map_from_dto_list(&hpo_dto_items)?;
        let mut term_id_set: HashSet<TermId>  = dto_map.keys().cloned().collect();
        let existing_term_ids = self.header.get_hpo_id_list()?;
        term_id_set.extend(existing_term_ids);
         // === STEP 2: Arrange TIDs before borrowing template mutably ===
        let all_tids: Vec<TermId> = term_id_set.into_iter().collect();
        let mut term_arrager = HpoTermArranger::new(self.hpo.clone());
        let arranged_terms = term_arrager.arrange_terms(&all_tids)?;
         // === Step 3: Rearrange the existing PpktRow objects to have the new HPO terms set to "na"
        // 3a. Update the HeaderDupletRow object.
        let update_hdr = self.header.update_old(&arranged_terms);
        let updated_hdr_arc = Arc::new(update_hdr);
        // 3b. Update the existing PpktRow objects
        let mut updated_ppkt_rows: Vec<PpktRow> = Vec::new();
        let mut term_id_map: HashMap<TermId, String> = HashMap::new();
            for term in &arranged_terms {
                term_id_map.insert(term.identifier().clone(), "na".to_string());
            }
        for ppkt in &self.ppkt_rows {
            let mut tid_map = term_id_map.clone();
            match ppkt.update(&mut tid_map, updated_hdr_arc.clone()) {
                Ok(updated_ppkt) => { updated_ppkt_rows.push(updated_ppkt.clone());},
                Err(e) => {verrs.add_errors(e.errors());}
            }
        }
        // Now add the new phenopacket
        // 1. get map with TermId and Value (e.g., observed) for the new terms
        let mut tid_to_value_map: HashMap<TermId, String> = HashMap::new();
        for dto in   hpo_dto_items {
            let tid = dto.ontolius_term_id().map_err(|_| 
                    ValidationErrors::from_one_err(format!("Could not create TermId from {:?}", &dto)))?;
            tid_to_value_map.insert(tid, dto.entry().to_string());
        }
        let new_ppkt_result = PpktRow::from_dtos(updated_hdr_arc.clone(), individual_dto,  gene_variant_list, tid_to_value_map,   cohort_dto);
          println!("{}{} -- result id OK?= n={}\n\n",file!(), line!(), new_ppkt_result.is_ok());
        let ppkt_row = match new_ppkt_result {
            Ok(row) => row,
            Err(msg) => {
                verrs.push_str(msg);
                return Err(verrs);
            }
        };
        println!("{}{} -- ppkt_row={:?}\n\n",file!(), line!(), ppkt_row);
        updated_ppkt_rows.push(ppkt_row);
        self.header = updated_hdr_arc;
        self.ppkt_rows = updated_ppkt_rows;
         println!("{}{} -- ppkt rows n={}\n\n",file!(), line!(), self.ppkt_rows.len());
        
        verrs.ok()
    }

     /// get the total number of rows (which is 2 for the header plus the number of phenopacket rows)
    pub fn n_rows(&self) -> usize {
        2 + self.ppkt_rows.len()
    }

    pub fn n_columns(&self) -> usize {
        self.header.n_columns()
    }


    pub fn extract_phenopackets(
        &self,
        hgvs_dict: &HashMap<String, HgvsVariant>,
        structural_dict: &HashMap<String, StructuralVariant>,
        orcid: &str) 
    -> std::result::Result<Vec<Phenopacket>, String> {
        let mut ppkt_list: Vec<Phenopacket> = Vec::new();
        let hpo_version = self.hpo.version();
        let ppkt_exporter = PpktExporter::new(hpo_version, orcid);
        for row in &self.ppkt_rows {
            match ppkt_exporter.extract_phenopacket(row,  hgvs_dict,
                structural_dict) {
                    Ok(ppkt) =>  { ppkt_list.push(ppkt); },
                    Err(e) => { return Err(format!("Could not extract phenopacket: {}", e));},
                }
        }
        Ok(ppkt_list)
    }


    
    pub fn add_hpo_term_to_cohort(
        &mut self,
        hpo_id: &str,
        hpo_label: &str) 
    -> std::result::Result<(), ValidationErrors> {
        let mut verrs = ValidationErrors::new();
        let tid = TermId::from_str(hpo_id)
                .map_err(|_| ValidationErrors::from_one_err(
                format!("Could not arrange terms: {}\n", hpo_id)))?;
        let term = self.hpo
            .term_by_id(&tid)
            .ok_or_else(|| ValidationErrors::from_one_err(
                format!("could not retrieve HPO term for '{hpo_id}'")))?;
        // === STEP 1: Add new HPO term to existing terms and arrange TIDs ===
        let hpo_util = HpoUtil::new(self.hpo.clone());
        let mut all_tids = self.header.get_hpo_id_list()?;
        if all_tids.contains(&tid) {
            return Err(ValidationErrors::from_one_err(format!("Not allowed to add term {} because it already is present", &tid)));
        }
        all_tids.push(tid);
        let mut term_arrager = HpoTermArranger::new(self.hpo.clone());
        let arranged_terms = term_arrager.arrange_terms(&all_tids)?;
        // === Step 3: Rearrange the existing PpktRow objects to have the new HPO terms and set the new terms to "na"
        // strategy: Make a HashMap with all of the new terms, initialize the values to na. Clone this, pass it to the
        // PpktRow object, and update the map with the current values. The remaining (new) terms will be "na". Then use
        // the new HeaderDupletRow object to write the values.
        // 3a. Update the HeaderDupletRow object.
        let update_hdr = self.header.update_old(&arranged_terms);
        let updated_hdr_arc = Arc::new(update_hdr);
        let mut updated_ppkt_rows: Vec<PpktRow> = Vec::new();
        for ppkt in &self.ppkt_rows {
            let result = ppkt.update_header(updated_hdr_arc.clone());
            if let Err(e) = result {
                verrs.add_errors(e.errors());
            } else {
                let new_ppkt = result.unwrap();
                updated_ppkt_rows.push(new_ppkt);
            }
        }
        if verrs.has_error() {
            Err(verrs)
        } else {
            self.header = updated_hdr_arc.clone();
            self.ppkt_rows = updated_ppkt_rows;
            Ok(())
        }
    }
}


#[cfg(test)]
mod test {
    use crate::{dto::template_dto::{DiseaseDto, GeneTranscriptDto}};
    use ontolius::{io::OntologyLoaderBuilder};
  
    use super::*;
    use std::{fs::File, io::BufReader};
    use rstest::{fixture, rstest};
    use flate2::bufread::GzDecoder;

    #[fixture]
    fn hpo() -> Arc<FullCsrOntology> {
        let path = "resources/hp.v2025-03-03.json.gz";
        let reader = GzDecoder::new(BufReader::new(File::open(path).unwrap()));
        let loader = OntologyLoaderBuilder::new().obographs_parser().build();
        let hpo = loader.load_from_read(reader).unwrap();
        Arc::new(hpo)
    }



    #[fixture]
    fn row1() -> Vec<String> 
    {
        let row: Vec<&str> = vec![
            "PMID", "title", "individual_id", "comment", "disease_id", "disease_label", "HGNC_id",	"gene_symbol", 
            "transcript", "allele_1", "allele_2", "variant.comment", "age_of_onset", "age_at_last_encounter", 
            "deceased", "sex", "HPO",	"Clinodactyly of the 5th finger", "Hallux valgus",	"Short 1st metacarpal", 
            "Ectopic ossification in muscle tissue", "Long hallux", "Pain", "Short thumb"
        ];
        row.into_iter().map(|s| s.to_owned()).collect()
    }

    #[fixture]
    fn row2() -> Vec<String> 
    {
        let row: Vec<&str> = vec![
            "CURIE", "str", "str", "optional", "CURIE", "str", "CURIE", "str", "str", "str", "str", "optional", "age", "age", "yes/no/na", "M:F:O:U", "na",
            "HP:0004209", "HP:0001822", "HP:0010034", "HP:0011987", "HP:0001847", "HP:0012531", "HP:0009778"];
        row.into_iter().map(|s| s.to_owned()).collect()
    }

    #[fixture]
    fn row3() -> Vec<String> {
        let row: Vec<&str> =  vec![
            "PMID:29482508", "Difficult diagnosis and genetic analysis of fibrodysplasia ossificans progressiva: a case report", "current case", "", 
            "OMIM:135100", "Fibrodysplasia ossificans progressiva", "HGNC:171", "ACVR1", 
            "NM_001111067.4", "c.617G>A", "na", "NP_001104537.1:p.(Arg206His)", 
            "P9Y", "P16Y", "no", "M", "na", "na", "P16Y", "na", "P16Y", "P16Y", "P16Y", "na"];
        row.into_iter().map(|s| s.to_owned()).collect()
    }


    #[fixture]
    fn disease_gene_dto() -> DiseaseGeneDto {
        let dx_dto = DiseaseDto{ 
            disease_id: "OMIM:135100".to_string(), 
            disease_label: "Fibrodysplasia ossificans progressiva".to_string()
        };
        let gv_dto = GeneTranscriptDto{ 
            hgnc_id: "HGNC:171".to_string(), 
            gene_symbol: "ACVR1".to_string(), 
            transcript:   "NM_001111067.4".to_string(),
        };
        DiseaseGeneDto{ 
            template_type: TemplateType::Mendelian, 
            disease_dto_list: vec![dx_dto], 
            gene_transcript_dto_list: vec![gv_dto]
        }
    }


    #[fixture]
    fn original_matrix(row1: Vec<String>, row2: Vec<String>, row3: Vec<String>)  -> Vec<Vec<String>> {
        let mut rows = Vec::with_capacity(3);
        rows.push(row1);
        rows.push(row2);
        rows.push(row3);
        rows
    }

    /// Make sure that our test matrix is valid before we start changing fields to check if we pick up errors
    #[rstest]
    fn test_factory_valid_input(
        original_matrix: Vec<Vec<String>>, 
        hpo: Arc<FullCsrOntology>) {
        let factory = PheToolsTemplate::from_mendelian_template(original_matrix, hpo, false);
        assert!(factory.is_ok());
    }


    #[rstest]
    fn test_malformed_hpo_label(mut original_matrix: Vec<Vec<String>>, hpo: Arc<FullCsrOntology>) {
        // "Hallux valgus" has extra white space
        original_matrix[0][19] = "Hallux  valgus".to_string(); 
        let factory = PheToolsTemplate::from_mendelian_template(original_matrix, hpo, false);
        assert!(&factory.is_err());
        assert!(matches!(&factory, Err(ValidationErrors { .. })));
        let validation_errs = factory.err().unwrap();
        assert!(validation_errs.has_error());
        let errors = validation_errs.errors();
        assert_eq!(1, errors.len());
        let err_msg = &errors[0];
        let expected = "Expected label 'Short 1st metacarpal' but got 'Hallux  valgus' for TermId 'HP:0010034'";
        assert_eq!(expected, err_msg);
    }



    /// Test that we detect errors in labels of headings
    #[rstest]
    #[case(0, "PMI", "PMID")]
    #[case(1, "title ", "title")]
    #[case(1, " title ", "title")]
    #[case(1, "titl", "title")]
    #[case(2, "individual", "individual_id")]
    #[case(3, "comm", "comment")]
    #[case(4, "disease_i", "disease_id")]
    #[case(5, "diseaselabel", "disease_label")]
    #[case(6, "hgnc", "HGNC_id")]
    #[case(7, "symbol", "gene_symbol")]
    #[case(8, "tx", "transcript")]
    #[case(9, "allel1", "allele_1")]
    #[case(10, "allele2", "allele_2")]
    #[case(11, "vcomment", "variant.comment")]
    #[case(12, "age", "age_of_onset")]
    #[case(13, "age_last_counter", "age_at_last_encounter")]
    #[case(14, "deceasd", "deceased")]
    #[case(15, "sexcolumn", "sex")]
    #[case(16, "", "HPO")]
    fn test_malformed_title_row(
        mut original_matrix: Vec<Vec<String>>, 
        hpo: Arc<FullCsrOntology>, 
        #[case] idx: usize, 
        #[case] label: &str,
        #[case] expected_label: &str) {
        // Test that we catch malformed labels for the first row
        original_matrix[0][idx] = label.to_string(); 
        let result = PheToolsTemplate::from_mendelian_template(original_matrix, hpo, false);
        if result.is_ok() {
            println!("{}{} {}", idx, label, expected_label);
        }
        assert!(&result.is_err());
        assert!(matches!(&result, Err(ValidationErrors { .. })));
        let verr = result.err().unwrap();
        assert!(verr.has_error());
        let errors = verr.errors();
        let err_msg = errors[0].clone();
        let expected = format!("Row 0, column {}: Expected '{}' but got '{}'", 
            idx, expected_label, label);
        assert_eq!(expected, err_msg); 
    }

    // test malformed entries
    // we change entries in the third row (which is the first and only data row)
    // and introduce typical potential errors
    #[rstest]
    /*  #[case(0, "PMID29482508", "Invalid CURIE with no colon: 'PMID29482508'")]
    #[case(0, "PMID: 29482508", "Contains stray whitespace: 'PMID: 29482508'")]
    #[case(1, "", "Value must not be empty")]
    #[case(1, "Difficult diagnosis and genetic analysis of fibrodysplasia ossificans progressiva: a case report ", 
        "Trailing whitespace in 'Difficult diagnosis and genetic analysis of fibrodysplasia ossificans progressiva: a case report '")]
    #[case(2, "individual(1)", "Forbidden character '(' found in label 'individual(1)'")]
    #[case(2, " individual A", "Leading whitespace in ' individual A'")]
    #[case(4, "MIM:135100", "Disease id has invalid prefix: 'MIM:135100'")]
    #[case(4, "OMIM: 135100", "Contains stray whitespace: 'OMIM: 135100'")]
    #[case(4, "OMIM:13510", "OMIM identifiers must have 6 digits: 'OMIM:13510'")]
    #[case(5, "Fibrodysplasia ossificans progressiva ", "Trailing whitespace in 'Fibrodysplasia ossificans progressiva '")]
    #[case(6, "HGNC:171 ", "Contains stray whitespace: 'HGNC:171 '")]
    #[case(6, "HGNC171", "Invalid CURIE with no colon: 'HGNC171'")]
    #[case(6, "HGNG:171", "HGNC id has invalid prefix: 'HGNG:171'")]
    #[case(7, "ACVR1 ", "Trailing whitespace in 'ACVR1 '")]
    #[case(8, "NM_001111067", "Transcript 'NM_001111067' is missing a version")]
    #[case(9, "617G>A", "Malformed allele '617G>A'")]
    #[case(10, "", "Value must not be empty")]
    #[case(12, "P2", "Malformed age_of_onset 'P2'")]
    #[case(13, "Adultonset", "Malformed age_at_last_encounter 'Adultonset'")]
    #[case(14, "?", "Malformed deceased entry: '?'")]
    #[case(14, "alive", "Malformed deceased entry: 'alive'")]
    #[case(15, "male", "Malformed entry in sex field: 'male'")]
    #[case(15, "f", "Malformed entry in sex field: 'f'")]*/
    #[case(18, "Observed", "Malformed entry for Ectopic ossification in muscle tissue (HP:0011987): 'Observed'")]
    #[case(18, "yes", "Malformed entry for Ectopic ossification in muscle tissue (HP:0011987): 'yes'")]
    #[case(18, "exc.", "Malformed entry for Ectopic ossification in muscle tissue (HP:0011987): 'exc.'")]
    fn test_malformed_entry(
        mut original_matrix: Vec<Vec<String>>, 
        hpo: Arc<FullCsrOntology>, 
        #[case] idx: usize, 
        #[case] entry: &str,
        #[case] error_msg: &str) 
    {
        original_matrix[2][idx] = entry.to_string();
        let result = PheToolsTemplate::from_mendelian_template(original_matrix, hpo, false);
        assert!(result.is_err());
        let verr = result.err().unwrap();
        for e in verr.errors() {
            println!("{}", e);
        }
    
       /*  let templates = factory.get_templates().unwrap();
        assert_eq!(1, templates.len());
        let itemplate = &templates[0];
        let result = itemplate.qc_check();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(error_msg, err.to_string());*/
    }


}
