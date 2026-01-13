use std::path::Path;

/// Get the file name at the end of the path
/// If there is any error, return the original path
pub(crate) fn extract_file_name(input_path: &str) -> String {
    let path = Path::new(input_path);
    if let Some(file_name_os) = path.file_name() {
        if let Some(file_name) = file_name_os.to_str() {
            return file_name.to_string();
        }       
    }
   return input_path.to_string();
}