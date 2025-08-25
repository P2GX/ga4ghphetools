//! HpoTermAge
//! 
//! Validate and calculate ages of onset based on HPO Terms
//! 
//! 

use std::collections::HashMap;

use once_cell::sync::Lazy;

use crate::dto::hpo_term_dto::HpoTermDuplet;


pub static ONSET_TERM_DICT: Lazy<HashMap<String, HpoTermDuplet>> = Lazy::new(|| {
    let mut onset_d: HashMap<String, HpoTermDuplet> = HashMap::new();
    let onset_terms = 
        vec![("HP:0003584", "Late onset"),
        ("HP:0003596", "Middle age onset"),
        ("HP:0011462", "Young adult onset"),
        ("HP:0025710", "Late young adult onset"),
        ("HP:0025709", "Intermediate young adult onset"),
        ("HP:0025708", "Early young adult onset"),
        ("HP:0003581", "Adult onset"),
        ("HP:0003621", "Juvenile onset"),
        ("HP:0011463", "Childhood onset"),
        ("HP:0003593", "Infantile onset"),
        ("HP:0003623", "Neonatal onset"),
        ("HP:0003577", "Congenital onset"),
        ("HP:0030674", "Antenatal onset"),
        ("HP:0011460", "Embryonal onset"),
        ("HP:0011461", "Fetal onset"),
        ("HP:0034199", "Late first trimester onset"),
        ("HP:0034198", "Second trimester onset"),
        ("HP:0034197", "Third trimester onset")];
    for tuple in onset_terms {
        onset_d.insert(tuple.1.to_string(), HpoTermDuplet::new(tuple.1, tuple.0));
    } 
    onset_d
});



pub struct HpoTermAge{}

impl HpoTermAge {
    pub fn is_valid(cell_contents: &str) -> bool {
        ONSET_TERM_DICT.contains_key(cell_contents)
    }

    pub fn get_duplet(cell_contents: &str) -> Result<HpoTermDuplet, String> {
        match ONSET_TERM_DICT.get(cell_contents) {
            Some(duplet) => Ok(duplet.clone()),
            None => Err(format!("Could not retrieve HPO term for '{cell_contents}'")),
        }
    }
}