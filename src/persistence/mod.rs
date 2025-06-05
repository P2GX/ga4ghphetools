pub mod dir_manager;


pub trait ValidatorOfVariants: Send + Sync {
    fn validate_hgvs(&mut self, variant: &str, transcript: &str) -> Result<(), String>;
    fn validate_sv(&mut self, variant: &str, hgnc_id: &str, gene_symbol: &str) -> Result<(), String>;
}



