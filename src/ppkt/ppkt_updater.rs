use std::{collections::HashMap, sync::Arc};

use ontolius::ontology::csr::FullCsrOntology;
use phenopackets::schema::v2::{Phenopacket, core::Resource};

use crate::{dto::cohort_dto::{CohortData, DiseaseData}, ppkt::ppkt_builder::{DEFAULT_SO_VERSION, PhenopacketBuilder}};

pub struct PpktUpdater {
    hpo: Arc<FullCsrOntology>,
    cohort_dto: CohortData,
    disease_id_map: HashMap<String, DiseaseData>,
    orcid_id: String,
    geno_version: String,
    omim_version: String,
    hgnc_version: String,
    so_version: String,
    existing: Phenopacket,
}

impl PpktUpdater {
    /// Re-derive a Phenopacket from `cohort`, but source the creator ORCID and every
    /// non-HPO resource version from `existing`'s MetaData rather than fresh defaults.
    /// The HPO version always comes from `hpo` (the whole point of the update).
    pub fn from_existing(
        hpo: Arc<FullCsrOntology>,
        cohort: &CohortData,
        existing: &Phenopacket,
    ) -> Result<Self, String> {
        let meta = existing.meta_data.as_ref()
            .ok_or_else(|| "existing Phenopacket is missing MetaData".to_string())?;
        
        let orcid_id = meta.created_by.clone(); // confirm exact field name in your generated types

        let mut disease_id_map = HashMap::new();
        for d in &cohort.disease_list {
            disease_id_map.insert(d.disease_id.clone(), d.clone());
        }

        let geno_version = Self::find_resource_version(&meta.resources, "geno")?;
        let omim_version = Self::find_resource_version(&meta.resources, "omim")?;
        let hgnc_version = Self::find_resource_version(&meta.resources, "hgnc")?;
        // SO is only added conditionally in get_meta_data, so don't hard-fail if it's absent
        let so_version = Self::find_resource_version(&meta.resources, "so")
            .unwrap_or_else(|_| DEFAULT_SO_VERSION.to_string());

        Ok(Self { 
            hpo, 
            cohort_dto: cohort.clone(), 
            disease_id_map, 
            orcid_id, 
            geno_version, 
            omim_version, 
            hgnc_version, 
            so_version,
            existing: existing.clone()
         })
    }

    fn find_resource_version(resources: &[Resource], namespace_prefix: &str) -> Result<String, String> {
        resources.iter()
            .find(|r| r.namespace_prefix.eq_ignore_ascii_case(namespace_prefix))
            .map(|r| r.version.clone())
            .ok_or_else(|| format!("Resource '{namespace_prefix}' not found in existing phenopacket's MetaData"))
    }

    pub fn get_ppkt(&self) -> Phenopacket {
        self.existing.clone()
    }
}

impl PhenopacketBuilder for PpktUpdater {
    fn hpo(&self) -> &Arc<FullCsrOntology> { &self.hpo }
    fn cohort(&self) -> &CohortData { &self.cohort_dto }
    fn disease_id_map(&self) -> &HashMap<String, DiseaseData> { &self.disease_id_map }
    fn orcid(&self) -> &str { &self.orcid_id }
    fn geno_version(&self) -> &str { &self.geno_version }
    fn omim_version(&self) -> &str { &self.omim_version }
    fn hgnc_version(&self) -> &str { &self.hgnc_version }
    fn so_version(&self) -> &str { &self.so_version }
}