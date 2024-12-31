use crate::generated;

use generated::org::phenopackets::schema::v2::core::Individual as Individual202;
use generated::org::phenopackets::schema::v2::core::TimeElement as TimeElement202;
use generated::org::phenopackets::schema::v2::core::time_element::Element::OntologyClass as toc;
use generated::org::phenopackets::schema::v2::core::OntologyClass as OntologyClass202;
use generated::org::phenopackets::schema::v2::core::Age as Age202;
use generated::org::phenopackets::schema::v2::core::Sex as Sex202;
use generated::org::phenopackets::schema::v2::core::KaryotypicSex as KaryotypicSex202;
use generated::org::phenopackets::schema::v2::core::VitalStatus as VitalStatus202;
use prost_types::Timestamp;

use super::time_elements::TimeElements;


pub struct IndividualBuilder {
    id: String,
    alternate_ids: Vec<String>,
    data_of_birth: Option<Timestamp>,
    time_at_last_encounter: Option<TimeElement202>,
    vital_status: Option<VitalStatus202>,
    sex: Sex202,
    gender: Option<OntologyClass202>,
    karyotypic_sex: KaryotypicSex202,
    taxonomy: Option<OntologyClass202>
}

impl IndividualBuilder {
    pub fn new<S : Into<String>>(identifier: S) -> Self {
        IndividualBuilder {
            id: identifier.into(),
            alternate_ids: vec![],
            data_of_birth: None,
            time_at_last_encounter: None,
            vital_status: None,
            sex: Sex202::UnknownSex,
            gender: None,
            karyotypic_sex: KaryotypicSex202::UnknownKaryotype,
            taxonomy: None
        }
    }

    pub fn add_alternate_id(&mut self, alternate: impl Into<String>) -> &Self {
        self.alternate_ids.push(alternate.into());
        self
    }

    pub fn date_of_birth(&mut self, dob: Timestamp) -> &Self {
        self.data_of_birth = Some(dob);
        self
    }

    pub fn timestamp_at_last_encounter(mut self, last_enc: Timestamp) -> Self {
        self.time_at_last_encounter = Some(TimeElements::timestamp(last_enc));
        self
    }

    pub fn ontology_class_at_last_encounter<T: Into<String>,U: Into<String>>(
        mut self, 
        term_id: T,
        term_label: U) -> Self {
            self.time_at_last_encounter = Some(TimeElements::ontology_class(term_id,term_label));
            self
        }

    pub fn age_at_last_encounter<S: Into<String>>(&mut self, iso8601_str: S) -> &Self {
        self.time_at_last_encounter = Some(TimeElements::age(iso8601_str));
        self
    }

    pub fn male(&mut self) -> &Self {
        self.sex = Sex202::Male;
        self
    }

    pub fn female(&mut self) -> &Self {
        self.sex = Sex202::Female;
        self
    }

    pub fn other_sex(&mut self) -> &Self {
        self.sex = Sex202::OtherSex;
        self
    }

    pub fn unknown_sex(&mut self) -> &Self {
        self.sex = Sex202::UnknownSex;
        self
    }

    pub fn vital_status(&mut self, vstatus: VitalStatus202) -> &Self {
        self.vital_status = Some(vstatus);
        self
    }

    /// todo, are there ways this can faile if we get here?
    pub fn build(self) -> Result<Individual202, String> {
       
        Ok(Individual202 {
            id: self.id,
            alternate_ids: self.alternate_ids,
            date_of_birth: self.data_of_birth,
            time_at_last_encounter: self.time_at_last_encounter,
            vital_status: self.vital_status,
            sex: self.sex as i32,
            gender: self.gender,
            karyotypic_sex: self.karyotypic_sex as i32,
            taxonomy: self.taxonomy
        })
       
    }
}