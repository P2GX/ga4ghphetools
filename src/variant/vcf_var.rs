

pub struct VcfVar {
    chromosome: String,
    pos: u32,
    reference: String,
    alternate: String
}


impl VcfVar {
    pub fn new(
        chromosome: impl Into<String>,
        pos: u32,
        reference: impl Into<String>,
        alternate: impl Into<String>
    ) -> Self
    {
        VcfVar { 
            chromosome: chromosome.into(), 
            pos: pos, 
            reference: reference.into(), 
            alternate: alternate.into() 
        }
    }

    pub fn chrom(&self) -> String {
        self.chromosome.clone()
    }

    pub fn pos(&self) -> u32 {
        self.pos
    }

    pub fn ref_allele(&self) -> String {
        self.reference.clone()
    }
    
    pub fn alt_allele(&self) -> String {
        self.alternate.clone()
    }
}

// region:    --- Tests

#[cfg(test)]
mod tests {
 
    use super::*;

    #[test]
    fn test_name() {
        let vvar = VcfVar::new("chr1", 1234, "A", "T");
        assert_eq!("T", vvar.alt_allele());
        assert_eq!("A", vvar.ref_allele());
        assert_eq!(1234, vvar.pos());
        assert_eq!("chr1", vvar.chrom());
    }
}

// endregion: --- Tests