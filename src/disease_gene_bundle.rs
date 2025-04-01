//! DiseaseGeneBundle contains the data needed to specify a disease and the associated gene.
//! 
//! 


use std::str::FromStr;

use ontolius::TermId;
use crate::error::{self, Error, Result};

pub struct DiseaseGeneBundle {
    /// A CURIE representing a disease identifier (e.g., OMIM:256550)
    disease_id: TermId,
    /// The corresponding label of the disease (e.g., Sialidosis, type I)
    disease_name: String,
    /// The HHUGO Gene Nomenclature Committe identifier, e.g., HGNC:7758
    hgnc_id: TermId,
    /// The corresponding gene symbol, e.g., NEU1
    gene_symbol: String,
    /// The transcript used for annotation, usually MANE Select, e.g., NM_000434.4
    transcript: String
}




impl DiseaseGeneBundle {

    pub fn new<T,U,V>(disease_id: &TermId, 
                        disease_name: T,
                        hgnc: &TermId,
                        symbol: U,
                        transcript: V) -> Result<Self>
        where   T: Into<String>,
                U: Into<String>,
                V: Into<String> {
        let name = disease_name.into();
        if name.starts_with(|c: char| c.is_whitespace()) {
            return Err(Error::leading_ws( name));
        }
        if name.ends_with(|c: char| c.is_whitespace()) {
            return Err(Error::trailing_ws( name));
        }
        if name.len() < 5 {
            return Err(Error::short_label(&name, name.len(), 5));
        }
        let gene_symbol = symbol.into();
        if gene_symbol.starts_with(|c: char| c.is_whitespace()) {
            return Err(Error::leading_ws( name));
        }
        if gene_symbol.ends_with(|c: char| c.is_whitespace()) {
            return Err(Error::trailing_ws( name));
        }
        let tx = transcript.into();
        if tx.starts_with(|c: char| c.is_whitespace()) {
            return Err(Error::leading_ws( name));
        }
        if tx.ends_with(|c: char| c.is_whitespace()) {
            return Err(Error::trailing_ws( name));
        }
        if ! tx.contains(".") {
            return Err(Error::lacks_transcript_version(tx));
        }

        Ok(Self {
            disease_id: disease_id.clone(),
            disease_name: name,
            hgnc_id: hgnc.clone(),
            gene_symbol: gene_symbol,
            transcript: tx
        })
    }

    pub fn new_from_str(disease_id: &str, 
        disease_name: &str,
        hgnc: &str,
        symbol: &str,
        transcript: &str) -> Result<Self> {
            let disease_tid = TermId::from_str(disease_id)
                .map_err(|e| Error::termid_parse_error(disease_id))?;
            let hgnc_tid = TermId::from_str(hgnc)
                .map_err(|e| Error::termid_parse_error(hgnc))?;
            return Self::new(&disease_tid, disease_name, &hgnc_tid, symbol, transcript);
    }

    pub fn disease_id_as_string(&self) -> String {
        self.disease_id.to_string()
    } 

    pub fn disease_name(&self) -> String {
        self.disease_name.clone()
    } 

    pub fn hgnc_id_as_string(&self) -> String {
        self.hgnc_id.to_string()
    } 

    pub fn gene_symbol(&self) -> String {
        self.gene_symbol.clone()
    } 

    pub fn transcript(&self) -> String {
        self.transcript.clone()
    } 


}












#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    

    #[test]
    fn test_disease_name() {
        // disease names less than 5 characters or with leading/trailing whitespace 
        // are not allowed
        let tests = vec![("Marfan syndrome", true),
        (" Marfan syndrome", false),
        ("Marfan syndrome ", false),
        ("xyz", false)
        ];
        for test in tests {
            let disease_id = TermId::from_str("OMIM:154700").unwrap();
            let hgnc_id = TermId::from_str("HGNC:1234").unwrap();
            let result = DiseaseGeneBundle::new(&disease_id, 
                        test.0, 
                        &hgnc_id, 
                        "FBN1",
                         "NM_0123.1");
            if test.1 {
                assert!(result.is_ok());
            } else {
                assert!(result.is_err());
            }
        }
    }


}