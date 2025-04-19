use crate::header::header_duplet::{HeaderDuplet, HeaderDupletItem, HeaderDupletItemFactory};
use crate::header::hpo_separator_duplet::HpoSeparatorDuplet;
use crate::header::hpo_term_duplet::HpoTermDuplet;
use crate::header::variant_comment_duplet;
use crate::header::{age_last_encounter::AgeLastEncounterDuplet, age_of_onset_duplet::AgeOfOnsetDuplet, allele_1_duplet::Allele1Duplet, allele_2_duplet::Allele2Duplet, comment_duplet::CommentDuplet, deceased_duplet::DeceasedDuplet, disease_id_duplet::DiseaseIdDuplet, disease_label_duplet::DiseaseLabelDuplet, gene_symbol_duplet::GeneSymbolDuplet, hgnc_duplet::HgncDuplet, individual_id_duplet::IndividualIdDuplet, pmid_duplet::PmidDuplet, sex_duplet::SexDuplet, title_duplet::TitleDuplet, transcript_duplet::TranscriptDuplet, variant_comment_duplet::VariantCommentDuplet};
use crate::error::{self, Error, Result};

use super::header_index::{HeaderIndexer, MendelianHeaderIndexer};


pub trait HeaderDupletRow {
    type Output: HeaderDupletRow;
    type Indexer: HeaderIndexer;
    fn qc_header(&self) -> Result<()>;
    fn extract_from_string_matrix(matrix: &Vec<Vec<String>>) -> Result<MendelianHDRow>;
    fn get_idx(&self, column_name: &str) -> Option<usize>;
}

trait HeaderDupletComponent {
    fn size(&self) -> usize;
    fn from_vector_slice(matrix: & Vec<Vec<String>>, start: usize) -> Result<Self> where Self: Sized;
}

#[derive(Debug)]
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
        4
    }

    fn from_vector_slice(matrix: & Vec<Vec<String>>, start: usize) -> Result<Self> where Self: Sized{
        let mut i = start;
        let pmid_dup = PmidDuplet::from_table(&matrix[0][i], &matrix[1][i])?;
        let title_dup = TitleDuplet::from_table(&matrix[0][i+1], &matrix[1][i+1])?;
        let individual_dup = IndividualIdDuplet::from_table(&matrix[0][i+2], &matrix[1][i+2])?;
        let comment_dup = CommentDuplet::from_table(&matrix[0][i+3], &matrix[1][i+3])?;
        Ok(IndividualDuplets::new(pmid_dup, title_dup, individual_dup, comment_dup))
    }
}


#[derive(Debug)]
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
        8
    }

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

#[derive(Debug)]
pub struct DemographicDuplets {
    age_of_onset: AgeOfOnsetDuplet,
    age_at_last_encounter: AgeLastEncounterDuplet,
    deceased: DeceasedDuplet,
    sex: SexDuplet,
}

impl DemographicDuplets {
    pub fn new(age_of_onset: AgeOfOnsetDuplet,
        age_at_last_encounter: AgeLastEncounterDuplet,
        deceased: DeceasedDuplet,
        sex: SexDuplet,) -> Self {
            Self { age_of_onset, age_at_last_encounter, deceased, sex }
        }
}

impl HeaderDupletComponent for DemographicDuplets {
    fn size(&self) -> usize {
        4
    }
    
    fn from_vector_slice(matrix: & Vec<Vec<String>>, start: usize) -> Result<Self> where Self: Sized {
        let mut i = start;
        let onset_dup = AgeOfOnsetDuplet::from_table(&matrix[0][i], &matrix[1][i])?;
        let encounter_dup = AgeLastEncounterDuplet::from_table(&matrix[0][i+1], &matrix[1][i+1])?;
        let deceased_dup = DeceasedDuplet::from_table(&matrix[0][i+2], &matrix[1][i+2])?;
        let sex_dup = SexDuplet::from_table(&matrix[0][i+3], &matrix[1][i+3])?;
        Ok(DemographicDuplets::new(onset_dup, encounter_dup, deceased_dup, sex_dup))
    }
}


#[derive(Debug)]
pub struct MendelianHDRow {
    individual_duplets: IndividualDuplets,
    disease_gene_duplets: DiseaseGeneDuplets,
    demographic_duplets: DemographicDuplets,
    separator: HpoSeparatorDuplet,
    hpo_duplets: Vec<HpoTermDuplet>,
    indexer: MendelianHeaderIndexer
}

impl MendelianHDRow {
    pub fn new( individual_duplets: IndividualDuplets,
        disease_gene_duplets: DiseaseGeneDuplets,
        demographic_duplets: DemographicDuplets,
        separator: HpoSeparatorDuplet,
        hpo_duplets: Vec<HpoTermDuplet>,) -> Self {
            Self {
                individual_duplets, disease_gene_duplets, demographic_duplets, separator, hpo_duplets,
                indexer: MendelianHeaderIndexer{}
            }
    }
}

impl HeaderDupletRow for MendelianHDRow {
    type Output = Self;
    type Indexer = MendelianHeaderIndexer;
    fn qc_header(&self) -> Result<()> {
        todo!()
    }

    fn get_idx(&self, column_name: &str) -> Option<usize> {
        self.indexer.get_idx(column_name)
    }

    fn extract_from_string_matrix(matrix: &Vec<Vec<String>>) -> Result<MendelianHDRow> {
        if matrix.len() < 2 {
            return Err(Error::TemplateError {
                msg: format!(
                    "Insuffient rows ({}) to construct header duplets",
                    matrix.len()
                ),
            });
        }
        let row_len = matrix[0].len();
        let mut i = 0 as usize;
        let individual_duplets = IndividualDuplets::from_vector_slice(&matrix, i)?;
        let i = i + individual_duplets.size();
        let dg_duplets = DiseaseGeneDuplets::from_vector_slice(&matrix, i)?;
        let i = i + dg_duplets.size();
        let demographic_dup = DemographicDuplets::from_vector_slice(&matrix, i)?;
        let i = i + dg_duplets.size();
        let separator_dup = HpoSeparatorDuplet::from_table(&matrix[0][i], &matrix[1][i])?;
        let i = i + 1;
        let mut hpo_duplets: Vec<HpoTermDuplet> = Vec::new();
        for j in i..row_len {
            let hpo_d = HpoTermDuplet::from_table(&matrix[0][j], &matrix[1][j])?;
            hpo_duplets.push(hpo_d);
        }
        Ok(MendelianHDRow::new(individual_duplets, dg_duplets, demographic_dup, separator_dup, hpo_duplets))
    }
}