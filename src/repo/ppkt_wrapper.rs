use std::path::{Path, PathBuf};

use phenopackets::schema::v2::Phenopacket;


#[derive(Clone, Debug)]
pub(crate) struct  PpktWrapper {
    /// Path to location of file with the current phenopacket
    pub path: PathBuf,
    /// OMIM identifier, e.g., OMIM:654321, of the current disease (only Mendelian is supported, blended with throw an error)
    pub disease_id: String,
    /// The GA4GH version 2 Phenopacket
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