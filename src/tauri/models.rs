use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", content = "payload", rename_all = "camelCase")]
#[serde(rename_all_fields = "camelCase")]
pub enum OntologyLoadEvent {
    Loading,
    Success { 
        status_message: String,
        term_count: usize,
    },
    Error { error_message: String },
    Cancel,
}

impl OntologyLoadEvent {
    pub fn loading() -> Self {
        Self::Loading
    }

    // Update the constructor to take the term count
    pub fn success(msg: impl Into<String>, term_count: usize) -> Self {
        Self::Success { 
            status_message: msg.into(),
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