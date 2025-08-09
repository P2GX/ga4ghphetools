
use std::{collections::HashMap, fs, path::{Path, PathBuf}};

pub struct DirManager {
    /// Path to the directory where we store the various files for the project (e.g., FBN1 for the FBN1 cohort)
    cache_dir_path: PathBuf,
}


impl DirManager {
    /// Open the directory at the indicated location; if it does not exist, create it.
    /// Once we have opened the directory, open or create the HGVS cache.
    pub fn new<P: AsRef<Path>>(dir_path: P) -> Result<Self, String> {
        let path_buf = dir_path.as_ref().to_path_buf();
        if !path_buf.exists() {
            fs::create_dir_all(&path_buf).map_err(|e| e.to_string())?;
        }
        if !path_buf.is_dir() {
            return Err(format!("Path exists but is not a directory: {:?}", path_buf));
        }
        Ok(Self {
            cache_dir_path: path_buf,
        })
    }

    /// Check an HGVS or structural variant.
    /// If we validate, we return the same DTO (except that the validated flag is set to true)
    /// The cause of any error is returned as a string.
   
    pub fn get_cohort_dir(&self) -> PathBuf {
        self.cache_dir_path.clone()
    }

 

 
}
