use std::path::PathBuf;

use crate::persistence::dir_manager::DirManager;


pub mod dir_manager;



///  Open the directory at the indicated location; if it does not exist, create it.
/// This module may no longer be necessary after refactoring TODO-consider
pub fn initialize_project_dir(project_dir: PathBuf) -> Result<(), String> {
    let dirman = DirManager::new(project_dir)?;
    Ok(())
}
