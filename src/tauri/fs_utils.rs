use tauri_plugin_fs::FilePath;

/// Converts a generic Tauri file-picker `FilePath` into a standard `String`.
pub fn get_full_path_as_str(file_path: FilePath) -> Result<String, String> {
    let path = file_path
        .as_path()
        .ok_or_else(|| "Failed to extract system path from FilePath entry".to_string())?;

    Ok(path.to_string_lossy().to_string())
}