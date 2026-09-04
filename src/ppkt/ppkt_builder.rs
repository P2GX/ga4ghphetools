//! PhenopacketBuilder
//!
//! Shared logic for turning a [`CohortData`] into a set of GA4GH [`Phenopacket`]s.
//!
//! We need to create phenopackets anew from CohortData when the information is initially entered.
//! Also, during the Q/C process, we update the HPO term id/label of Cohorts if one or more terms used
//! in the cohort was updated during the review process of the HPO.
//! 
//! For the initial export, we need to provide the latest versions of the Ontology resources, as well as the biocurator
//! For the update, we we only want to change the version of the HPO as well as the headers (HPO ids/labels).
//!
//! # Implementors
//!
//! - [`PpktExporter`] — builds phenopackets from a [`CohortData`] using default or
//!   explicitly-supplied resource versions and a caller-provided ORCID. This is the
//!   "create from nothing" path.
//! - [`PpktUpdater`] — builds phenopackets from a (possibly revised) [`CohortData`],
//!   but sources the ORCID and every resource version *except HPO* from an existing
//!   [`Phenopacket`]'s `MetaData`, so re-running extraction after an HPO release
//!   doesn't silently change provenance it wasn't asked to change. The HPO version
//!   always comes from the `hpo: Arc<FullCsrOntology>` passed in, since updating
//!   that version is the entire point of this path. 
use std::{collections::HashMap, str::FromStr, sync::{Arc, LazyLock}};

use ontolius::{Identified, TermId, ontology::{HierarchyQueries, MetadataAware, OntologyTerms, csr::FullCsrOntology}, term::MinimalTerm};
use phenopacket_tools::builders::{builder::Builder, resources::Resources};
use phenopackets::schema::v2::{Phenopacket, core::{Disease, ExternalReference, Individual, KaryotypicSex, MetaData, OntologyClass, PhenotypicFeature, Sex, VitalStatus, vital_status::Status}};
use regex::Regex;

use phenopacket_tools::builders::time_elements::time_element_from_str;
use crate::{dto::{cohort_dto::{CohortData, DiseaseData, RowData}, hpo_term_dto::HpoTermDuplet}, ppkt::ppkt_variant_exporter::PpktVariantExporter};

static CLINICAL_MODIFIER: LazyLock<TermId> = LazyLock::new(|| {
    "HP:0012823".parse().expect("Failed to parse static HP:0012823")
});

/// All valid severity terms
static SEVERITY_MAP: LazyLock<HashMap<String, OntologyClass>> = LazyLock::new(||{
    let mut smap = HashMap::new();
    for (label, id) in [
        ("Borderline","HP:0012827"),
        ("Mild", "HP:0012825"),
        ("Moderate", "HP:0012826"),
        ("Severe", "HP:0012828"),
        ("Profound", "HP:0012829")
    ] {
        smap.insert(id.to_string(), OntologyClass{id: id.to_string(), label: label.to_string()});
    }
    smap
});

// in the same module as `pub trait PhenopacketBuilder`

/// Fallback HGNC resource version used when no explicit version is supplied.
pub(crate) const DEFAULT_HGNC_VERSION: &str = "06/01/25";
/// Fallback OMIM resource version used when no explicit version is supplied.
pub(crate) const DEFAULT_OMIM_VERSION: &str = "06/01/25";
/// Fallback GENO resource version used when no explicit version is supplied.
pub(crate) const DEFAULT_GENO_VERSION: &str = "2025-07-25";
/// Fallback Sequence Ontology (SO) resource version used when no explicit
/// version is supplied. Used by [`PpktExporter::new`] as the default for new
/// phenopackets, and by [`PpktUpdater::from_existing`] when an existing
/// phenopacket has no SO resource to carry forward (SO is only ever added
/// for rows that reference a structural variant, so its absence is expected
/// for many existing phenopackets).
pub(crate) const DEFAULT_SO_VERSION: &str = "2024-11-18";

pub trait PhenopacketBuilder {
    // --- required ---
    fn hpo(&self) -> &Arc<FullCsrOntology>;
    fn cohort(&self) -> &CohortData;
    fn disease_id_map(&self) -> &HashMap<String, DiseaseData>;
    fn orcid(&self) -> &str;
    fn geno_version(&self) -> &str;
    fn omim_version(&self) -> &str;
    fn hgnc_version(&self) -> &str;
    fn so_version(&self) -> &str;


    fn hpo_version(&self) -> &str {
        self.hpo().version()
    }

    fn has_sequence_ontology(&self, ppkt_row: &RowData) -> bool {
        ppkt_row.allele_count_map.keys()
            .any(|allele| self.cohort().structural_variants.contains_key(allele))
    }

    fn get_meta_data(&self, row_dto: &RowData) -> Result<MetaData, String> {
        let mut meta_data = Builder::meta_data_now(self.orcid().to_string());
        meta_data.resources.push(Resources::hpo_version(self.hpo_version()));
        meta_data.resources.push(Resources::geno_version(self.geno_version()));
        meta_data.resources.push(Resources::omim_version(self.omim_version()));
        meta_data.resources.push(Resources::hgnc_version(self.hgnc_version()));
        if self.has_sequence_ontology(row_dto) {
            meta_data.resources.push(Resources::so_version(self.so_version()));
        }
        meta_data.external_references.push(ExternalReference {
            id: row_dto.individual_data.pmid.clone(),
            reference: String::default(),
            description: row_dto.individual_data.title.clone(),
        });
        Ok(meta_data)
    }

   
    /// Create a GA4GH Individual message
    fn extract_individual(&self, ppkt_row: &RowData) -> Result<Individual, String> {
        let individual_dto = &ppkt_row.individual_data;
        let mut idvl = Individual{ 
            id: individual_dto.individual_id.clone(), 
            alternate_ids: vec![], 
            date_of_birth: None, 
            time_at_last_encounter: None, 
            vital_status: None, 
            sex: Sex::UnknownSex.into(), 
            karyotypic_sex: KaryotypicSex::UnknownKaryotype.into(), 
            gender: None, 
            taxonomy: None };
        match individual_dto.sex.as_ref() {
            "M" => idvl.sex = Sex::Male.into(),
            "F" => idvl.sex = Sex::Female.into(),
            "O" => idvl.sex = Sex::OtherSex.into(),
            "U" => idvl.sex = Sex::UnknownSex.into(),
            _ => { return Err(format!("Did not recognize sex string '{}' for '{}' ({})", idvl.sex, idvl.id, ppkt_row.individual_data.pmid)); }
        };
        let last_enc = &individual_dto.age_at_last_encounter;
        if last_enc != "na" {
            let age = time_element_from_str(last_enc)
                .map_err(|e| format!("malformed time_element for last encounter '{}':{} for {}",last_enc, e.to_string(), idvl.id))?;
            idvl.time_at_last_encounter = Some(age);
        }
        if individual_dto.deceased == "yes" {
            idvl.vital_status = Some(VitalStatus{ 
                status: Status::Deceased.into(), 
                time_of_death: None, 
                cause_of_death: None, 
                survival_time_in_days: 0 
            });
        } 
        Ok(idvl)

    }
    
    /// Derives a Phenopacket id from a row's PMID and individual identifier.
    ///
    /// The result is safe to use as a filename and satisfies the Phenopacket
    /// Schema's id constraints: only ASCII letters, digits, and single
    /// underscores as separators, with no leading/trailing or repeated
    /// underscores.
    ///
    /// # Algorithm
    /// 1. Replace `:` in the PMID with `_` (e.g. `PMID:12345` → `PMID_12345`).
    /// 2. Replace spaces in the individual id with `_`.
    /// 3. Join the two with `_`.
    /// 4. Replace every character that is not an ASCII letter or digit
    ///    (punctuation, whitespace, non-ASCII characters, etc.) with `_`.
    /// 5. Collapse any run of consecutive underscores into a single `_`.
    /// 6. Strip a trailing `_`, if the sanitization left one.
    ///
    /// # Note
    /// This id is derived solely from PMID + individual id, so two rows sharing
    /// both values will collide (identical output). The cohort loader is
    /// expected to guarantee that combination is unique; this function does not
    /// itself check or enforce uniqueness across rows.
    fn get_phenopacket_id(&self, ppkt_row: &RowData) -> String {
        let individual_dto = &ppkt_row.individual_data;
        let pmid = ppkt_row.individual_data.pmid.replace(":", "_");
        let individual_id = individual_dto.individual_id.replace(" ", "_");
        let ppkt_id = format!("{}_{}", pmid, individual_id);
        let ppkt_id = ppkt_id.replace("__", "_");
        // Collapse any character that is not an ASCII letter or digit to `_`
        // (this includes punctuation like ':' or '.', whitespace, and any
        // non-ASCII characters — not just non-ASCII ones).
        let mut sanitized: String = ppkt_id.chars()
            .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
            .clone().collect();
        // Collapse multiple underscores, if any
        let re = Regex::new(r"_+").unwrap();
        sanitized = re.replace_all(&sanitized, "_").to_string();
        if let Some(stripped) = sanitized.strip_suffix('_') {
            sanitized = stripped.to_string();
        }
        sanitized
    }
    
    /// Builds the row's disease list from its `disease_id_list`.
    ///
    /// Each disease id is looked up in [`disease_id_map`](Self::disease_id_map)
    /// to fill in the disease's ontology label; every other `Disease` field
    /// (stage, TNM finding, primary site, laterality, resolution) is left
    /// unset — this function currently only populates `term`, `excluded`, and
    /// (conditionally) `onset`.
    ///
    /// # Onset
    /// The row's `individual_data.age_of_onset` is attached as `onset` **only
    /// when the row lists exactly one disease**. With multiple diseases there is
    /// no way to know which disease the recorded onset age belongs to, so onset
    /// is left `None` for every disease in that case rather than guessing. A
    /// literal age-of-onset value of `"na"` is treated as "not recorded" and
    /// also leaves `onset` as `None`.
    ///
    /// # Errors
    /// Returns an error if:
    /// - `ppkt_row.disease_id_list` is empty, or
    /// - any listed disease id is not present in
    ///   [`disease_id_map`](Self::disease_id_map), or
    /// - the row has a single disease and a non-`"na"` `age_of_onset` that
    ///   isn't a valid ISO 8601 age/duration string.
    ///
    /// On error, no partial `Disease` list is returned — extraction stops at
    /// the first failing entry.
    fn get_disease_list(&self, ppkt_row: &RowData) -> Result<Vec<Disease>, String> {
        let disease_id_list = &ppkt_row.disease_id_list;
        if disease_id_list.is_empty() {
            return Err("No disease data found".to_string());
        }
        let has_multiple_dx = disease_id_list.len() > 1;
        let mut disease_list: Vec<Disease> = Vec::new();
        for dx_id in disease_id_list {
            let d_data = self.disease_id_map().get(dx_id)
                .ok_or_else(|| format!("Disease with id {} not found", dx_id))?;
            let dx_clz = OntologyClass { 
                id:d_data.disease_id.clone(), 
                label: d_data.disease_label.clone()
            };
            let mut disease = Disease{ 
                term: Some(dx_clz), 
                excluded: false, 
                onset: None, 
                resolution: None, 
                disease_stage: vec![], 
                clinical_tnm_finding: vec![], 
                primary_site: None, 
                laterality: None 
            };
            // If we have multiple diseases, we cannot automatically say when the disease onset was (which disease has the earliest onset)
            if ! has_multiple_dx {
                let idl_dto = ppkt_row.individual_data.individual_id.clone();
                let onset = &ppkt_row.individual_data.age_of_onset;
                if onset != "na" {
                    let age = time_element_from_str(onset)
                        .map_err(|e| format!("malformed time_element for onset '{}': {}", onset, e.to_string()))?;
                    disease.onset = Some(age);
                };
            }
            disease_list.push(disease);
        }
        Ok(disease_list)
    }



    /// Validates an HPO term id/label pair against the ontology and converts it
    /// to a Phenopacket `OntologyClass` (checking for matches with latest HPO version)
    ///
    /// # Errors
    /// Returns an error if:
    /// - `term.hpo_id()` is not a syntactically valid `TermId`, or
    /// - the id is not present in [`hpo()`](Self::hpo) at all, or
    /// - the id is a secondary/obsolete id — i.e. `hpo()` resolves it to a term
    ///   whose primary identifier differs from `hpo_id` (the error message
    ///   includes that current primary id), or
    /// - the label is not correct
    /// - the final `OntologyClass` fails to build (in practice unreachable here,
    ///   since `hpo_id`/`hpo_label` were already taken from `term` and the id
    ///   already parsed successfully above).
    fn get_ontology_class(&self, term: &HpoTermDuplet) -> Result<OntologyClass, String> {
        let hpo_id = term.hpo_id();
        let hpo_label = term.hpo_label();
        let hpo_term_id = TermId::from_str(hpo_id).map_err(|e| e.to_string())?;
        let hpo_term = match self.hpo().term_by_id(&hpo_term_id) {
            Some(term) => term.clone(),
            None => {
                return Err(format!("Could not find HPO term for {hpo_id}"));
            }
        };
        if hpo_term.identifier() != &hpo_term_id {
            return Err(format!("{} is not the primary id ({}) for {}",
                hpo_term_id, hpo_term.identifier(), hpo_label));
        }
        if hpo_term.name() != hpo_label {
            return Err(format!("{} is not the correct label ({}) for {}",
                hpo_label, hpo_term.identifier(), hpo_label));
        }
        let hpo_term = Builder::ontology_class(term.hpo_id(), term.hpo_label())
                .map_err(|e| format!("termid_parse_error '{:?}'", term))?;
        Ok(hpo_term)
    }


    

    fn get_phenopacket_features(&self, ppkt_row: &RowData) -> Result<Vec<PhenotypicFeature>, String> {
        let hpo_term_list = &self.cohort().hpo_headers;
        let hpo_data = &ppkt_row.hpo_data;
        if hpo_data.len() != hpo_term_list.len() {
            return Err(format!("Length of HPO headers ({}) does not match length of HPO values {}",
            hpo_term_list.len(), hpo_data.len()));
        }
        let mut ppkt_feature_list: Vec<PhenotypicFeature> = Vec::with_capacity(hpo_data.len());
        for (term, cell_contents) in hpo_term_list.iter().zip(hpo_data.iter()) {
            if ! cell_contents.is_ascertained() {
                continue;
            }
            let hpo_term = self.get_ontology_class(term)?;
            let mut pf = PhenotypicFeature{ 
                description: String::default(), 
                r#type: Some(hpo_term), 
                excluded: cell_contents.is_excluded(), 
                severity: None, 
                modifiers: vec![], 
                onset: None,
                resolution: None, 
                evidence: vec![]
            };
            if cell_contents.has_onset() {
                let ost = time_element_from_str(&cell_contents.entry())
                    .map_err(|e| format!("malformed time_element for cell '{}': {}", cell_contents, e.to_string()))?;
                pf.onset = Some(ost);
            }
            if cell_contents.has_modifier() {
                let mut mod_list: Vec<OntologyClass> = Vec::new();
                for mod_str in cell_contents.modifers() {
                    let mod_id: TermId = mod_str.parse().map_err(|_| format!("Could not create TermId from modifier String '{mod_str}'"))?;
                    let term = self.hpo().term_by_id(&mod_id).ok_or_else(|| {
                        format!("Could not retrieve Modifier term for id '{mod_id}' for {}", ppkt_row.individual_data.pmid)
                    })?;
                    if let Some(severity_term) = SEVERITY_MAP.get(mod_str) {
                        pf.severity = Some(severity_term.clone());
                    } else if self.hpo().is_descendant_of(term.identifier(), &*CLINICAL_MODIFIER) {
                        let label = term.name().to_string();
                        let oclass =  OntologyClass {
                            id: label,
                            label: mod_str.clone(),
                        };
                        mod_list.push(oclass);
                    }
                }
                pf.modifiers = mod_list;
            }
            ppkt_feature_list.push(pf);
        }
        Ok(ppkt_feature_list)
    }


    fn extract_phenopacket_from_row(&self, ppkt_row_dto: &RowData) -> Result<Phenopacket, String> {
        let is_male = &ppkt_row_dto.individual_data.sex == "M";
        let ppkt_var_exporter = PpktVariantExporter::new(is_male, self.cohort());
        let interpretation_list = ppkt_var_exporter.get_interpretation_list(ppkt_row_dto)?;
        Ok(Phenopacket {
            id: self.get_phenopacket_id(ppkt_row_dto),
            subject: Some(self.extract_individual(ppkt_row_dto)?),
            phenotypic_features: self.get_phenopacket_features(ppkt_row_dto)?,
            measurements: vec![],
            biosamples: vec![],
            interpretations: interpretation_list,
            diseases: self.get_disease_list(ppkt_row_dto)?,
            medical_actions: vec![],
            files: vec![],
            meta_data: Some(self.get_meta_data(ppkt_row_dto)?),
        })
    }

    fn get_all_phenopackets(&self) -> Result<Vec<Phenopacket>, String> {
        self.cohort().rows.iter()
            .map(|row| self.extract_phenopacket_from_row(row))
            .collect()
    }
}