use crate::header::header_duplet::{HeaderDuplet, HeaderDupletItem, HeaderDupletItemFactory};
use crate::header::hpo_separator_duplet::HpoSeparatorDuplet;
use crate::header::hpo_term_duplet::HpoTermDuplet;
use crate::header::{individual_id_duplet, variant_comment_duplet};
use crate::header::{age_last_encounter::AgeLastEncounterDuplet, age_of_onset_duplet::AgeOfOnsetDuplet, allele_1_duplet::Allele1Duplet, allele_2_duplet::Allele2Duplet, comment_duplet::CommentDuplet, deceased_duplet::DeceasedDuplet, disease_id_duplet::DiseaseIdDuplet, disease_label_duplet::DiseaseLabelDuplet, gene_symbol_duplet::GeneSymbolDuplet, hgnc_duplet::HgncDuplet, individual_id_duplet::IndividualIdDuplet, pmid_duplet::PmidDuplet, sex_duplet::SexDuplet, title_duplet::TitleDuplet, transcript_duplet::TranscriptDuplet, variant_comment_duplet::VariantCommentDuplet};
use crate::error::{self, Error, Result};

use super::header_index::HeaderIndexer;

macro_rules! impl_duplet_getters {
    ($( $variant:ident => $method:ident : $ty:ty ),*) => {
        impl HeaderDuplet {
            $(
            pub fn $method(&self) -> std::result::Result<&$ty, crate::error::Error> {
                match self {
                    HeaderDuplet::$variant(inner) => Ok(inner),
                    _ => Err(crate::error::Error::TemplateError {
                        msg: format!("Expected {}", stringify!($variant)),
                    }),
                }
            }
            )*
        }
    };
}

/// Number of columns in the Individual section
const NUMBER_OF_INDIVIDUAL_FIELDS: usize = 4;
/// Number of columns in the Disease/Gene/Variant bundle section
const NUMBER_OF_DISEASE_GENE_BUNDLE_FIELDS: usize = 8;
/// Number of columns in the Demographic section
const NUMBER_OF_DEMOGRAPHIC_FIELDS: usize = 4;
/// Separator field (HPO/na)
const NUMBER_OF_SEPARATOR_FIELDS: usize = 1;

/// Total number of constant fields (columns) in the Mendelian template
const NUMBER_OF_CONSTANT_HEADER_FIELDS_MENDELIAN: usize = 
    NUMBER_OF_INDIVIDUAL_FIELDS + NUMBER_OF_DISEASE_GENE_BUNDLE_FIELDS + NUMBER_OF_DEMOGRAPHIC_FIELDS + NUMBER_OF_SEPARATOR_FIELDS;

impl_duplet_getters!(
    PmidDuplet => as_pmid_duplet: PmidDuplet,
    TitleDuplet => as_title_duplet: TitleDuplet,
    IndividualIdDuplet => as_individual_id_duplet: IndividualIdDuplet,
    CommentDuplet => as_comment_duplet: CommentDuplet,
    DiseaseIdDuplet => as_disease_id: DiseaseIdDuplet,
    DiseaseLabelDuplet => as_disease_label: DiseaseLabelDuplet,
    HgncDuplet => as_hgnc_duplet: HgncDuplet,
    GeneSymbolDuplet => as_gene_symbol_duplet: GeneSymbolDuplet,
    TranscriptDuplet => as_transcript_duplet: TranscriptDuplet,
    Allele1Duplet => as_allele_1_duplet: Allele1Duplet,
    Allele2Duplet => as_allele_2_duplet: Allele2Duplet,
    VariantCommentDuplet => as_variant_comment_duplet: VariantCommentDuplet,
    AgeOfOnsetDuplet => as_age_of_onset_duplet: AgeOfOnsetDuplet,
    AgeLastEncounterDuplet => as_age_last_encounter_duplet: AgeLastEncounterDuplet,
    DeceasedDuplet => as_deceased_duplet: DeceasedDuplet,
    SexDuplet => as_sex_duplet: SexDuplet,
    HpoSeparatorDuplet => as_separator_duplet: HpoSeparatorDuplet,
    HpoTermDuplet => as_hpo_term_duplet: HpoTermDuplet
);

impl Error {
    fn could_not_extract_duplet(item: &str, i: usize) -> Self {
        Error::TemplateError { msg: format!("Could not extract {item} at index {i}") }
    }

    fn could_not_extract_hpo_duplet(i: usize) -> Self {
        Error::TemplateError { msg: format!("Could not extract HPO Term Column at index {i}") }
    }

    fn template_index_error(actual: usize, maxi: usize, template_name: &str) -> Self {
        Error::TemplateError { msg: format!("Attempt to access item at index {actual} but {template_name} has {maxi} items.") }
    }
    
}

trait HeaderDupletComponent {
    fn size(&self) -> usize;
    fn qc_check(&self, i: usize, cell_contents: &str) -> Result<()>;
}

trait HeaderDupletComponentFactory {
    fn from_vector_slice(matrix: & Vec<Vec<String>>, start: usize) -> Result<Self> where Self: Sized;
}

#[derive(Clone, Debug)]
pub struct IndividualDuplets {
    pmid: PmidDuplet,
    title: TitleDuplet,
    individual_id: IndividualIdDuplet,
    comment: CommentDuplet
}

impl IndividualDuplets {
    pub fn new( pmid: PmidDuplet,
        title: TitleDuplet,
        individual_id: IndividualIdDuplet,
        comment: CommentDuplet) -> Self {
            Self { pmid, title, individual_id, comment }
        }
}

impl HeaderDupletComponent for IndividualDuplets {
    fn size(&self) -> usize {
        NUMBER_OF_INDIVIDUAL_FIELDS
    }
    
    fn qc_check(&self, i: usize, cell_contents: &str) -> Result<()> {
        match i {
            0 => self.pmid.qc_cell(cell_contents),
            1 => self.title.qc_cell(cell_contents),
            2 => self.individual_id.qc_cell(cell_contents),
            3 => self.comment.qc_cell(cell_contents),
            _ => Err(Error::template_index_error(i, self.size(), "IndividualDuplets"))
        }
    }
}

impl HeaderDupletComponentFactory for IndividualDuplets {
    fn from_vector_slice(matrix: & Vec<Vec<String>>, start: usize) -> Result<Self> where Self: Sized{
        let mut i = start;
        let pmid_dup = PmidDuplet::from_table(&matrix[0][i], &matrix[1][i])?;
        let title_dup = TitleDuplet::from_table(&matrix[0][i+1], &matrix[1][i+1])?;
        let individual_dup = IndividualIdDuplet::from_table(&matrix[0][i+2], &matrix[1][i+2])?;
        let comment_dup = CommentDuplet::from_table(&matrix[0][i+3], &matrix[1][i+3])?;
        Ok(IndividualDuplets::new(pmid_dup, title_dup, individual_dup, comment_dup))
    }
}


#[derive(Clone, Debug)]
pub struct DiseaseGeneDuplets {
    disease_id: DiseaseIdDuplet,
    disease_label: DiseaseLabelDuplet,
    hgnc_id: HgncDuplet,
    gene_symbol: GeneSymbolDuplet,
    transcript: TranscriptDuplet,
    allele_1: Allele1Duplet,
    allele_2: Allele2Duplet,
    variant_comment: VariantCommentDuplet
}

impl DiseaseGeneDuplets {
    pub fn new(
        disease_id: DiseaseIdDuplet,
        disease_label: DiseaseLabelDuplet,
        hgnc_id: HgncDuplet,
        gene_symbol: GeneSymbolDuplet,
        transcript: TranscriptDuplet,
        allele_1: Allele1Duplet,
        allele_2: Allele2Duplet,
        variant_comment: VariantCommentDuplet
    ) -> Self {
        Self {
            disease_id,
            disease_label,
            hgnc_id,
            gene_symbol,
            transcript,
            allele_1,
            allele_2,
            variant_comment
        }
    }
}

impl HeaderDupletComponent for DiseaseGeneDuplets {
    fn size(&self) -> usize {
        NUMBER_OF_DISEASE_GENE_BUNDLE_FIELDS
    }

    fn qc_check(&self, i: usize, cell_contents: &str) -> Result<()> {
        match i {
            0 => self.disease_id.qc_cell(cell_contents),
            1 => self.disease_label.qc_cell(cell_contents),
            2 => self.hgnc_id.qc_cell(cell_contents),
            3 => self.gene_symbol.qc_cell(cell_contents),
            4 => self.transcript.qc_cell(cell_contents),
            5 => self.allele_1.qc_cell(cell_contents),
            6 => self.allele_2.qc_cell(cell_contents),
            7 => self.variant_comment.qc_cell(cell_contents),
            _ => Err(Error::template_index_error(i, self.size(), "DiseaseGeneDuplets"))
        }
    }
}

impl HeaderDupletComponentFactory for DiseaseGeneDuplets {
    fn from_vector_slice(matrix: & Vec<Vec<String>>, start: usize) -> Result<Self> where Self: Sized{
        let mut i = start;
        let disease_id_dup = DiseaseIdDuplet::from_table(&matrix[0][i], &matrix[1][i])?;
        let disease_label_dup = DiseaseLabelDuplet::from_table(&matrix[0][i+1], &matrix[1][i+1])?;
        let hgnc_dup = HgncDuplet::from_table(&matrix[0][i+2], &matrix[1][i+2])?;
        let gene_dup = GeneSymbolDuplet::from_table(&matrix[0][i+3], &matrix[1][i+3])?;
        let transcript_dup = TranscriptDuplet::from_table(&matrix[0][i+4], &matrix[1][i+4])?;
        let allele_1_dup = Allele1Duplet::from_table(&matrix[0][i+5], &matrix[1][i+5])?;
        let allele_2_dup = Allele2Duplet::from_table(&matrix[0][i+6], &matrix[1][i+6])?;
        let variant_c_dup = VariantCommentDuplet::from_table(&matrix[0][i+7], &matrix[1][i+7])?;
        Ok(Self::new(disease_id_dup, disease_label_dup, hgnc_dup, gene_dup, transcript_dup, allele_1_dup, allele_2_dup, variant_c_dup))
    }
}

#[derive(Clone, Debug)]
pub struct DemographicDuplets {
    age_of_onset: AgeOfOnsetDuplet,
    age_at_last_encounter: AgeLastEncounterDuplet,
    deceased: DeceasedDuplet,
    sex: SexDuplet,
}

impl DemographicDuplets {
    pub fn new(
        age_of_onset: AgeOfOnsetDuplet,
        age_at_last_encounter: AgeLastEncounterDuplet,
        deceased: DeceasedDuplet,
        sex: SexDuplet,) -> Self {
            Self { age_of_onset, age_at_last_encounter, deceased, sex }
        }
}

impl HeaderDupletComponent for DemographicDuplets {
    fn size(&self) -> usize {
        NUMBER_OF_DEMOGRAPHIC_FIELDS
    }
    
    fn qc_check(&self, i: usize, cell_contents: &str) -> Result<()> {
        match i {
            0 => self.age_of_onset.qc_cell(cell_contents),
            1 => self.age_at_last_encounter.qc_cell(cell_contents),
            2 => self.deceased.qc_cell(cell_contents),
            3 => self.sex.qc_cell(cell_contents),
            _ => Err(Error::template_index_error(i, self.size(), "DemographicDuplets"))
        }
    }

}

impl HeaderDupletComponentFactory for DemographicDuplets {
    fn from_vector_slice(matrix: & Vec<Vec<String>>, start: usize) -> Result<Self> where Self: Sized {
        let mut i = start;
        let onset_dup = AgeOfOnsetDuplet::from_table(&matrix[0][i], &matrix[1][i])?;
        let encounter_dup = AgeLastEncounterDuplet::from_table(&matrix[0][i+1], &matrix[1][i+1])?;
        let deceased_dup = DeceasedDuplet::from_table(&matrix[0][i+2], &matrix[1][i+2])?;
        let sex_dup = SexDuplet::from_table(&matrix[0][i+3], &matrix[1][i+3])?;
        Ok(DemographicDuplets::new(onset_dup, encounter_dup, deceased_dup, sex_dup))
    }
}


#[derive(Clone, Debug)]
pub struct HeaderDupletRow {
    /// Four columns that specify the PMID, title, and individual_id with optional comment
    individual_duplets: IndividualDuplets,
    /// Columns that specific the disease, gene, and variants
    disease_gene_duplets: DiseaseGeneDuplets,
    /// Columns to specify age, sex, vital status
    demographic_duplets: DemographicDuplets,
    /// A Column to specify the constant data columns from the variable HPO Term columns
    separator: HpoSeparatorDuplet,
    /// Variable number of columns with the HPO annotations.
    hpo_duplets: Vec<HpoTermDuplet>,
    indexer: HeaderIndexer,
}

impl HeaderDupletRow {
    pub fn mendelian(
        individual_duplets: IndividualDuplets,
        disease_gene_duplets: DiseaseGeneDuplets,
        demographic_duplets: DemographicDuplets,
        separator: HpoSeparatorDuplet,
        hpo_duplets: Vec<HpoTermDuplet>
    ) -> Self {
            Self {
                individual_duplets, 
                disease_gene_duplets, 
                demographic_duplets, 
                separator, 
                hpo_duplets,
                indexer: HeaderIndexer::mendelian(),
            }
    }
}

impl HeaderDupletRow  {
    fn qc_header(&self) -> Result<()> {
        todo!()
    }

    pub fn get_idx(&self, column_name: &str) -> Option<usize> {
        self.indexer.get_idx(column_name)
    }


    fn check_individual_duplets(header_duplets: &Vec<HeaderDuplet>, i: usize) -> Result<IndividualDuplets> {
        let pmid_dup = header_duplets
            .get(i)
            .ok_or_else(|| Error::could_not_extract_duplet("PMID", i))?
            .as_pmid_duplet()?;
        let title_dup = header_duplets
            .get(i+1)
            .ok_or_else(|| Error::could_not_extract_duplet("title", i+1))?
            .as_title_duplet()?;
        let individual_id_dupl = header_duplets
            .get(i+2)
            .ok_or_else(|| Error::could_not_extract_duplet("individual_id", i+2))?
            .as_individual_id_duplet()?;
        let comment_dup =  header_duplets
            .get(i+3)
            .ok_or_else(|| Error::could_not_extract_duplet("comment", i+3))?
            .as_comment_duplet()?;
        Ok(IndividualDuplets::new(pmid_dup.clone(), title_dup.clone(), individual_id_dupl.clone(), comment_dup.clone()))
    }

    pub  fn check_disease_gene_duplets(header_duplets: &Vec<HeaderDuplet>, i: usize) -> Result<DiseaseGeneDuplets> {
        let disease_id = header_duplets
            .get(i)
            .ok_or_else(|| Error::could_not_extract_duplet("disease_id", i))?
            .as_disease_id()?;
        let disease_label = header_duplets
            .get(i+1)
            .ok_or_else(|| Error::could_not_extract_duplet("disease_label", i+1))?
            .as_disease_label()?;
        let hgnc_id =  header_duplets
            .get(i+2)
            .ok_or_else(|| Error::could_not_extract_duplet("hgnc_id", i+2))?
            .as_hgnc_duplet()?;
        let gene_symbol = header_duplets
            .get(i+3)
            .ok_or_else(|| Error::could_not_extract_duplet("gene_symbol", i+3))?
            .as_gene_symbol_duplet()?;
        let transcript= header_duplets
            .get(i+4)
            .ok_or_else(|| Error::could_not_extract_duplet("transcript", i+4))?
            .as_transcript_duplet()?;
        let allele_1 = header_duplets
            .get(i+5)
            .ok_or_else(|| Error::could_not_extract_duplet("allele_1", i+5))?
            .as_allele_1_duplet()?;
        let allele_2 = header_duplets
            .get(i+6)
            .ok_or_else(|| Error::could_not_extract_duplet("allele_2", i+6))?
            .as_allele_2_duplet()?;
        let variant_comment =  header_duplets
            .get(i+7)
            .ok_or_else(|| Error::could_not_extract_duplet("variant.comment", i+7))?
            .as_variant_comment_duplet()?;
        Ok(DiseaseGeneDuplets::new(disease_id.clone(), disease_label.clone(), hgnc_id.clone(), gene_symbol.clone(), transcript.clone(), allele_1.clone(), allele_2.clone(), variant_comment.clone()))
    }

    fn check_demographic_dups(header_duplets: &Vec<HeaderDuplet>, i: usize) -> Result<DemographicDuplets> {
        let age_of_onset = header_duplets
            .get(i)
            .ok_or_else(|| Error::could_not_extract_duplet("age_of_onset", i))?
            .as_age_of_onset_duplet()?;
        let age_at_last_encounter = header_duplets
            .get(i+1)
            .ok_or_else(|| Error::could_not_extract_duplet("age_at_last_encounter", i+1))?
            .as_age_last_encounter_duplet()?;
        let deceased =  header_duplets
            .get(i+2)
            .ok_or_else(|| Error::could_not_extract_duplet("deceased", i+2))?
            .as_deceased_duplet()?;
        let sex =  header_duplets
            .get(i+3)
            .ok_or_else(|| Error::could_not_extract_duplet("sex", i+3))?
            .as_sex_duplet()?;
        Ok(DemographicDuplets::new(age_of_onset.clone(), age_at_last_encounter.clone(), deceased.clone(), sex.clone()))
    }

    fn check_hpo_separator(header_duplets: &Vec<HeaderDuplet>, i: usize) -> Result<HpoSeparatorDuplet> {
        let separator = header_duplets
            .get(i)
            .ok_or_else(|| Error::could_not_extract_duplet("HPO", i))?
            .as_separator_duplet()?;
        Ok(separator.clone())
    }

    fn check_hpo_term(header_duplets: &Vec<HeaderDuplet>, i: usize) -> Result<HpoTermDuplet> {
        let hpo_term_dup = header_duplets
            .get(i)
            .ok_or_else(|| Error::could_not_extract_hpo_duplet(i))?
            .as_hpo_term_duplet()?;
        Ok(hpo_term_dup.clone())
    }

    pub fn n_mendelian_contant_fields() -> usize {
        NUMBER_OF_CONSTANT_HEADER_FIELDS_MENDELIAN
    }

    /// TODO. Currently, this function just does Mendelian, we need to generalize to blended
    pub fn from_duplets(header_duplets: &Vec<HeaderDuplet>) -> Result<Self> {
        let mut i: usize = 0;
        let individual_dups = Self::check_individual_duplets(header_duplets, i)?;
        i = i + individual_dups.size();
        let dgb_dups = Self::check_disease_gene_duplets(header_duplets, i)?;
        i = i + dgb_dups.size();
        let demographic = Self::check_demographic_dups(header_duplets, i)?;
        i = i + demographic.size();
        let separator = Self::check_hpo_separator(header_duplets, i)?;
        i = i+1;
        /// The rest of the duplets are HpoTermDuplets
        let mut hpo_duplets: Vec<HpoTermDuplet> = Vec::new();
        for j in i..header_duplets.len() {
            match Self::check_hpo_term(header_duplets, j) {
                Ok(duplet) => hpo_duplets.push(duplet),
                Err(e) => { return Err(e); }
            }
        }
        Ok(HeaderDupletRow::mendelian(individual_dups, dgb_dups, demographic, separator, hpo_duplets))
    }

    pub fn qc_check(&self, i: usize, cell_contents: &str) -> Result<()> {
        let mut j = i;
        if j < self.individual_duplets.size() {
            self.individual_duplets.qc_check(j, cell_contents)?;
            return Ok(());
        }
        j = j - self.individual_duplets.size();
        if j < self.disease_gene_duplets.size() {
            self.disease_gene_duplets.qc_check(j, cell_contents)?;
            return Ok(());
        }
        j = j - self.disease_gene_duplets.size();
        if j < self.demographic_duplets.size() {
            self.demographic_duplets.qc_check(j, cell_contents)?;
            return Ok(());
        }
        j = j - self.demographic_duplets.size();
        if j == 0 {
            self.separator.qc_cell(cell_contents)?;
            return Ok(());
        }
        j = j+1;
        match self.hpo_duplets.get(j) {
            Some(hpo_dup) => {
                return hpo_dup.qc_cell(cell_contents);
            },
            None => return  {
                Err(Error::TemplateError { msg:format!("Unable to retrieve HPO duplet at i={}, j={}", i, j) })
            }
        }
    }

    /*
      /// Four columns that specify the PMID, title, and individual_id with optional comment
    individual_duplets: IndividualDuplets,
    /// Columns that specific the disease, gene, and variants
    disease_gene_duplets: DiseaseGeneDuplets,
    /// Columns to specify age, sex, vital status
    demographic_duplets: DemographicDuplets,
    /// A Column to specify the constant data columns from the variable HPO Term columns
    separator: HpoSeparatorDuplet,
    /// Variable number of columns with the HPO annotations.
    hpo_duplets: Vec<HpoTermDuplet>,
    indexer: HeaderIndexer,
     */

}


#[cfg(test)]
mod test {
    use super::*;
    use crate::{error::Error, header::{header_duplet::HeaderDupletItem, hpo_term_duplet::HpoTermDuplet}};
    use ontolius::{io::OntologyLoaderBuilder, ontology::csr::MinimalCsrOntology};
    use rstest::{fixture, rstest};

    #[rstest]
    fn test_n_fields() {
        /// We expect a total of 17 fields before the HPO Term fields start
        assert_eq!(17, HeaderDupletRow::n_mendelian_contant_fields())
    }


}