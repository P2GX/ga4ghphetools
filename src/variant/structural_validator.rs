use crate::{dto::variant_dto::VariantDto, variant::structural_variant::StructuralVariant};




const GENOME_ASSEMBLY_HG38: &str = "hg38";

const ACCEPTABLE_GENOMES: [&str; 2] = [ "GRCh38",  "hg38"];


pub struct StructuralValidator {
    genome_assembly: String,
}

impl StructuralValidator {
    
    pub fn new(genome_build: &str) -> Result<Self, String> {
        if !ACCEPTABLE_GENOMES.contains(&genome_build) {
            return Err(format!("genome_build \"{}\" not recognized", genome_build));
        }
        Ok(Self {
            genome_assembly: genome_build.to_string(),
        })
    }

    pub fn hg38() -> Self {
        Self {
            genome_assembly: GENOME_ASSEMBLY_HG38.to_string(),
        }
    }

    pub fn validate_sv(&mut self, dto: &VariantDto) -> Result<StructuralVariant, String> {
        let mut parts = dto.variant_string().splitn(2, ':');
        let prefix = parts.next().unwrap_or("").trim();  // "DEL"
        let rest = parts.next().unwrap_or("").trim(); 
        if ! rest.is_ascii() {
            return Err(format!("Variant string {} contains non-ASCII character", rest));
        }
        match prefix {
            "DEL" => StructuralVariant::code_as_chromosomal_deletion(rest, dto),
            "INV" => StructuralVariant::code_as_chromosomal_inversion(rest, dto),
            "TRANSL" => StructuralVariant::code_as_chromosomal_translocation(rest, dto),
            "DUP" => StructuralVariant::code_as_chromosomal_duplication(rest, dto),
            "SV" => StructuralVariant::code_as_chromosomal_structure_variation(rest, dto),
            _ =>  Err(format!("Did not recognize structural variant prefix '{prefix}' in {}", dto.variant_string()))
        }
    }

}



#[cfg(test)]
mod tests {
    use rstest::{fixture, rstest};

    use super::*;


    #[fixture]
    fn invalid_sv1() -> VariantDto {
        VariantDto::new_sv("DEL: arr 16q24.3 DEL89,754,790 −89,757,400", "NM_052988.5", "HGNC:1770", "CDK10")
    }

    fn valid_sv1() -> VariantDto {
        VariantDto::new_sv("DEL: arr 16q24.3 DEL89,754,790-89,757,400", "NM_052988.5", "HGNC:1770", "CDK10")
    }

    

    #[rstest]
    fn test_valid_sv()  {
        let dto = valid_sv1();
        let mut validator = StructuralValidator::hg38();
        let result = validator.validate_sv(&dto);
        assert!(result.is_ok());
    }


    #[rstest]
    fn test_invalid_sv()  {
        let dto = invalid_sv1();
        let mut validator = StructuralValidator::hg38();
        let result = validator.validate_sv(&dto);
        assert!(result.is_err());
        let msg = result.err().unwrap();
        let expected = "Variant string arr 16q24.3 DEL89,754,790 −89,757,400 contains non-ASCII character";
        assert_eq!(expected, msg);
    }

    
}

