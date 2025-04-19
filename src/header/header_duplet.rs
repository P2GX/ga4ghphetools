//! The HeaderDuplet represents the first two rows of the PheTools template.
//!
//! There are two header lines. For the static fields, the information is only needed from the
//! first header. For the HPO columns, the label is shown in the first header and the HPO id is
//! shown in the second field. The purpose of this struct is simply to record the strings in
//! both rows so that we can do some Q/C prior to starting to create the DataFrame object.


use crate::error::{self, Error, Result};
use lazy_static::lazy_static;
use regex::Regex;
use std::{collections::{HashMap, HashSet}, fmt};

use crate::header::pmid_duplet::PmidDuplet;

use super::{age_last_encounter::AgeLastEncounterDuplet, age_of_onset_duplet::AgeOfOnsetDuplet, allele_1_duplet::Allele1Duplet, allele_2_duplet::Allele2Duplet, comment_duplet::CommentDuplet, deceased_duplet::DeceasedDuplet, disease_id_duplet::DiseaseIdDuplet, disease_label_duplet::DiseaseLabelDuplet, gene_symbol_duplet::GeneSymbolDuplet, hgnc_duplet::HgncDuplet, hpo_separator_duplet::HpoSeparatorDuplet, hpo_term_duplet::HpoTermDuplet, individual_id_duplet::IndividualIdDuplet, sex_duplet::SexDuplet, title_duplet::TitleDuplet, transcript_duplet::TranscriptDuplet, variant_comment_duplet::VariantCommentDuplet};

/// Each HeaderDuplet (column) implements these methods.
pub trait HeaderDupletItem {
    /// return the string in the first row of the column, e.g., "PMID"
    fn row1(&self) -> String;
    /// return the string in the second row of the column, e.g., "CURIE"
    fn row2(&self) -> String;
    /// check the validity of a cell
    fn qc_cell(&self, cell_contents: &str) -> Result<()>;
}


pub trait HeaderDupletItemFactory: Sized {
    fn from_table(row1: &str, row2: &str) -> Result<Self>;
    fn into_enum(self) -> HeaderDuplet;
}


#[derive(Clone, Debug, PartialEq)]
pub enum HeaderDuplet {
    PmidDuplet(PmidDuplet),
    TitleDuplet(TitleDuplet),
    IndividualIdDuplet(IndividualIdDuplet),
    CommentDuplet(CommentDuplet),
    DiseaseIdDuplet(DiseaseIdDuplet),
    DiseaseLabelDuplet(DiseaseLabelDuplet),
    HgncDuplet(HgncDuplet),
    GeneSymbolDuplet(GeneSymbolDuplet),
    TranscriptDuplet(TranscriptDuplet),
    Allele1Duplet(Allele1Duplet),
    Allele2Duplet(Allele2Duplet),
    VariantCommentDuplet(VariantCommentDuplet),
    AgeOfOnsetDuplet(AgeOfOnsetDuplet),
    AgeLastEncounterDuplet(AgeLastEncounterDuplet),
    DeceasedDuplet(DeceasedDuplet),
    SexDuplet(SexDuplet),
    HpoSeparatorDuplet(HpoSeparatorDuplet),
    HpoTermDuplet(HpoTermDuplet)
}


lazy_static! {
    pub static ref TITLE_MAP: HashMap<String, HeaderDuplet> = {
        let mut tmap = HashMap::new();
        let pmid_instance = HeaderDuplet::PmidDuplet(PmidDuplet::new());
        tmap.insert(pmid_instance.row1(), pmid_instance);
        let title_instance = HeaderDuplet::TitleDuplet(TitleDuplet::new());
        tmap.insert(title_instance.row1(), title_instance);
        let individual_instance = HeaderDuplet::IndividualIdDuplet(IndividualIdDuplet::new());
        tmap.insert(individual_instance.row1(), individual_instance);
        let comment_instance = HeaderDuplet::CommentDuplet(CommentDuplet::new());
        tmap.insert(comment_instance.row1(), comment_instance);
        let dis_id_instance = HeaderDuplet::DiseaseIdDuplet(DiseaseIdDuplet::new());
        tmap.insert(dis_id_instance.row1(), dis_id_instance);
        let dis_label_instance = HeaderDuplet::DiseaseLabelDuplet(DiseaseLabelDuplet::new());
        tmap.insert(dis_label_instance.row1(), dis_label_instance);
        let hgnc_instance = HeaderDuplet::HgncDuplet(HgncDuplet::new());
        tmap.insert(hgnc_instance.row1(), hgnc_instance);
        let gene_instance = HeaderDuplet::GeneSymbolDuplet(GeneSymbolDuplet::new());
        tmap.insert(gene_instance.row1(), gene_instance);
        let transcript_instance = HeaderDuplet::TranscriptDuplet(TranscriptDuplet::new());
        tmap.insert(transcript_instance.row1(), transcript_instance);
        let a1_instance = HeaderDuplet::Allele1Duplet(Allele1Duplet::new());
        tmap.insert(a1_instance.row1(), a1_instance);
        let a2_instance = HeaderDuplet::Allele2Duplet(Allele2Duplet::new());
        tmap.insert(a2_instance.row1(), a2_instance);
        let var_comment_instance = HeaderDuplet::VariantCommentDuplet(VariantCommentDuplet::new());
        tmap.insert(var_comment_instance.row1(), var_comment_instance);
        let age_onset_instance = HeaderDuplet::AgeOfOnsetDuplet(AgeOfOnsetDuplet::new());
        tmap.insert(age_onset_instance.row1(), age_onset_instance);
        let age_encounter_instance = HeaderDuplet::AgeLastEncounterDuplet(AgeLastEncounterDuplet::new());
        tmap.insert(age_encounter_instance.row1(), age_encounter_instance);
        let deceased_instance = HeaderDuplet::DeceasedDuplet(DeceasedDuplet::new());
        tmap.insert(deceased_instance.row1(), deceased_instance);
        let sex_instance = HeaderDuplet::SexDuplet(SexDuplet::new());
        tmap.insert(sex_instance.row1(), sex_instance);
        let hpo_sep_instance = HeaderDuplet::HpoSeparatorDuplet(HpoSeparatorDuplet::new());
        tmap.insert(hpo_sep_instance.row1(), hpo_sep_instance);
        tmap
    };
}



impl HeaderDuplet {
    pub fn as_trait(&self) -> &dyn HeaderDupletItem {
        match self {
            HeaderDuplet::PmidDuplet(inner) => inner,
            HeaderDuplet::TitleDuplet(inner) => inner,
            HeaderDuplet::IndividualIdDuplet(inner) => inner,
            HeaderDuplet::CommentDuplet(inner) => inner,
            HeaderDuplet::DiseaseIdDuplet(inner) => inner,
            HeaderDuplet::DiseaseLabelDuplet(inner) => inner,
            HeaderDuplet::HgncDuplet(inner) => inner,
            HeaderDuplet::GeneSymbolDuplet(inner) => inner,
            HeaderDuplet::TranscriptDuplet(inner) => inner,
            HeaderDuplet::Allele1Duplet(inner) => inner,
            HeaderDuplet::Allele2Duplet(inner) => inner,
            HeaderDuplet::VariantCommentDuplet(inner ) => inner,
            HeaderDuplet::AgeOfOnsetDuplet(inner) => inner,
            HeaderDuplet::AgeLastEncounterDuplet(inner) => inner,
            HeaderDuplet::DeceasedDuplet(inner) => inner,
            HeaderDuplet::SexDuplet(inner) => inner,
            HeaderDuplet::HpoSeparatorDuplet(inner) => inner,
            HeaderDuplet::HpoTermDuplet(inner) => inner,
        }
    }


    pub fn extract_from_string_matrix(matrix: &Vec<Vec<String>>) -> Result<Vec<HeaderDuplet>> {
        if matrix.len() < 2 {
            return Err(Error::TemplateError {
                msg: format!(
                    "Insuffient rows ({}) to construct header duplets",
                    matrix.len()
                ),
            });
        }
        let row_len = matrix[0].len();
        let mut header_duplet_list: Vec<HeaderDuplet> = Vec::new();
        for i in 0..row_len {
            let title = &matrix[0][i];
            match TITLE_MAP.get(title) {
                Some(hdup) => header_duplet_list.push(hdup.clone()),
                None => {
                    // these are either HpoTerm columns or errors that will be caught downstream
                    let hdup = HpoTermDuplet::new(&matrix[0][i], &matrix[1][i]);
                    header_duplet_list.push(hdup.into_enum());
                }
            }
        }
        Ok(header_duplet_list)
    }

    pub fn get_duplet(title: &str) -> Option<Self> {
        TITLE_MAP.get(title).cloned()
    }
}


impl HeaderDupletItem for HeaderDuplet {
    fn row1(&self) -> String {
        let inner = self.as_trait();
        inner.row1()
    }
    
    fn row2(&self) -> String {
        let inner = self.as_trait();
        inner.row2()
    }
    
    fn qc_cell(&self, cell_contents: &str) -> Result<()> {
        let inner = self.as_trait();
        inner.qc_cell(cell_contents)
    }
}

impl fmt::Display for HeaderDuplet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let inner = self.as_trait();
        write!(f, "HeaderDuplet(h1: {}, h2: {})", inner.row1(), inner.row2())
    }
}


/// A valid label does not begin with or end with a white space
/// Valid labels also may not contain /,\, (,  ), or perdiod (".").
pub fn check_white_space(value: &str) -> Result<()> {
    if value.chars().last().map_or(false, |c| c.is_whitespace()) {
        return Err(Error::trailing_ws(value));
    } else if value.chars().next().map_or(false, |c| c.is_whitespace()) {
        return Err(Error::leading_ws(value));
    } else if value.contains("  ") {
        return Err(Error::consecutive_ws(value));
    } else {
        Ok(())
    }
}

/// Some ColumnTypes do not allow empty cells.
pub fn check_empty(value: &str) -> Result<()> {
    if value.is_empty() {
        Err(Error::HeaderError{msg: format!("Value must not be empty")})
    } else {
        Ok(())
    }
}
    
/// Cell entries are not allow to contain tab characters 
pub fn check_tab(value: &str) -> Result<()> {
    if value.contains('\t') {
        Err(Error::HeaderError{msg: format!("Value must not contain a tab character")})
    } else {
        Ok(())
    }
}


/// A valid curie must have a non-empty prefix and a non-empty numeric suffix. White-space is not allowed.
pub fn check_valid_curie(s: &str) -> Result<bool> {
    if s.is_empty() {
        return Err(Error::CurieError {
            msg: "Empty CURIE".to_string(),
        });
    } else if let Some(pos) = s.find(':') {
        if s.chars().any(|c| c.is_whitespace()) {
            return Err(Error::CurieError {
                msg: format!("Contains stray whitespace: '{}'", s),
            });
        } else if s.matches(':').count() != 1 {
            return Err(Error::CurieError {
                msg: format!("Invalid CURIE with more than one colon: '{}", s),
            });
        } else if pos == 0 {
            return Err(Error::CurieError {
                msg: format!("Invalid CURIE with no prefix: '{}'", s),
            });
        } else if pos == s.len() - 1 {
            return Err(Error::CurieError {
                msg: format!("Invalid CURIE with no suffix: '{}'", s),
            });
        } else if let Some((_prefix, suffix)) = s.split_once(':') {
            if !suffix.chars().all(char::is_numeric) {
                return Err(Error::CurieError {
                    msg: format!("Invalid CURIE with non-digit characters in suffix: '{}'", s),
                });
            }
        }
    } else {
        return Err(Error::CurieError {
            msg: format!("Invalid CURIE with no colon: '{}'", s),
        });
    }
    Ok(true)
}




/// perform quality control of the two header rows of a pyphetools template file
pub fn qc_list_of_header_items(
    header_duplets: &Vec<HeaderDuplet>,
) -> core::result::Result<(), Vec<String>> {
    // check each of the items in turn
    let mut errors: Vec<String> = vec![];
    if errors.len() > 0 {
        return Err(errors);
    }
    Ok(())
}

// region:    --- Tests

#[cfg(test)]
mod tests {
    type Error = Box<dyn std::error::Error>;
    type Result<T> = core::result::Result<T, Error>; // For tests.

    use super::*;

    #[test]
    fn test_ctor() -> Result<()> {
        let hdup_a = PmidDuplet::new().into_enum();
        let hdup_b = PmidDuplet::new().into_enum();
        let hdup_c = HgncDuplet::new().into_enum();
        assert_eq!(hdup_a, hdup_b);
        assert_ne!(hdup_a, hdup_c);

        Ok(())
    }
}

// endregion: --- Tests
