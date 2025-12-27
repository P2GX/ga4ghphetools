//! Some shared functions for the variant validators
//! 
//! The HGVS and intergenic-HGVS validators share some of the same JSON-extraction code here

use serde_json::Value;
use crate::variant::vcf_var::VcfVar;

pub trait VariantValidatorHandler {
    fn extract_variant_validator_warnings(&self, response: &Value) -> Result<(), String> {
        // 1. Check if the global flag is "warning"
        if response.get("flag").and_then(|f| f.as_str()) != Some("warning") {
            return Ok(());
        }

        // 2. Look through all keys (e.g., intergenic_variant_1, validation_warning_1, etc.)
        if let Some(obj) = response.as_object() {
            for (key, val) in obj {
                // We are looking for any sub-object that contains the "validation_warnings" key
                if let Some(warnings) = val.get("validation_warnings").and_then(|w| w.as_array()) {
                    // Grab the first message if it exists
                    if let Some(first_msg) = warnings.iter().filter_map(|w| w.as_str()).next() {
                        return Err(first_msg.to_string());
                    }
                }
            }
        }

        // 3. Fallback if "warning" flag was set but no warning text was found
        Err(format!(
            "[variant_validator: {}:{}] Identified as warning but no messages found",
            file!(),
            line!()
        ))
    }

    /// Dynamically finds the variant data block by skipping boilerplate keys
    /// os_object(): treats the JSON response as a Map/Dictionary.
    /// The find command lists all keys that are not one of the standard boilerplate keys.
    /// This will be the name that we use to extract the information about our variant from the JSON object
    fn get_variant_data<'a>(&self, response: &'a Value) -> Result<&'a Value, String> {
        let variant_key = response.as_object()
            .ok_or_else(|| "Response is not a JSON object".to_string())?
            .keys()
            .find(|&k| k != "flag" && k != "metadata")
            .ok_or_else(|| "Missing variant data key in response".to_string())?;

        Ok(&response[variant_key])
    }

    /// Extracts the assembly block from the VariantValidator response 
    /// and checks that we have the correct assemlby (hg38 for now)object that looks like this:
    /// "grch38": {
    ///    "hgvs_genomic_description": "NC_000019.10:g.12887294G>A",
    ///    "vcf": {
    ///      "alt": "A",
    ///      "chr": "19",
    ///      "pos": "12887294",
    ///      "ref": "G"
    ///    }
    ///  },
    fn get_assembly_block<'a>(
        &self, 
        var: &'a serde_json::Value, 
        genome_assembly: &str
    ) -> Result<&'a serde_json::Value, String> {
        let assemblies = var.get("primary_assembly_loci")
            .ok_or_else(|| "Missing primary_assembly_loci".to_string())?;

        let assembly = assemblies.get(genome_assembly)
            .ok_or_else(|| format!("Could not identify assembly block for {} in response", genome_assembly))?;

        Ok(assembly)
    }

    /// Extracts the HGVS genomic string (e.g., NC_000019.10:g.12887294G>A) from the JSON object (assembly) 
    /// returned by get_assembly_block
    fn get_genomic_hgvs<'a>(
        &self,
        assembly: &'a serde_json::Value) -> Result<String, String> {
        let genomic_hgvs = assembly.get("hgvs_genomic_description")
            .and_then(|v| v.as_str())
            .map(str::to_string)
            .ok_or_else(|| "Missing field: hgvs_genomic_description".to_string())?;
        return Ok(genomic_hgvs);
    }

    /// Get the gene symbol (returns an option, because some intergenic variants may not have a gene)
    fn get_gene_symbol(&self, var_data: &serde_json::Value) -> Option<String> {
        var_data.get("gene_symbol")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }

    fn get_hgnc(&self, var_data: &serde_json::Value) -> Option<String> {
        var_data.get("gene_ids")
            .and_then(|ids| ids.get("hgnc_id"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }

       // The following will either be a String or None, and can be assigned to an Option<String>
        // if we have a non-coding RNA variant, e.g., n.4G>A, then this will evaluate to None
    fn get_hgvs_predicted_protein_consequence(&self, var_data: &serde_json::Value) -> Option<String> {
        var_data.get("hgvs_predicted_protein_consequence")
            .and_then(|hgvs_protein| hgvs_protein.get("tlr"))
            .and_then(|tlr| tlr.as_str())
            .filter(|s| !s.is_empty())   // this will turn empty string ("") into None
            .map(|s| s.to_string())
    }

    /// Extracts the VCF fields from the JSON object (assembly) 
    /// returned by get_assembly_block
    fn get_vcf_var<'a>(&self,
                    assembly: &'a serde_json::Value) 
        -> Result<VcfVar, String> 
    {
        let vcf = assembly.get("vcf")
            .ok_or_else(|| "Could not identify vcf element".to_string())?;
        let chrom: String = vcf.get("chr")
                .and_then(Value::as_str)
                .ok_or_else(|| format!("Malformed chr: {:?}", vcf))? 
                .to_string();
            let position: u32 = vcf.get("pos")
            .and_then(Value::as_str) // "pos" is stored as a string
            .ok_or_else(|| format!("Malformed pos: {:?}", vcf))? 
            .parse() 
            .map_err(|e| format!("Error '{}'", e))?; 
        let reference = vcf.get("ref").
            and_then(Value::as_str)
            .ok_or_else(|| format!("Malformed REF: '{:?}'", vcf))?
            .to_string();
        let alternate = vcf.get("alt").
            and_then(Value::as_str)
            .ok_or_else(|| format!("Malformed ALT: '{:?}'", vcf))?
            .to_string();
        let vcf_var = VcfVar::new(chrom, position, reference, alternate);
        Ok(vcf_var)
    }

    }

     