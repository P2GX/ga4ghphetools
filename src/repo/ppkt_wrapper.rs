use std::path::{Path, PathBuf};

use phenopackets::schema::v2::Phenopacket;


#[derive(Clone)]
pub(crate) struct  PpktWrapper {
    pub path: PathBuf,
    pub disease_id: String,
    pub ppkt: Phenopacket,
}

impl PpktWrapper {
    pub fn new(
        path: &Path,
        disease_id: impl Into<String>,
        ppkt: Phenopacket) -> Self {
            Self {
                path: path.to_path_buf(),
                disease_id: disease_id.into(),
                ppkt
            }
    }
}