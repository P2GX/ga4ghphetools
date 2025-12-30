


/// Represents a directory for one gene with all of the contained files.
use std::{fs::{self, File}, io::BufReader, path::PathBuf};
use serde_json::{Value, json};
use phenopackets::schema::v2::Phenopacket;
use walkdir::WalkDir;
use std::path::Path;

use crate::{dto::cohort_dto::CohortData, repo::cohort_qc::CohortQc};
/// Represents a directory for one gene with all contained files and metadata.
#[derive(Debug, Default)]
pub struct CohortDir {
    pub cohort_name: String,
    /// Path of the directory corresponding to cohort_name (e.g., a gene smbol such as ZRSR2)
    pub path: PathBuf,
    /// The specific file or files like "ZRSR2_OFD21_individuals.json" (there must be at least one but can be many)
    pub individuals_json: Vec<PathBuf>,
    /// All JSON files inside the 'phenopackets' subdirectory (there must be at least one)
    pub phenopackets: Vec<PathBuf>,
    /// Any files or directories that don't belong in the standard structure
    pub unexpected_entries: Vec<PathBuf>,
}


impl CohortDir {
    

pub fn process_gene_directory(path: &Path) -> CohortDir {
    let mut gene_dir = CohortDir {
        cohort_name: path.file_name().unwrap_or_default().to_string_lossy().into(),
        path: path.to_path_buf(),
        ..Default::default()
    };

    // Iterate through the immediate children of the gene directory
    for entry in WalkDir::new(path).min_depth(1).max_depth(1).into_iter().filter_map(|e| e.ok()) {
        let file_name = entry.file_name().to_string_lossy();

        if entry.file_type().is_dir() && file_name == "phenopackets" {
            // Recurse into phenopackets
            gene_dir.phenopackets = WalkDir::new(entry.path())
                .min_depth(1)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file())
                .map(|e| e.into_path())
                .collect();
        } else if entry.file_type().is_file() && file_name.ends_with("_individuals.json") {
            gene_dir.individuals_json.push(entry.into_path());
        } else {
            // Anything else (odd files, extra folders) is flagged
            gene_dir.unexpected_entries.push(entry.into_path());
        }
    }

    gene_dir
}

    
    fn read_cohort(path: &PathBuf) -> Result<CohortData, String> {
        let file_data = fs::read_to_string(path.clone())
            .map_err(|e| 
                format!("Could not extract string data from {}: {}", path.to_string_lossy(), e.to_string())).unwrap();
        let cohort: CohortData = serde_json::from_str(&file_data)
            .map_err(|e| format!("Could not transform string {} to CohortDto: {}",
                file_data, e.to_string()))?;
        Ok(cohort)
    }
    
    pub fn get_cohort_data(&self) -> Result<Vec<CohortData>, String> {
        let mut cohorts: Vec<CohortData> = Vec::new();
        for pth in &self.individuals_json {
            let cohort = Self::read_cohort(pth)?;
            cohorts.push(cohort);
        }
        Ok(cohorts)
    }

    fn load_phenopacket<P: AsRef<Path>>(path: P) -> Result<Phenopacket, String> {
        let file = File::open(path).map_err(|e| e.to_string())?;
        let reader = BufReader::new(file);
        
        // 1. Parse into a flexible JSON Value first
        let mut v: Value = serde_json::from_reader(reader).map_err(|e| e.to_string())?;

        // 2. Patch the specific fields causing the panic
        Self::patch_missing_defaults(&mut v);

        // 3. Deserialize the patched Value into your actual Struct
        let phenopacket: Phenopacket = serde_json::from_value(v)
            .map_err(|e| format!("Schema error after patching: {}", e))?;

        Ok(phenopacket)
    }


    fn patch_missing_defaults(v: &mut Value) {
        if let Some(obj) = v.as_object_mut() {
            // Handle "vitalStatus" fields
            if let Some(vs) = obj.get_mut("vitalStatus").and_then(|vs| vs.as_object_mut()) {
                // Field 1: survivalTimeInDays
                vs.entry("survivalTimeInDays").or_insert_with(|| json!(0));
                
                // Field 2: status (If it's missing, default to the 0-variant)
                vs.entry("status").or_insert_with(|| json!("UNKNOWN_STATUS"));
            }

            // Recurse to find vitalStatus inside nested structures (Families, etc.)
            for child in obj.values_mut() {
                Self::patch_missing_defaults(child);
            }
        } else if let Some(array) = v.as_array_mut() {
            for item in array {
                Self::patch_missing_defaults(item);
            }
        }
    }

    pub fn get_phenopackets(&self) -> Result<Vec<Phenopacket>, String> {
        let mut ppkt_list = Vec::new();
        for ppkt_path in &self.phenopackets {
            let ppkt = Self::load_phenopacket(ppkt_path)?;
            ppkt_list.push(ppkt);
        }
        Ok(ppkt_list)
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

    pub fn get_cohort_qc(&self) -> Result<CohortQc, String> {
        let cohorts = self.get_cohort_data()?;
        let phenopackets = self.get_phenopackets()?;
        let unexpected_files = self.get_unexpected_file_names();
        CohortQc::new(&self.cohort_name, cohorts, phenopackets, unexpected_files)
    }
    
}