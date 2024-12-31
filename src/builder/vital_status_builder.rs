use crate::generated;
use generated::org::phenopackets::schema::v2::core::OntologyClass as OntologyClass202;
use crate::generated::org::phenopackets::schema::v2::core::TimeElement as TimeElement202;
use generated::org::phenopackets::schema::v2::core::VitalStatus as VitalStatus202;
use generated::org::phenopackets::schema::v2::core::vital_status::Status as VsStatus202;



pub struct VitalStatusBuilder {
    status: VsStatus202,
    time_of_death: Option<TimeElement202>,
    cause_of_death: Option<OntologyClass202>,
    survival_time_in_days: u32,
    
}


impl VitalStatusBuilder {

    pub fn new(status: VsStatus202) -> Self {
        VitalStatusBuilder {
            status: status,
            time_of_death: None,
            cause_of_death: None,
            survival_time_in_days:0
        }
    }

    pub fn alive() -> VitalStatusBuilder {
        VitalStatusBuilder::new(VsStatus202::Alive)
    }

    pub fn deceased() -> VitalStatusBuilder {
        VitalStatusBuilder::new(VsStatus202::Deceased)
    }

    pub fn time_of_death(&mut self, tod: TimeElement202) -> &Self {
        self.time_of_death = Some(tod);
        self
    }

    pub fn cause_of_death(&mut self, cause: OntologyClass202) -> &Self {
        self.cause_of_death = Some(cause);
        self
    }

    pub fn survival_time_in_days(&mut self, days: u32) -> &Self {
        self.survival_time_in_days = days;
        self
    }

    pub fn build(self) -> Result<VitalStatus202, &'static str> {
        return Ok(VitalStatus202 {
            status: self.status as i32,
            time_of_death: self.time_of_death,
            cause_of_death: self.cause_of_death,
            survival_time_in_days:0
        });
    }
    
}