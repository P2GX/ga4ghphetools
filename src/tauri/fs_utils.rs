use tauri::{AppHandle, Emitter};
use tauri_plugin_dialog::{DialogExt, FilePath};
use std::future::Future;
use ontolius::{io::OntologyLoaderBuilder, ontology::csr::FullCsrOntology};
use std::sync::Arc;
use crate::tauri::models::OntologyLoadEvent;

/// Converts a `tauri_plugin_fs::FilePath` into a `String` representing the full path.
///
/// # Arguments
///
/// * `file_path` - The `FilePath` object returned by Tauri's file picker.
///
/// # Returns
///
/// * `Ok(String)` - The full path as a UTF-8 string (lossily converted if necessary).
/// * `Err(String)` - An error message if the path is missing.
///
/// # Example
///
/// ```rust
/// let full_path = get_full_path_as_str(file_path)?;
/// println!("Selected file path: {}", full_path);
/// ```
fn get_full_path_as_str(file_path: FilePath) -> Result<String, String> {
    let path = file_path
        .as_path()
        .ok_or_else(|| "Failed to extract system path from FilePath entry".to_string())?;

    Ok(path.to_string_lossy().to_string())
}



/// Prompts the user to select a file using the native system dialog on an asynchronous 
/// background thread, extracting its absolute platform path before passing it to a worker closure.
///
/// This utility centralizes the boilerplate required to manage Tauri's blocking file dialog, 
/// process `FilePath` variables safely into native `String` routes, and emit standard 
/// error payloads back across the frontend communication core if extraction fails.
///
/// # Type Parameters
///
/// * `F` - The async closure handler that processes the verified path string.
/// * `Fut` - The future returned by the closure, executed on the spawned worker pool.
///
/// # Arguments
///
/// * `app` - An active [`AppHandle`] instance used to spin up file dialogs and emit event channel updates.
/// * `channel_name` - The identifier string used to broadcast JSON payloads back to the frontend listener.
/// * `on_success` - A lifecycle callback triggered precisely once when a file path is successfully resolved.
///
/// # Examples
///
/// ```rust
/// use tauri::AppHandle;
/// 
/// #[tauri::command]
/// async fn load_custom_data(app: AppHandle) -> Result<(), String> {
///     pick_file_and_process(app, "data-load-channel", move |file_path, app_handle| async move {
///         println!("Processing valid system file path: {}", file_path);
///         // Business processing logic runs here safely on a worker thread...
///     });
///     Ok(())
/// }
/// ```
pub fn pick_file_and_process<F, Fut>(
    app: AppHandle, 
    channel_name: &'static str, 
    on_success: F
) 
where
    F: FnOnce(String, AppHandle) -> Fut + Send + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    let app_handle = app.clone();

    tauri::async_runtime::spawn(async move {
        match app_handle.dialog().file().blocking_pick_file() {
            Some(file) => {
                match get_full_path_as_str(file) {
                    Ok(valid_path_str) => {
                        on_success(valid_path_str, app_handle).await;
                    },
                    Err(e) => {
                        let _ = app_handle.emit(channel_name, OntologyLoadEvent::error(e));
                    }
                }
            },
            None => {
                let _ = app_handle.emit(channel_name, OntologyLoadEvent::cancel());
            },
        };
    });
}




pub fn load_ontology(json_path: &str) -> Result<Arc<FullCsrOntology>, Box<dyn std::error::Error>> {
    let loader = OntologyLoaderBuilder::new().obographs_parser().build();
    let onto: FullCsrOntology = loader.load_from_path(json_path)?;
    Ok(Arc::new(onto))
}
