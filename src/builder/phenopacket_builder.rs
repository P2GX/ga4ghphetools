use crate::generated;
use generated::org::phenopackets::schema::v2::core::Individual as Individual202;
use generated::org::phenopackets::schema::v2::Phenopacket as Phenopacket202;


struct PhenopacketBuilder {
    id: String,
    subject: Individual202,
}

impl PhenopacketBuilder {

    pub fn new<S: Into<String>>(identifier: S) -> PhenopacketBuilder {
        PhenopacketBuilder {
            id: identifier.into(),
            subject: None
        }
    }

    pub fn subject(mut self, subj: Individual202) -> Self {
        self.subject = subj;
        self
    }


    pub fn build(self) -> Phenopacket202 {
        Phenopacket202 {
            id: self.id,
            subject: self.subject,
        }
    }
    
}