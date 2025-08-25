use std::{collections::{HashMap, HashSet}, fs::File, io::{BufWriter, Write}, path::PathBuf, sync::Arc};


use chrono::Local;
use ontolius::ontology::csr::FullCsrOntology;

use crate::{dto::{cohort_dto::{CohortData, DiseaseData}, hpo_term_dto::CellValue}, hpoa::{hpoa_onset_calculator::HpoaOnsetCalculator, hpoa_table_row::HpoaTableRow, pmid_counter::PmidCounter}};





pub struct HpoaTable {
    hpoa_row_list: Vec<HpoaTableRow>,
    today: String
}

impl HpoaTable {

    pub fn new(
        cohort: CohortData, 
        hpo: Arc<FullCsrOntology>,
        biocurator: &str) -> Result<Self, String>{
        let todays_date = Local::now().format("%Y-%m-%d").to_string();
        if biocurator.starts_with("O") {
            return Err(format!("Malformed biocurator string ({biocurator}). Must be the ORCID 16-digit identifier (only)."));
        }
        let biocurator = format!("ORCID:{biocurator}[{todays_date}]");
        if ! cohort.is_mendelian() {
            return Err(format!("Can only export Mendelian HPOA table, but this cohort is {:?}", 
                cohort.cohort_type));
        }
        let hpo_header = cohort.hpo_headers;
        let mut pmid_map: HashMap<String, PmidCounter> = HashMap::new();
        let mut onset_map: HashMap<String, HpoaOnsetCalculator> = HashMap::new();
        let mut disease_set: HashSet<DiseaseData> = HashSet::new();
        let mut hpoa_rows = Vec::new();
        for row in &cohort.rows {
            if row.disease_dto_list.len() != 1 {
                // should never happen
                return Err("Can only export Mendelian (one disease) HPOA file".to_string());
            }
            let disease_dto = row.disease_dto_list[0].clone();
            disease_set.insert(disease_dto);
            let pmid = &row.individual_dto.pmid;
            let disease_onset = &row.individual_dto.age_of_onset;
            if disease_onset != "na" {
                let onset_c = onset_map.entry(pmid.to_string()).or_insert(HpoaOnsetCalculator::new());
                onset_c.add_onset(&disease_onset)?;
            }
            let counter = pmid_map
                .entry(pmid.clone())
                .or_insert(PmidCounter::new(pmid));
            // Iterate across HPO terms and add to counter 
            if hpo_header.len() != row.hpo_data.len() {
                let mut i = 0 as usize;
                for h in &hpo_header {
                    i += 1;
                    let data = if i < row.hpo_data.len() {
                        row.hpo_data[i].to_string()
                    } else {
                        "ran out".to_string()
                    };
                    println!("{}) hpo_header:{:?} // row_data: {}", i, h, data);
                }
                 for h in &row.hpo_data {
                    i += 1;
                    println!("{}) {:?} ", i, h);
                }
                return Err(format!("Length mismatch: hpo_header has {}, hpo_data has {}", hpo_header.len(), row.hpo_data.len()));
            }
            for (hpo_duplet, data_item) in hpo_header.iter().zip(row.hpo_data.iter()) {
                let hpo_id = hpo_duplet.hpo_id();
                let label = hpo_duplet.hpo_label();
                if *data_item == CellValue::Na {
                    continue;
                } else if *data_item == CellValue::Excluded {
                    counter.excluded(hpo_id);
                } else if *data_item == CellValue::Observed {
                    counter.observed(hpo_id);
                }  else {
                    println!("[INFO] Unknown HPO cell contents '{:?}' for HPO '{}'", data_item, hpo_id);
                }
            }
        }
        if disease_set.len() != 1 {
            return Err(format!("Expected exactly one disease, found {}", disease_set.len()));
        }
        let disease_dto = disease_set.into_iter().next().unwrap();
        for (pmid, counter) in &pmid_map {
            for hpo_duplet in &hpo_header {
               
                if counter.contains(hpo_duplet.hpo_id()) {
                    let freq = counter.get_freq(hpo_duplet.hpo_id())?;
                    let row = HpoaTableRow::new(
                        &disease_dto, 
                        hpo_duplet.hpo_id(), 
                        hpo_duplet.hpo_label(),
                        &freq,
                        &pmid, 
                        &biocurator)?;
                    hpoa_rows.push(row);
                }
            }
        }

        Ok(Self{
            hpoa_row_list: hpoa_rows,
            today: todays_date
        })

    }


    
    pub fn get_dataframe(&self) -> Vec<Vec<String>> {
        let mut rows:  Vec<Vec<String>> = Vec::new();
        rows.push(HpoaTableRow::header_fields());
        for r in &self.hpoa_row_list {
            rows.push(r.row());
        }
        rows
    }

    /// Output the HPOA-format file representing data from the entire cohort.
    /// This format is used in the internal pipeline of the HPO project that generates teh
    /// phenotype.hpoa file for releases. The file can be used by that pipeline to add
    /// data to existing data for a disease or to create the disease file (for new diseases).
    /// We have used the PhenoteFX tool to manage this. This function should not be
    /// needed except by the internal HPO team.
    pub fn write_tsv(&self, path: &PathBuf) -> std::io::Result<()> {
        let file = File::create(path)?;
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