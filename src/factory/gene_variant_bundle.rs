use std::sync::Arc;
use once_cell::sync::Lazy;

use crate::{dto::cohort_dto::GeneVariantData, header::gene_variant_header::GeneVariantHeader};


static SHARED_HEADER: Lazy<Arc<GeneVariantHeader>> = Lazy::new(|| {
    Arc::new(GeneVariantHeader::new())
});


/// This struct represents six variant-related columns in the legacy Excel pyphetools format.
/// TODO it can be removed once we have transformed all of the legacy Excel files and replaced them with the corresponding JSON files.
#[derive(Clone, Debug)]
pub struct GeneVariantBundle {
    header: Arc<GeneVariantHeader>,
    pub(crate) hgnc_id: String,
    pub(crate) gene_symbol: String,
    pub(crate) transcript: String,
    pub(crate) allele1: String,
    pub(crate) allele2: String,
    pub(crate) variant_comment: String,
}


impl GeneVariantBundle {
    pub fn new(
        hgnc_id: &str,
        gene_symbol: &str,
        transcript: &str,
        allele1: &str,
        allele2: &str,
        variant_comment: &str) 
    -> Self {
        Self { 
            header: SHARED_HEADER.clone(), 
            hgnc_id: hgnc_id.to_string(), 
            gene_symbol: gene_symbol.to_string(), 
            transcript: transcript.to_string(), 
            allele1: allele1.to_string(), 
            allele2: allele2.to_string(), 
            variant_comment: variant_comment.to_string()
        }
    }

      // Start index is the index in the template matrix where this block of columns starts
    pub fn from_row(
        row: &[String],
        start_idx: usize
    ) -> std::result::Result<Self, String> {
        let i = start_idx;
        let bundle = Self::new(&row[i], &row[i+1],&row[i+2],&row[i+3],&row[i+4],&row[i+5]);
        let _ = bundle.do_qc()?;
        Ok(bundle)
    }

    pub fn do_qc(&self) -> Result<(), String> {
        self.header.qc_bundle(self)?;
        Ok(())
    }

    pub fn to_dto(&self) -> GeneVariantData {
        GeneVariantData:: new(self.hgnc_id(), self.gene_symbol(), self.transcript(), self.allele1(), self.allele2(), self.variant_comment())
    }

    pub fn from_dto(dto: GeneVariantData) -> Self {
        Self { 
            header: SHARED_HEADER.clone(), 
            hgnc_id: dto.hgnc_id, 
            gene_symbol: dto.gene_symbol, 
            transcript: dto.transcript, 
            allele1: dto.allele1, 
            allele2: dto.allele2, 
            variant_comment: dto.variant_comment
        }
    }

    pub fn from_dto_list(dto_list: Vec<GeneVariantData>) -> Vec<Self> {
        dto_list.into_iter()
            .map(Self::from_dto)
            .collect()
    }

    pub fn  hgnc_id(&self) -> &str {
        &self.hgnc_id
    }

    pub fn  gene_symbol(&self) -> &str{
        &self.gene_symbol
    }
    pub fn transcript(&self) -> &str{
        &self.transcript
    }
    pub fn allele1(&self) -> &str{
        &self.allele1
    }
    pub fn  allele2(&self)  ->&str{
        &self.allele2
    }
    pub fn variant_comment(&self)  ->&str{
        &self.variant_comment
    }
}
