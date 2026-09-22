//! CohortDir
//! Represents a directory for one gene with all contained files and metadata.
//! We use the data for Q/C reports

use std::{collections::HashMap, fs::self, hash::Hash, path::PathBuf, sync::Arc};
use ontolius::ontology::csr::FullCsrOntology;
use phenopackets::schema::v2::Phenopacket;
use walkdir::WalkDir;
use std::path::Path;
use crate::{
    cohort_qc::{disease_qc::DiseaseQc, qc_report::QcReport}, dto::cohort_dto::CohortData, error::{PheToolsError, cohort_error::CohortError}, ppkt, repo::{CohortWrapper, PpktWrapper}};


#[derive(Clone, Debug, Default)]
pub struct CohortDir {
    /// Directory name — usually a gene symbol (e.g. ZRSR2), but can also be a
    /// chromosomal region or syndrome name (e.g. 11q_terminal_deletion). This
    /// is the directory's identity, not any single cohort's — a directory can
    /// hold multiple cohort files (e.g. multiple diseases for one gene).
    pub directory_name: String,
    /// Reference to full path of the directory
    pub directory_path: PathBuf,
    /// The specific file or files like "ZRSR2_OFD21_individuals.json" (there must be at least one but can be many)
    pub individuals_json: Vec<PathBuf>,
    /// All JSON files inside the 'phenopackets' subdirectory (there must be at least one)
    pub ppkt_path_list: Vec<PathBuf>,
    /// Map from disease id to list of phenopacket wrappers with info about disease id, Path, and the phenopacket
    pub ppkt_path_map: HashMap<String, Vec<PpktWrapper>>,
    /// One of these for each disease in the current directory (e.g., all diseases associated with a gene)
    pub cohort_list: Vec<CohortWrapper>,
    /// Any files or directories that don't belong in the standard structure
    pub unexpected_entries: Vec<PathBuf>,
    /// Any errors encountered during loading of data
    pub loading_errors: Vec<String>,
}


impl CohortDir {
    
    /// Called by GptRepository::new, gets all of the CohortData files in each gene directory.
    /// Each gene directory is named according to the gene symbol (e.g., FBN1) and
    /// contains one or multiple CohortData files -- one per disease associated with the gene.
    pub fn process_gene_directory(path: &Path) -> CohortDir {
        let mut cohort_dir = CohortDir {
            directory_name: path.file_name().unwrap_or_default().to_string_lossy().into(),
            directory_path: path.to_path_buf(),
            ..Default::default()
        };
        let mut individuals_json: Vec<PathBuf> = Vec::new();
        let mut ppkt_path_list: Vec<PathBuf> = Vec::new();
        let mut loading_errors: Vec<String> = Vec::new();
        let mut ppkt_map: HashMap<String, Vec<PpktWrapper>> = HashMap::new();
        // Iterate through the immediate children of the gene directory
        for entry in WalkDir::new(path).min_depth(1).max_depth(1).into_iter().filter_map(|e| e.ok()) {
            let file_name = entry.file_name().to_string_lossy();

            if entry.file_type().is_dir() && file_name == "phenopackets" {
                // Recurse into phenopackets
                ppkt_path_list = WalkDir::new(entry.path())
                    .min_depth(1)
                    .into_iter()
                    .filter_map(|e| e.ok())
                    .filter(|e| e.file_type().is_file())
                    .map(|e| e.into_path())
                    .collect();
            } else if entry.file_type().is_file() && file_name.ends_with("_individuals.json") {
                cohort_dir.individuals_json.push(entry.into_path());
            } else {
                // Anything else (odd files, extra folders) is flagged
                cohort_dir.unexpected_entries.push(entry.into_path());
            }
        }
         // 2. Load and group phenopackets by disease id.
        let mut ppkt_wrappers: Vec<PpktWrapper> = Vec::new();
        for ppkt_json in &ppkt_path_list {
            let ppkt = match ppkt::load_phenopacket(ppkt_json) {
                Ok(p) => p,
                Err(e) => {
                    cohort_dir.loading_errors.push(format!(
                        "Could not load phenopacket at '{}': {}",
                        ppkt_json.display(), e
                    ));
                    continue;
                }
            };
            let disease_id = match ppkt::get_disease_id(&ppkt) {
                Ok(id) => id,
                Err(e) => {
                    cohort_dir.loading_errors.push(format!(
                        "Could not extract disease id for phenopacket at '{}': {}",
                        ppkt_json.display(), e
                    ));
                    continue;
                }
            };
            ppkt_wrappers.push(PpktWrapper::new(ppkt_json, &disease_id, ppkt));
        }
        // 3. Build one CohortWrapper per individuals_json file (one per disease).
        //    <-- THIS is where the `for cohort_path in individuals_json` loop goes.
        for cohort_path in individuals_json {
            let cohort = match Self::read_cohort(&cohort_path) {
                Ok(c) => c,
                Err(e) => {
                    cohort_dir.loading_errors.push(format!(
                        "Could not load cohort data at '{}': {}",
                        cohort_path.display(), e
                    ));
                    continue;
                }
            };
            let disease_id = match Self::derive_disease_id_from_cohort(&cohort) {
                Ok(id) => id,
                Err(e) => {
                    cohort_dir.loading_errors.push(format!(
                        "Could not determine disease id for '{}': {}",
                        cohort_path.display(), e
                    ));
                    continue;
                }
            };
            cohort_dir.cohort_list.push(CohortWrapper::new(
                disease_id,
                cohort,
                cohort_path,
                &ppkt_wrappers,
            ));
        }
 
    
        cohort_dir
    }

    

    
    pub fn read_cohort(path: &Path) -> Result<CohortData, CohortError> {
        let file_data = fs::read_to_string(path)
            .map_err(|e| CohortError::io_error(path, &e.to_string()))?;
        let cohort: CohortData = serde_json::from_str(&file_data)
            .map_err(|e| CohortError::json_error(&file_data, &e.to_string()))?;
        Ok(cohort)
    }
    
    pub fn get_cohort_data(&self) -> Result<Vec<CohortData>, CohortError> {
        let mut cohorts: Vec<CohortData> = Vec::new();
        for pth in &self.individuals_json {
            let cohort = Self::read_cohort(pth)?;
            cohorts.push(cohort);
        }
        Ok(cohorts)
    }


    pub fn get_ppkt_w_list_by_disease_id(&self, disease_id: &str) -> &[PpktWrapper] {
        self.ppkt_path_map
            .get(disease_id)
            .map_or(&[], Vec::as_slice)
    }


   pub fn filter_ppkt_by_disease(ppkt_list: &[Phenopacket], disease_id: &str) -> Result<Vec<Phenopacket>, PheToolsError> {
        let mut filtered = Vec::new();
        for ppkt in ppkt_list {
            let ppkt_disease_id = ppkt::get_disease_id(ppkt)?;
            if ppkt_disease_id == disease_id {
                filtered.push(ppkt.clone());
            }
        }
        Ok(filtered)
    }

    pub fn get_phenopackets(&self) -> Result<Vec<Phenopacket>, String> {
        let mut ppkt_list = Vec::new();
        for ppkt_path in &self.ppkt_path_list {
            let ppkt = ppkt::load_phenopacket(ppkt_path)?;
            ppkt_list.push(ppkt);
        }
        Ok(ppkt_list)
    }

    pub fn get_ppkt_map(&self) -> Result<HashMap<PathBuf, Phenopacket>, String> {
        let mut ppkt_map: HashMap<PathBuf, Phenopacket> = HashMap::new();
        for ppkt_path in &self.ppkt_path_list {
            eprint!("get_ppm{:?}", ppkt_path);
            let ppkt = ppkt::load_phenopacket(ppkt_path)?;
            ppkt_map.insert(ppkt_path.clone(),ppkt);
        }
        Ok(ppkt_map)
    }

 
    pub fn get_unexpected_file_names(&self) -> Vec<String> {
        let mut fnames: Vec<String> = Vec::new();
        for pth in &self.unexpected_entries {
            if let Some(file_name) = pth.file_name() {
                let name_str = file_name.to_string_lossy();
                fnames.push(name_str.to_string());
            } else {
                eprintln!("Error: Could not extract filename from {:?}", pth);
            }
        }
        fnames
    }

    /// The GENE_DISEASE_individuals.json files are the files that contain information
    /// about the cohorts. This function returns a list of references to these files 
    /// so that clients can iterate over all cohorts in a cohort (gene) directory.
    pub fn get_individuals_json_files(&self) -> &[PathBuf] {
        &self.individuals_json
    }

    pub fn get_cohort_path(&self) -> &Path {
        &self.directory_path
    }

  

    /// Get Q/C reports for this Cohort Directory
    pub fn get_cohort_dir_qc(&self, hpo: Arc<FullCsrOntology>) -> Result<QcReport, String> {
        let cohorts = self.get_cohort_data().map_err(|e|e.to_string())?;
        let phenopackets = self.get_phenopackets()?;
        let unexpected_files = self.get_unexpected_file_names();
        let mut qc_report = QcReport::new(&self.directory_name);
        for unexp in unexpected_files.iter() {
            qc_report.unexpected_file(unexp);
        }
        // Divide up the phenopackets according to cohort
        let mut ppkt_by_disease: HashMap<String, Vec<&Phenopacket>> = HashMap::new();
        for ppkt in phenopackets.iter() {
            match ppkt::get_disease_id(&ppkt) {
                Ok(disease_id) => ppkt_by_disease.entry(disease_id).or_default().push(ppkt),
                Err(e) => qc_report.no_disease_id(&ppkt.id),
            }
        }
        for cohort in &cohorts {
            if ! cohort.is_mendelian() {
                qc_report.non_mendelian(cohort);
                continue;
            }
            let Some(disease_data) = cohort.disease_list.first() else {
                qc_report.no_disease();
                continue;
            };
            let disease_id = &disease_data.disease_id;
            let Some(ppkt_list) = ppkt_by_disease.get(disease_id) else {
                qc_report.no_ppkt_found(disease_id);
                continue;
            };
            let dqc = DiseaseQc::from_ppkt_list(&cohort.disease_list[0], cohort, ppkt_list);
            qc_report.extend_cohort_errors(dqc.check_moi());
            qc_report.extend_cohort_errors(dqc.check_acronym());
            qc_report.extend_cohort_errors(dqc.check_all_rows_output_as_ppkt());
            qc_report.extend_cohort_errors(dqc.check_no_hpo());
            match crate::hpo::duplets_need_update(hpo.clone(), &cohort.hpo_headers) {
                Ok(true) => {
                    qc_report.hpo_needs_version_update();
                }
                Ok(false) => {
                    // up to date — nothing to report
                }
                Err(e) => {
                    qc_report.ontology_error(&e);
                }
            }

        }
        Ok(qc_report)
    }

    fn derive_disease_id_from_cohort(cohort: &CohortData) -> Result<String, String> {
        if cohort.disease_list.len() != 1 {
            return Err(format!("Invalid disease count for {}: n={}", cohort.acronym(), cohort.disease_list.len()));
        }
        Ok(cohort.disease_list[0].disease_id.clone())
    }
    
}



