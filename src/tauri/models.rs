// src/tauri/models.rs
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", content = "payload", rename_all = "camelCase")]
pub enum OntologyLoadEvent {
    Loading,
    Success { status_message: String },
    Error { error_message: String },
    Cancel,
}

impl OntologyLoadEvent {
    pub fn loading() -> Self {
        Self::Loading
    }

    pub fn success(msg: String) -> Self {
        Self::Success { status_message: msg }
    }

    pub fn error(msg: String) -> Self {
        Self::Error { error_message: msg }
    }

    pub fn cancel() -> Self {
        Self::Cancel
    }
}