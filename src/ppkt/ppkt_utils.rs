use phenopackets::schema::v2::core::Interpretation;
use phenopackets::schema::v2::core::genomic_interpretation::Call;



pub fn get_gene_symbol_from_interpretation(interpretation: &Interpretation) -> Option<String> {
    let diagx = interpretation.diagnosis.as_ref()?;
    // As a rule, for Mendelian, we will have a single interpretation per disease. We do not support digenic etc.!
    let genomic_interpretation = diagx.genomic_interpretations.first()?;
        let call = genomic_interpretation.call.as_ref()?;
        match call {
            Call::Gene(gene_descriptor) => {
                let symbol = gene_descriptor.symbol.clone();
                return Some(symbol);
            },
            Call::VariantInterpretation(variant_interpretation) => {
                let variant_descriptor = variant_interpretation.variation_descriptor.as_ref()?;
                let gene_context = variant_descriptor.gene_context.as_ref()?;
                let symbol = gene_context.symbol.clone();
                return Some(symbol);
            },
        }
    }




#[cfg(test)]
mod test {
use phenopackets::{ga4gh::vrsatile::v1::{GeneDescriptor, VariationDescriptor}, schema::v2::core::{Diagnosis, GenomicInterpretation, Interpretation, VariantInterpretation, genomic_interpretation::Call}};

use rstest::*;

use crate::ppkt::get_gene_symbol_from_interpretation;

// --- FIXTURES: Building Blocks for our Harness ---

#[fixture]
fn base_interpretation() -> Interpretation {
    Interpretation {
        id: "interp-001".to_string(),
        diagnosis: Some(Diagnosis::default()),
        ..Default::default()
    }
}

// --- TESTS ---

#[rstest]
fn test_get_gene_symbol_from_gene_descriptor(mut base_interpretation: Interpretation) {
    // 1. Arrange: Build a direct GeneDescriptor call variant
    let gene_desc = GeneDescriptor {
        symbol: "FBN1".to_string(),
        ..Default::default()
    };
    
    let genomic_interp = GenomicInterpretation {
        call: Some(Call::Gene(gene_desc)),
        ..Default::default()
    };

    // Inject into our base fixture structure safely
    if let Some(ref mut diag) = base_interpretation.diagnosis {
        diag.genomic_interpretations.push(genomic_interp);
    }

    // 2. Act: Execute your extraction logic
    let result = get_gene_symbol_from_interpretation(&base_interpretation);

    // 3. Assert
    assert_eq!(result, Some("FBN1".to_string()));
}

#[rstest]
fn test_get_gene_symbol_from_variant_interpretation(mut base_interpretation: Interpretation) {
    // 1. Arrange: Build the deeply nested VariantInterpretation layout
    let gene_context = GeneDescriptor {
        symbol: "SCN1A".to_string(),
        ..Default::default()
    };

    let variant_desc = VariationDescriptor {
        gene_context: Some(gene_context),
        ..Default::default()
    };

    let variant_interp = VariantInterpretation {
        variation_descriptor: Some(variant_desc), // matches schema spec field name
        ..Default::default()
    };

    let genomic_interp = GenomicInterpretation {
        call: Some(Call::VariantInterpretation(variant_interp)),
        ..Default::default()
    };

    if let Some(ref mut diag) = base_interpretation.diagnosis {
        diag.genomic_interpretations.push(genomic_interp);
    }

    // 2. Act
    let result = get_gene_symbol_from_interpretation(&base_interpretation);

    // 3. Assert
    assert_eq!(result, Some("SCN1A".to_string()));
}

#[rstest]
fn test_returns_none_when_diagnosis_is_missing() {
    // Arrange: Build an interpretation with absolutely no diagnosis element block
    let incomplete_interp = Interpretation {
        id: "interp-empty".to_string(),
        diagnosis: None,
        ..Default::default()
    };

    // Act
    let result = get_gene_symbol_from_interpretation(&incomplete_interp);

    // Assert
    assert!(result.is_none());
}

    
}