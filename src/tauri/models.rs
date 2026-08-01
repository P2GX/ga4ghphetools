use serde::{Serialize, Deserialize};


/// OntologyLoadEvent is designed to be used with tauri User Interfaces
/// Upon successful/failed loading of an ontology (e.g., HPO), a signal
/// is emitting from the Rust backend to the front end (e.g., Angular) that
/// has corresponding listeners.
/// OntologyLoadEvent is not coupled to the functions in the library for flexibility
/// It can be used in applications as follows
/// ``ìgnore
/// use ga4ghphetools::tauri::{pick_file_and_process, load_ontology, OntologyLoadEvent};
/// #[tauri::command]
/// async fn load_hpo(
///    app: AppHandle,
///    state: tauri::State<'_, Arc<AppState>>,
/// ) -> Result<(), String> {
///    let state_handle = state.inner().clone();
///    let _ = app.emit("hpo-load-event", OntologyLoadEvent::loading());
///    pick_file_and_process(app, "hpo-load-event", move |hpo_json_path, app_handle| async move {
///        match load_ontology(&hpo_json_path) {
///            Ok(ontology) => {
///                let mut singleton = state_handle.phenoblendtk.lock().unwrap();
///                let n_terms = ontology.len();
///                singleton.set_hpo(ontology, &hpo_json_path);
///                let _ = app_handle.emit(
///                    "hpo-load-event", 
///                    OntologyLoadEvent::success("HPO loaded".to_string(), n_terms)
///                );
///            },
///            Err(e) => { 
///                let _ = app_handle.emit("hpo-load-event", OntologyLoadEvent::error(e.to_string()));
///            }
///        }
///    });
///    Ok(())
///}
/// ```
///
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", content = "payload", rename_all = "camelCase")]
#[serde(rename_all_fields = "camelCase")]
pub enum OntologyLoadEvent {
    Loading,
    Success { 
        version: String,
        term_count: usize,
    },
    Error { error_message: String },
    Cancel,
}

impl OntologyLoadEvent {
    pub fn loading() -> Self {
        Self::Loading
    }

    // If we successfully parse the ontology, record the version and the term count!
    pub fn success(ontology_version: impl Into<String>, term_count: usize) -> Self {
        Self::Success { 
            version: ontology_version.into(),
            term_count,
        }
    }

    pub fn error(msg: impl Into<String>) -> Self {
        Self::Error { error_message: msg.into() }
    }

    pub fn cancel() -> Self {
        Self::Cancel
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HpoTermMinimal {
    pub term_id: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct HierarchyMapItem {
    pub current_term_id: String,
    pub parents: Vec<HpoTermMinimal>,
    pub children: Vec<HpoTermMinimal>,
}