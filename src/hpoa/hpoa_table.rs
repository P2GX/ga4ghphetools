use std::{collections::HashSet, fs::File, io::{BufWriter, Write}, path::PathBuf, sync::Arc};


use chrono::Local;
use ontolius::ontology::csr::FullCsrOntology;
use regex::Regex;

use crate::{dto::cohort_dto::{CohortData, DiseaseData}, hpoa::{counted_hpo_term::CountedHpoTerm, hpoa_onset_calculator::HpoaOnsetCalculator, hpoa_table_row::HpoaTableRow, hpo_term_counter::HpoTermCounter}};





pub struct HpoaTable {
    hpoa_row_list: Vec<HpoaTableRow>,
    file_name: String
}

impl HpoaTable {

    pub fn new(
        cohort: CohortData, 
        hpo: Arc<FullCsrOntology>,
        biocurator: &str) -> Result<Self, String>{
        let todays_date = Local::now().format("%Y-%m-%d").to_string();
        if ! Self::is_valid_orcid(biocurator) {
            return Err(format!("Malformed biocurator string ({biocurator}). Must be the ORCID 16-digit identifier (only)."));
        }
        let biocurator = format!("ORCID:{biocurator}[{todays_date}]");
        if ! cohort.is_mendelian() {
            return Err(format!("Can only export Mendelian HPOA table, but this cohort is {:?}", 
                cohort.cohort_type));
        }
        let hpo_header = &cohort.hpo_headers;
        let onset_term_list: Vec<CountedHpoTerm> = HpoaOnsetCalculator::pmid_to_onset_freq_d(&cohort)?;
        let hpo_counted_term_list = HpoTermCounter::pmid_term_count_list(&cohort)?;
        let disease_data = Self::get_disease_data(&cohort)?;
        let mut hpoa_rows = Vec::new();
        for counted_term in hpo_counted_term_list {
            let row = HpoaTableRow::from_counted_term(&disease_data, counted_term, &biocurator)?;
            hpoa_rows.push(row);
        }
        for counted_onset in onset_term_list {
            let row = HpoaTableRow::from_counted_term(&disease_data,counted_onset, &biocurator)?;
            hpoa_rows.push(row);
        }
        let file_name = Self::get_hpoa_filename(&cohort)?;
        Ok(Self{
            hpoa_row_list: hpoa_rows,
            file_name
        })

    }


    /// Extract the DiseaseData object from the Cohort.
    /// We check if there is only one such object, because the HPOA export is only intended for Mendelian disease cohorts
    /// and if we have zero or two, then there is some error.
    fn get_disease_data(cohort: &CohortData) -> Result<DiseaseData, String> {
        if cohort.disease_gene_data.disease_data_map.len() != 1 {
            return Err(format!("Exactly one disease required but there were {}", cohort.disease_gene_data.disease_data_map.len()));
        }
        match cohort.disease_gene_data.disease_data_map.values().next() {
            Some(disease_data) => Ok(disease_data.clone()),
            None => Err(format!("Exactly one disease required but there were none"))
        }
    }

    /// Regex for ORCID IDs: 0000-0000-0000-0000 where the last char can be a digit or X
    fn is_valid_orcid(orcid: &str) -> bool {
        let re = Regex::new(r"^\d{4}-\d{4}-\d{4}-\d{3}[\dX]$").unwrap();
        re.is_match(orcid)
    }
    
    pub fn get_dataframe(&self) -> Vec<Vec<String>> {
        let mut rows:  Vec<Vec<String>> = Vec::new();
        rows.push(HpoaTableRow::header_fields());
        for r in &self.hpoa_row_list {
            rows.push(r.row());
        }
        rows
    }


    /// Create the filename for the cohort, something like MFS-FBN1.tsv
    fn get_hpoa_filename(cohort: &CohortData) -> Result<String, String> {
         if ! cohort.is_mendelian() {
            return Err(format!("HPOA export only supported for Mendelian. Invalid for '{:?}'", cohort.cohort_type));
        }
        let gt_map = &cohort.disease_gene_data.gene_transcript_data_map;

        if gt_map.len() != 1 {
            return Err(format!("HPOA export only supported for one gene (Mendelian) but we got '{}'", gt_map.len()));
        }
        let gt = match gt_map.values().cloned().into_iter().next() {
            Some(gtr) => gtr.clone(),
            None => { return Err(format!("Could not get GeneTranscriptData"));}
        };
        match &cohort.cohort_acronym {
            Some(acronym) => {
                let outfile = format!("{}-{}.tsv", gt.gene_symbol, acronym);
                Ok(outfile)
            },
            None =>  Err(format!("HPOA export requires cohort acronym but got '{:?}'", cohort.cohort_acronym))
        }
    }

    /// Output the HPOA-format file representing data from the entire cohort.
    /// This format is used in the internal pipeline of the HPO project that generates teh
    /// phenotype.hpoa file for releases. The file can be used by that pipeline to add
    /// data to existing data for a disease or to create the disease file (for new diseases).
    /// We have used the PhenoteFX tool to manage this. This function should not be
    /// needed except by the internal HPO team.
    pub fn write_tsv(&self, path: &PathBuf) -> std::io::Result<()> {  
        let file_path: PathBuf = path.join(&self.file_name);
         let file = File::create(file_path)?;
        let mut writer = BufWriter::new(file);

        for row in self.get_dataframe() {
            writeln!(writer, "{}", row.join("\t"))?;
        }

        Ok(())
    }


}



#[cfg(test)]
mod test {
    use ontolius::{io::OntologyLoaderBuilder};
  
    use super::*;
    use std::{fs::{self, File}, io::BufReader};
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
    fn biocurator() -> String {
        "0000-0002-0736-9199".to_string()
    }

     #[fixture]
     fn cohort() -> CohortData {
        let json_template_path = "/Users/robin/GIT/phenopacket-store/notebooks/NT5C2/NT5C2_SPG45_individuals.json";
         let file_data = fs::read_to_string(json_template_path)
            .map_err(|e| 
                format!("Could not extract string data from {}: {}", json_template_path, e.to_string())).unwrap();
        let cohort: CohortData = serde_json::from_str(&file_data)
            .map_err(|e| format!("Could not transform string {} to CohortDto: {}",
                file_data, e.to_string())).unwrap();
        cohort
     }


     #[rstest]
     #[ignore = "for development, add better test once API stable for new template"]
     fn test_hpoa(
        hpo: Arc<FullCsrOntology>,
        biocurator: String,
        cohort: CohortData
     ) {
        let hpoa = HpoaTable::new(cohort, hpo, &biocurator).unwrap();
        let matrix = hpoa.get_dataframe();
        assert!(matrix.len() > 2);
        let outpath =  "/Users/robin/GIT/phenopacket-store/notebooks/NT5C2/NT5C2_SPG45_TEST.hpoa";
        let pth = PathBuf::from(outpath);
        hpoa.write_tsv(&pth).unwrap();
     }



}