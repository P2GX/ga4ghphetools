use clap::builder::Str;
use ontolius::base::TermId;

use crate::transcript;

///! This class bundles and performs QC on the data needed to specify a disease and the
/// corresponding disease gene information for a pyphetools template file.
/// 
/// 


pub struct DiseaseGeneBundle {
    disease_id: TermId,
    disease_name: String,
    hgnc_id: TermId,
    gene_symbol: String,
    transcript: String
}


impl DiseaseGeneBundle {

    pub fn new<T,U,V>(disease_id: &TermId, 
                        disease_name: T,
                        hgnc: &TermId,
                        symbol: U,
                        transcript: V) -> Result<Self, String>
        where   T: Into<String>,
                U: Into<String>,
                V: Into<String> {
        let name = disease_name.into();
        if name.starts_with(|c: char| c.is_whitespace()) {
                return Err(format!("Disease name '{}' starts with whitespace", name));
        }
        if name.ends_with(|c: char| c.is_whitespace()) {
            return Err(format!("Disease name '{}' ends with whitespace", name));
        }
        if name.len() < 5 {
            return Err(format!("Disease name '{}' too short", name));
        }
        let gene_symbol = symbol.into();
        if gene_symbol.starts_with(|c: char| c.is_whitespace()) {
            return Err(format!("Gene symbol '{}' starts with whitespace", gene_symbol));
        }
        if gene_symbol.ends_with(|c: char| c.is_whitespace()) {
            return Err(format!("Gene symbol '{}' ends with whitespace", gene_symbol));
        }
        let tx = transcript.into();
        if tx.starts_with(|c: char| c.is_whitespace()) {
            return Err(format!("Transcript '{}' starts with whitespace", tx));
        }
        if tx.ends_with(|c: char| c.is_whitespace()) {
            return Err(format!("Transcript '{}' ends with whitespace", tx));
        }
        if ! tx.contains(".") {
            return Err(format!("Transcript '{}' does not contain version", tx));
        }

        Ok(Self {
            disease_id: disease_id.clone(),
            disease_name: name,
            hgnc_id: hgnc.clone(),
            gene_symbol: gene_symbol,
            transcript: tx
        })
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
            let result = DiseaseGeneBundle::new(&           disease_id, 
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