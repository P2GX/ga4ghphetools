use crate::{dto::cohort_dto::GeneVariantDto, header::duplet_item::DupletItem, template::gene_variant_bundle::GeneVariantBundle};



#[derive(Clone, Debug)]
pub struct GeneVariantHeader {
    pub hgnc_id: DupletItem,
    pub gene_symbol: DupletItem,
    pub transcript: DupletItem,
    pub allele1: DupletItem,
    pub allele2: DupletItem,
    pub variant_comment: DupletItem,
}

impl GeneVariantHeader {
    pub fn new() -> Self {
        Self { 
            hgnc_id: DupletItem::hgnc_id(), 
            gene_symbol: DupletItem::gene_symbol(), 
            transcript: DupletItem::transcript(), 
            allele1: DupletItem::allele1(), 
            allele2: DupletItem::allele2(), 
            variant_comment: DupletItem::variant_comment() 
        }
    }

     /// Perform quality control on the labels of the two header rows for a GeneVariantBundle
     /// Input is a matrix generated from a tabular input file 
     /// We need the start index because for melded phenotypes or digenic there are two GeneVariantBundles
    pub fn from_matrix(
        matrix: &Vec<Vec<String>>,
        start_idx: usize
    ) -> Result<Self, String> {
        let header = GeneVariantHeader::new();
        if matrix.len() < 2 {
            return Err(format!("Empty template with {} rows.", matrix.len()));
        }
        let mut i = start_idx;
        header.hgnc_id.check_column_labels(&matrix, i)?;
        i += 1;
        header.gene_symbol.check_column_labels(&matrix, i)?;
        i += 1;
        header.transcript.check_column_labels(&matrix, i)?;
        i += 1;
        header.allele1.check_column_labels(&matrix, i)?;
        i += 1;
        header.allele2.check_column_labels(&matrix, i)?;
        i += 1;
        header.variant_comment.check_column_labels(&matrix, i)?;
        Ok(header)
        
    }

    /// Check an GeneVariant bundle for errors.
    pub fn qc_dto(&self, dto: &GeneVariantDto) -> Result<(), String> {
        self.qc_data(&dto.hgnc_id, &dto.gene_symbol, &dto.transcript, &dto.allele1, &dto.allele2, &dto.variant_comment)
    }

    pub fn qc_bundle(&self, bundle: &GeneVariantBundle) -> Result<(), String> {
        self.qc_data(&bundle.hgnc_id, &bundle.gene_symbol, &bundle.transcript, &bundle.allele1, &bundle.allele2, &bundle.variant_comment)
    }

    pub fn qc_data(&self, hgnc_id: &str, gene_symbol:  &str, transcript:  &str, allele1:  &str, allele2:  &str, variant_comment:  &str)
    -> Result<(), String> {
        self.hgnc_id.qc_data(hgnc_id)?;
        self.gene_symbol.qc_data(gene_symbol)?;
        self.transcript.qc_data(transcript)?;
        self.allele1.qc_data(allele1)?;
        self.allele2.qc_data(allele2)?;
        self.variant_comment.qc_data(variant_comment)?;
            Ok(())
    }


    
}