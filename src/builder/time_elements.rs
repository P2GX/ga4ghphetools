
use crate::generated;
use prost_types::Timestamp;
use generated::org::phenopackets::schema::v2::core::Age as Age202;
use generated::org::phenopackets::schema::v2::core::OntologyClass as OntologyClass202;
use generated::org::phenopackets::schema::v2::core::TimeElement as TimeElement202;

use generated::org::phenopackets::schema::v2::core::time_element::Element::Timestamp as time_elem_timestamp;
use generated::org::phenopackets::schema::v2::core::time_element::Element::OntologyClass as time_elem_oclass;
use generated::org::phenopackets::schema::v2::core::time_element::Element::Age as time_elem_age;



pub struct TimeElements {

}

impl TimeElements {

    pub fn timestamp(tstamp: Timestamp) -> TimeElement202 {
         TimeElement202 { element: Some(time_elem_timestamp(tstamp))}
    }

    pub fn age<S: Into<String>>(iso_8601_string: S) -> TimeElement202 {
        let age: Age202 = Age202 { iso8601duration: iso_8601_string.into() };
        TimeElement202 { element: Some(time_elem_age(age))}
    }

    pub fn ontology_class<T: Into<String>, U: Into<String>>(
        term_id: T, 
        label: U) -> TimeElement202 {
            let oclass = OntologyClass202{id:term_id.into(), label: label.into()};
            TimeElement202 { element: Some(time_elem_oclass(oclass))}
        }
    
   
}