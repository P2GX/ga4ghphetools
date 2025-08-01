use std::collections::HashMap;


#[derive(Debug, Clone, Copy)]
pub struct Freq {
    pub numerator: usize,
    pub denominator: usize,
}


pub struct PmidCounter {
    pmid: String,
    term_map: HashMap<String, Freq>
}


impl PmidCounter {
    pub fn new(pmid: &str) -> Self {
        Self { 
            pmid: pmid.to_string(),
            term_map: HashMap::new() 
        }
    }

    /// Add the term as observed or increment if term already present
    pub fn observed(&mut self, tid: &str) {
        self.term_map.entry(tid.to_string())
            .and_modify(|freq| { freq.numerator += 1; freq.denominator += 1; } )
            .or_insert(Freq {
                numerator: 1,
                denominator: 1,
            });
    }

    /// Add the term as excluded or increment if term already present
    pub fn excluded(&mut self, tid: &str) {
        self.term_map.entry(tid.to_string())
            .and_modify(|freq| { freq.denominator += 1; } )
            .or_insert(Freq {
                numerator: 1,
                denominator: 1,
            });
    }

    pub fn contains(&self, term_id: &str) -> bool {
        self.term_map.contains_key(term_id)
    }

    pub fn get_freq(&self, tid: &str) -> Result<String,String> {
        match self.term_map.get(tid) {
            Some(freq) => Ok(format!("{}/{}", freq.numerator, freq.denominator)),
            None => Err(format!("Could not retrieve frequency for {:?}", tid)),
        }
    }
}