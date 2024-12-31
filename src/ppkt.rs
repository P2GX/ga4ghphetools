use std::process::id;

use crate::age::{Age, AgeTrait};
use crate::builder::individual_builder::IndividualBuilder;
use crate::builder::time_elements::TimeElements;
use crate::builder::vital_status_builder::VitalStatusBuilder;
use crate::{generated, individual_template::IndividualTemplate};

use generated::org::phenopackets::schema::v2::core::Individual as Individual202;
use generated::org::phenopackets::schema::v2::core::TimeElement as TimeElement202;
use generated::org::phenopackets::schema::v2::core::time_element::Element::OntologyClass as toc;
use generated::org::phenopackets::schema::v2::core::OntologyClass as OntologyClass202;
use generated::org::phenopackets::schema::v2::core::Age as Age202;
use polars::io::is_cloud_url;



pub struct Ppkt {
    my_ppkt: generated::org::phenopackets::schema::v2::phenopackets::Phenopacket,
}


fn create_ga4gh_individual(idvl_tmpl: IndividualTemplate) -> Result<Individual202, String> {
    
    let mut indvl_builder = IndividualBuilder::new(idvl_tmpl.individual_id());
    if idvl_tmpl.sex().male() {
        indvl_builder.male();
    } else if idvl_tmpl.sex().female() {
        indvl_builder.female();
    } else if idvl_tmpl.sex().other_sex() {
        indvl_builder.other_sex();
    } 
    // TODO -- THE BELOW IS HACK, figure out better style
    let last_encounter =   match idvl_tmpl.age_at_last_encounter() {
        Some (age) => {
            match age {
                Age::Gestational(_ga) => None, // todo
                Age::HpoTerm(hpo) => {
                    indvl_builder.ontology_class_at_last_encounter(hpo.term_id(), hpo.label());
                    Some(TimeElements::ontology_class(hpo.term_id(), hpo.label()))
                },
                Age::Iso8601(iso) => {
                    indvl_builder.age_at_last_encounter(iso.age_string());
                    Some(TimeElements::age(iso.age_string()))
                } 
            }
        },
        None => None,
    };
    if idvl_tmpl.deceased().is_deceased() {
        let mut vstatb = VitalStatusBuilder::deceased();
        if last_encounter.is_some() {
            vstatb.time_of_death(last_encounter.unwrap());
        }
        indvl_builder.vital_status(vstatb.build().unwrap());
    } else if idvl_tmpl.deceased().is_alive() {
        indvl_builder.vital_status(VitalStatusBuilder::alive().build().unwrap());
    }
    indvl_builder.build()

}




impl Ppkt {

    pub fn new(idvl_tmpl: IndividualTemplate) -> Result<Self, String> {
        Err("Could not generate phenopacket".to_string())
    }
    
}