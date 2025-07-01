//! PptTemplate2
//!
//! The struct that contains all data needed to create or edit a cohort of phenopackets
//! in "pyphetools" format, and to export GA4GH Phenopackets.
//! Each template has the following main members
//! - HeaderDupletRow (defines each of the columns)
//! - A list of PpktRow (one per phenopacket)
use std::{collections::{HashMap, HashSet}, fmt::format, str::FromStr, sync::Arc, vec};

use ontolius::{
    ontology::{csr::FullCsrOntology, OntologyTerms},
    term::{simple::{SimpleMinimalTerm, SimpleTerm}, MinimalTerm},
    Identified, TermId,
};
use phenopackets::schema::v2::Phenopacket;
use prost::Name;

use crate::{dto::{case_dto::CaseDto, hpo_term_dto::HpoTermDto, template_dto::{IndividualBundleDto, RowDto, TemplateDto}, validation_errors::ValidationErrors}, error::{self, Error, Result}, header::hpo_term_duplet::HpoTermDuplet, hpo::hpo_util::HpoUtil, ppkt::{ppkt_exporter::{self, PpktExporter}, ppkt_row::PpktRow}, template::header_duplet_row::HeaderDupletRow};
use crate::{
    template::disease_gene_bundle::DiseaseGeneBundle,
    hpo::hpo_term_arranger::HpoTermArranger
};

use super::{operations::Operation};

/// Phetools can be used to curate cases with Mendelian disease or with melded phenotypes
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TemplateType {
    Mendelian,
    Melded,
}

/// All data needed to edit a cohort of phenopackets or export as GA4GH Phenopackets
pub struct PheToolsTemplate {
    header: Arc<HeaderDupletRow>,
    template_type: TemplateType,
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

impl Error {
    fn empty_template(nlines: usize) -> Self {
        let msg = format!("Valid template must have at least three rows (at least one data row) but template has only {nlines} rows");
        Error::TemplateError { msg }
    }
    fn unequal_row_lengths() -> Self {
        let msg = format!("Not all rows of template have the same number of fields");
        Error::TemplateError { msg }
    }

    fn could_not_find_column(colname: &str) -> Self {
        let msg = format!("Could not retrieve {colname} column");
        Error::TemplateError { msg }
    }
}

impl PheToolsTemplate {
    /// Create the initial pyphetools template (Table)
    pub fn create_pyphetools_template_mendelian(
        dg_bundle: DiseaseGeneBundle,
        hpo_term_ids: Vec<TermId>,
        // Reference to the Ontolius Human Phenotype Ontology Full CSR object
        hpo: Arc<FullCsrOntology>,
    ) -> Result<Self> {
        let mut hp_header_duplet_list: Vec<HpoTermDuplet> = Vec::new();
        for hpo_id in hpo_term_ids {
            match hpo.term_by_id(&hpo_id) {
                Some(term) => {
                    let hpo_duplet = HpoTermDuplet::new(term.name(), term.identifier().to_string());
                    hp_header_duplet_list.push(hpo_duplet);
                }
                None => {
                    return Err(Error::HpIdNotFound {
                        id: hpo_id.to_string(),
                    });
                }
            }
        }/*
        let header_dup_row = HeaderDupletRow::mendelian(hp_header_duplet_list);
        let hdr_arc = Arc::new(header_dup_row);
        Ok(Self {
            header: hdr_arc,
            template_type: TemplateType::Mendelian,
            hpo: hpo.clone(),
            ppkt_rows: vec![]
        }) */
       eprint!("refacotr");
       Err(Error::TemplateError { msg: "OUAGHDASAS".to_string() })
        
    }


   
    

    pub fn get_template_dto(&self) -> Result<TemplateDto> {
        let header_dto = self.header.get_hpo_header_dtos();
        println!("get_template_dto: {:?}", header_dto);
        let row_dto_list: Vec<RowDto> = self.ppkt_rows
            .iter()
            .map(RowDto::from_ppkt_row)
            .collect();
        Ok(TemplateDto::mendelian(header_dto, row_dto_list))
    }

    pub fn from_template_dto(
        template_dto: TemplateDto, 
        hpo: Arc<FullCsrOntology>) 
    -> std::result::Result<Self, ValidationErrors> {
        let header_duplet_row = match template_dto.cohort_type.as_str() {
            "mendelian" => HeaderDupletRow::new_mendelian_ppkt_from_dto(template_dto.hpo_headers),
            other => {
                return Err(ValidationErrors::from_string(format!("Only Mendelian implemented. We cannot yet handle '{other}'")));
            }
        };
        let header_arc = Arc::new(header_duplet_row);
        let mut ppkt_rows: Vec<PpktRow> = Vec::new();
        for row_dto in template_dto.rows {
            let ppkt_row = PpktRow::from_dto(row_dto, header_arc.clone());
            ppkt_rows.push(ppkt_row);
        }
        let template = PheToolsTemplate {
            header: header_arc.clone(),
            template_type: TemplateType::Mendelian,
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
        dg_bundle: DiseaseGeneBundle,
        hpo_term_ids: Vec<TermId>,
        hpo: Arc<FullCsrOntology>,
    ) -> Result<PheToolsTemplate> {
        let mut smt_list: Vec<SimpleMinimalTerm> = Vec::new();
        for hpo_id in &hpo_term_ids {
            match hpo.term_by_id(hpo_id) {
                Some(term) => {
                    let smt =
                        SimpleMinimalTerm::new(term.identifier().clone(), term.name(), vec![], false);
                    smt_list.push(smt);
                }
                None => {
                    return Err(Error::HpIdNotFound {
                        id: hpo_id.to_string(),
                    });
                }
            }
        }
    
        let result = Self::create_pyphetools_template_mendelian(dg_bundle, hpo_term_ids, hpo)?;
        Ok(result)
    }



    pub fn from_mendelian_template(
        matrix: Vec<Vec<String>>,
        hpo: Arc<FullCsrOntology>,
    ) -> std::result::Result<Self, ValidationErrors> {
        println!("from_mendelian_template");
        let verrs = ValidationErrors::new();
        let header = HeaderDupletRow::mendelian(&matrix, hpo.clone())?;

        const HEADER_ROWS: usize = 2; // first two rows of template are header
        let hdr_arc = Arc::new(header);
        let mut ppt_rows: Vec<PpktRow> = Vec::new();
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

    /// Get a value from a column that we expect to be the same in all phenopackets (DiseaseGeneBundle)
    fn get_unique(&self, i: usize) -> Result<String> {
       // let mut value_set = HashSet::new();
        /* *
        for ppkt in &self.ppkt_rows {
            let value = ppkt.get_value_at(i)?;
            value_set.insert(value);
        }
        if value_set.is_empty() {
            Err(Error::TemplateError { msg: format!("contents empty") })
        } else if value_set.len() > 1 {
            Err(Error::TemplateError { msg: format!("Multiple values found") })
        } else {
            Ok(value_set.iter().next().unwrap().to_string())
        }
        */
        Ok(format!("REFACOTR get unique"))

    }

    pub fn get_mendelian_disease_gene_bundle(&self) -> Result<DiseaseGeneBundle> {
        if ! self.is_mendelian() {
            return  Err(Error::TemplateError { msg: format!("The template is not Mendelian") })
        }
        let disease_id_idx = 4; //self.header.get_idx("disease_id")?;
        let disease_label_idx = 5; //self.header.get_idx("disease_label")?;
        let hgnc_idx = 6; //self.header.get_idx("HGNC_id")?;
        let symbol_idx =7;// self.header.get_idx("gene_symbol")?;
        let transcript_idx =8;// self.header.get_idx("transcript")?;
        eprint!("NEED TO REFACTOR get_mendelian_disease_gene_bundle");
        let disease_id = self.get_unique(disease_id_idx)?;
        let disease_tid = TermId::from_str(&disease_id).map_err(|e| Error::termid_parse_error(disease_id))?;
        let disease_name = self.get_unique(disease_label_idx)?;
        let hgnc = self.get_unique(hgnc_idx)?;
        let hgnc_tid = TermId::from_str(&hgnc).map_err(|e| Error::termid_parse_error(hgnc))?;
        let symbol = self.get_unique(symbol_idx)?;
        let transcript = self.get_unique(transcript_idx)?;
        let dgb = DiseaseGeneBundle::new(&disease_tid, disease_name, &hgnc_tid, symbol, transcript)?;
        Ok(dgb)
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
        return self.template_type == TemplateType::Mendelian;
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
        hpo_dto_items: Vec<HpoTermDto>
    ) -> std::result::Result<(), ValidationErrors> {
        let mut verrs = ValidationErrors::new();
        let hpo_util = HpoUtil::new(self.hpo.clone());
        // === STEP 1: Extract all HPO TIDs from DTO and classify ===
        let mut dto_map: HashMap<TermId, String> = hpo_util.term_label_map_from_dto_list(&hpo_dto_items)?;
        let mut term_id_set: HashSet<TermId>  = dto_map.keys().cloned().collect();
        let existing_term_ids = self.header.get_hpo_id_list()?;
        term_id_set.extend(existing_term_ids);
         // === STEP 2: Arrange TIDs before borrowing template mutably ===
        let all_tids: Vec<TermId> = term_id_set.into_iter().collect();
        let mut term_arrager = HpoTermArranger::new(self.hpo.clone());
        let arranged_terms = term_arrager.arrange_terms(&all_tids)?;
         // === Step 3: Rearrange the existing PpktRow objects to have the new HPO terms and set the new terms to "na"
        // strategy: Make a HashMap with all new terms, initialize the values to na. Clone this, pass it to the
        // PpktRow object, and update the map with the current values. The remaining (new) terms will be "na". Then use
        // the new HeaderDupletRow object to write the values.
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
        /// Now add the new phenopacket
        let mut tid_map = term_id_map.clone();
        /*let new_ppkt = PpktRow::mendelian_from_dto( 
            updated_hdr_arc.clone(), 
            individual_dto, 
            hpo_dto_items,
            tid_map)?;
        updated_ppkt_rows.push(new_ppkt);*/
        self.header = updated_hdr_arc;
        self.ppkt_rows = updated_ppkt_rows;
        
        
        verrs.ok()
    }

     /// get the total number of rows (which is 2 for the header plus the number of phenopacket rows)
    pub fn n_rows(&self) -> usize {
        2 + self.ppkt_rows.len()
    }

    pub fn n_columns(&self) -> usize {
        self.header.n_columns()
    }

    pub fn get_summary(&self) -> HashMap<String, String> {
        let mut summary: HashMap<String, String> = HashMap::new();
        let result = self.get_mendelian_disease_gene_bundle();
        if result.is_err() {
            return summary;
        }
        let dgb = result.unwrap();
        summary.insert("disease".to_string(), dgb.disease_name());
        summary.insert("disease_id".to_string(), dgb.disease_id_as_string());
        summary.insert("hgnc_id".to_string(), dgb.hgnc_id_as_string());
        summary.insert("gene_symbol".to_string(), dgb.gene_symbol());
        summary.insert("transcript".to_string(), dgb.transcript());
        let hpo_terms = self.header.hpo_count();
        summary.insert("hpo_term_count".to_string(), format!("{}", hpo_terms));
        let ppkt_n = self.phenopacket_count();
        summary.insert("n_ppkt".to_string(), format!("{}", ppkt_n));
        summary
    }

    pub fn export_phenopackets(&self) -> Vec<Phenopacket> {
        let mut ppkt_list: Vec<Phenopacket> = Vec::new();
        let hpo_version = "TEMP";
        let creator_orcid = "TEMP_ORCID";
        let ppkt_exporter = PpktExporter::new(hpo_version, creator_orcid);
        for row in &self.ppkt_rows {
            let ppkt = ppkt_exporter.export_phenopacket(row);
            if ppkt.is_ok() {
                ppkt_list.push(ppkt.unwrap());
            } else {
                eprintln!("TODO ERROR HANDLINGS");
            }
        }

        ppkt_list
    }


    
    pub fn add_hpo_term_to_cohort(
        &mut self,
        hpo_id: &str,
        hpo_label: &str) -> std::result::Result<(), ValidationErrors> {
            let mut verrs = ValidationErrors::new();
            let tid = TermId::from_str(hpo_id);
            if tid.is_err() {
                return Err(ValidationErrors::from_one_err(format!("Could not arrange terms: {}\n", hpo_id)));
            };
            let tid = tid.unwrap();
            let term = match &self.hpo.term_by_id(&tid) {
                Some(term) => term,
                None =>{ return  Err(ValidationErrors::from_one_err(format!("could not retrieve HPO term for '{hpo_id}'"))); }
            };
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
    use crate::{error::Error, header::{hpo_term_duplet::HpoTermDuplet}};
    use lazy_static::lazy_static;
    use ontolius::{io::OntologyLoaderBuilder, ontology::csr::MinimalCsrOntology};
    use polars::io::SerReader;
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
        let factory = PheToolsTemplate::from_mendelian_template(original_matrix, hpo);
        assert!(factory.is_ok());
    }


    #[rstest]
    fn test_malformed_hpo_label(mut original_matrix: Vec<Vec<String>>, hpo: Arc<FullCsrOntology>) {
        // "Hallux valgus" has extra white space
        original_matrix[0][19] = "Hallux  valgus".to_string(); 
        let factory = PheToolsTemplate::from_mendelian_template(original_matrix, hpo);
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
        let result = PheToolsTemplate::from_mendelian_template(original_matrix, hpo);
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
        let result = PheToolsTemplate::from_mendelian_template(original_matrix, hpo);
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
