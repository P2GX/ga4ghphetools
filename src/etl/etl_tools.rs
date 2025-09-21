use std::collections::hash_map::Entry;
use std::collections::HashSet;
use std::{collections::HashMap, fmt, fs, sync::Arc};
use ontolius::ontology::{csr::FullCsrOntology, MetadataAware};
use regex::Regex;

use crate::dto::cohort_dto::DiseaseData;
use crate::dto::etl_dto::{EtlColumnType::{self, *}, EtlDto};
use crate::dto::hpo_term_dto::CellValue;
use crate::{dto::{cohort_dto::{CohortData, CohortType, IndividualData, RowData}, etl_dto::ColumnTableDto, hpo_term_dto::HpoTermDuplet}, hpo};



pub struct EtlTools {
    /// Reference to the Ontolius Human Phenotype Ontology Full CSR object
    hpo: Arc<FullCsrOntology>,
    /// The data that has been extracted and transformed in a front end
    dto: EtlDto,
}


impl EtlTools {


    pub fn from_dto(
        hpo: Arc<FullCsrOntology>, 
        dto: &EtlDto,
    ) -> Self {
        Self{
            hpo,
            dto: dto.clone(),
        }
    }

    pub fn from_json(
        etl_file_path: &str,
        hpo: Arc<FullCsrOntology>,
    ) -> Result<Self, String> {
        let dto = EtlTools::load_etl_dto_from_json(etl_file_path)?;
        Ok(
            Self {
                hpo,
                dto
            }
        ) 
    }

    pub fn raw_table(&self) -> &EtlDto {
        &self.dto
    }

    // Function to load JSON file and deserialize to ColumnTableDto
    pub fn load_etl_dto_from_json(file_path: &str) -> Result<EtlDto, String> {
        let json_content = fs::read_to_string(file_path)
            .map_err(|e| e.to_string())?;
        let etl_dto: EtlDto = serde_json::from_str(&json_content)
            .map_err(|e| e.to_string())?;
        Ok(etl_dto)
    }



    
    /// Retrieve all HPO Duplets from the Single and Multiple HPO columns
    /// We need this to know how many HPO terms we have altogether for the CohortData
    fn all_hpo_duplets(&self) -> Vec<HpoTermDuplet> {
        self.dto.table.columns.iter()
            .filter_map(|col| {
                match col.header.column_type {
                    EtlColumnType::SingleHpoTerm | EtlColumnType::MultipleHpoTerm => {
                        col.header.hpo_terms.as_ref()
                    },
                    _ => None
                }
            })
            .flatten()
            .cloned()
            .collect()
    }

    /// Extract the string value of of table cell
    fn extract_value(values: &[String], i: usize, field: &str) -> Result<String, String> {
        values.get(i)
            .map(|v| v.to_string())
            .ok_or_else(|| format!("Could not extract {} from column", field))
    }

    /// Get the individual Data for row i
    fn get_individual(&self, i: usize) -> Result<IndividualData, String> {
         let pmid = self.dto.pmid.clone().ok_or_else(|| format!("Could not extract pmid for individual {i}"))?;
         let title: String = self.dto.title.clone().ok_or_else(|| format!("Could not extract title for individual {i}"))?;
         let mut individual = IndividualData{ 
            pmid: pmid, 
            title: title, 
            individual_id: String::default(), 
            comment: String::default(), 
            age_of_onset: "na".to_string(), 
            age_at_last_encounter: "na".to_string(), 
            deceased: "na".to_string(), 
            sex: "na".to_string(), 
         };
   
         for col in &self.dto.table.columns {
            match &col.header.column_type {
                Raw | FamilyId | SingleHpoTerm | MultipleHpoTerm |
                GeneSymbol | Variant | Ignore => {}
                PatientId => {
                    individual.individual_id = Self::extract_value(&col.values, i, "individual ID")?;
                }
                AgeOfOnset => {
                    individual.age_of_onset = Self::extract_value(&col.values, i, "age_of_onset")?;
                }
                AgeAtLastEncounter => {
                    individual.age_at_last_encounter = Self::extract_value(&col.values, i, "age_at_last_encounter")?;
                }
                Sex => {
                    individual.sex = Self::extract_value(&col.values, i, "sex")?;
                }
                Deceased => {
                    individual.deceased = Self::extract_value(&col.values, i, "deceased")?;
                }
            }
        }
        if individual.individual_id.len() < 1 {
            return Err(format!("Invalid individual without identifier: {:?}", individual));
        }
        Ok(individual)
    }


    fn resolve_hpo_conflict(val1: &str, val2: &str) -> Result<String, String> {
        if val1 == "na" {
            return Ok(val2.to_string());
        } else if val2 == "na" {
            return Ok(val1.to_string());
        }
        // if we get here, neither value is "na".
        // if one of the values is excluded and the other is observed or an onset,
        // then we conclude that the relevant HPO term was reported to be present in one of the columns
        // and we return the reported one
        if val1 == "excluded" {
            return Ok(val2.to_string());
        } else if val2 == "excluded" {
            return Ok(val1.to_string());
        }
        // if we get here, then either one of the strings is observed and the other is an onset,
        // or we have two onsets. If one is observed and the other is onset, then we take the
        // onset, this provides more information
        if val1 == "observed" {
            return Ok(val2.to_string());
        } else if val2 == "observed" {
            return Ok(val1.to_string());
        }
        // if we get here, we have two onset terms.
        // todo -- choose the earliest onset
        Err(format!("Conflicting HPO entries: '{}' and '{}'", val1, val2))
    }

    /// We check if there is already an entry for some HPO term in some row. If yes, we throw an
    /// error if the two values disagree.
    fn insert_or_validate(
        map: &mut HashMap<HpoTermDuplet, String>, 
        key: HpoTermDuplet, 
        value: String) 
    -> Result<(), String> {
            match map.entry(key) {
                Entry::Occupied(mut entry) => {
                    if entry.get() != &value {
                        let resolved_val = Self::resolve_hpo_conflict(entry.get(), &value)?;
                        *entry.get_mut() = resolved_val;
                    }
                },
                Entry::Vacant(entry) => {
                    entry.insert(value);
                }
            }
            Ok(())
        }

    /// Insert multiple HPO term observations into the given map.
    ///
    /// The input `value` is expected to be a semicolon-separated list of
    /// `HPO_ID-status` pairs, for example:
    ///
    /// ```text
    /// HP:0011968-excluded;HP:0008947-observed
    /// ```
    ///
    /// Each pair must contain exactly one `-`.  
    /// If a pair is malformed (missing `-` or containing more than one), the
    /// function returns an `Err` with a message indicating the offending observation.
    ///
    /// For each `HpoTermDuplet` in `duplet_list`, the function will:
    /// - Look up its `hpo_id()` in the parsed observation map.
    /// - If found, insert the associated status string into `map`.
    /// - If not found, insert `"na"`.
    ///
    /// # Arguments
    ///
    /// * `map` - A mutable reference to a `HashMap` that will be updated with the results.
    /// * `duplet_list` - A slice of `HpoTermDuplet` values that should be filled with statuses.
    /// * `value` - The raw string containing the semicolon-separated HPO observations.
    ///
    /// # Errors
    ///
    /// Returns `Err(String)` if any observation pair in `value` does not contain
    /// exactly one `-` (empty strings are also considered valid -- this would be "na" for all HPOs).
    fn insert_multiple_hpo_column(
        map: &mut HashMap<HpoTermDuplet, String>, 
        duplet_list: &[HpoTermDuplet], 
        value: String) -> Result<(), String>{
        let observation_list = value.split(";");
        let mut observation_map: HashMap<String, String> = HashMap::new();
        for obs in observation_list {
            if obs.is_empty() {
                continue;
            }
            let obs_pair: Vec<&str> = obs.split("-").collect();
            if obs_pair.len() != 2 {
                return Err(format!("Malformed observation pair '{obs}'"))
            }
            observation_map.insert(obs_pair[0].to_string(), obs_pair[1].to_string());

        }
        for hdup in duplet_list {
        let val = observation_map
            .get(hdup.hpo_id())
            .cloned()
            .unwrap_or_else(|| "na".to_string());
            map.insert(hdup.clone(), val);
        }
        Ok(())
    }


   
    fn get_row(
        &self, 
        i: usize, 
        all_hpo_duplets: &[HpoTermDuplet], 
        disease: &DiseaseData) 
    -> Result<RowData, String> {
         let individual = self.get_individual(i)?;
         let mut hpo_to_status_map: HashMap<HpoTermDuplet, String> = HashMap::new();
         let mut allele_count_map: HashMap<String, usize> = HashMap::new();
         for col in &self.dto.table.columns {
            if col.header.column_type == SingleHpoTerm {
                if let Some(hpo_terms) = &col.header.hpo_terms {
                    if hpo_terms.len() != 1 {
                        return Err(format!(
                            "Expected exactly one HPO term in SingleHpoTerm header '{}' but found {}", 
                            col.header.original, 
                            hpo_terms.len()
                        ));
                    };
                    let single_term = &hpo_terms[0]; 
                    Self::insert_or_validate(&mut hpo_to_status_map, single_term.clone(), col.values[i].clone())?;
                } else {
                    return Err("Could not extract HpoTermDuplet from Single HPO column".to_string());
                }
            } else if col.header.column_type == MultipleHpoTerm {
                if let Some(hpo_terms) = &col.header.hpo_terms {
                    Self::insert_multiple_hpo_column(&mut hpo_to_status_map, hpo_terms, col.values[i].clone())?;
                } else {
                    return Err("Could not extract HpoTermDuplet from Multiple HPO column".to_string());
                }
            } else if col.header.column_type == Variant {
                allele_count_map.entry(col.values[i].clone())
                    .and_modify(|count| *count += 1)
                    .or_insert(1);
            }   
         }
         let mut values: Vec<CellValue> = Vec::new();
         for hpo_duplet in all_hpo_duplets {
            match hpo_to_status_map.get(hpo_duplet) {
                Some(status) => {
                    match status.as_str() {
                        "observed" => { values.push(CellValue::Observed);},
                        "excluded" => { values.push(CellValue::Excluded);},
                        "na" => { values.push(CellValue::Na);},
                        _ => { values.push(CellValue::OnsetAge(status.clone()));}
                    }
                }
                None => {
                    values.push(CellValue::Na);
                }
            }
         }
         let row = RowData{
            individual_data: individual,
            disease_id_list: vec![disease.disease_id.clone()],
            allele_count_map,
            hpo_data: values,
        };
               

        Ok(row)
    }

    pub fn get_row_count(&self) -> Result<usize, String> {
        let first_col = self.dto.table.columns.first()
            .ok_or("No columns in table")?;
        
        let n_rows = first_col.values.len();
        
        if !self.dto.table.columns.iter().all(|col| col.values.len() == n_rows) {
            return Err("Inconsistent column lengths".to_string());
        }
        
        Ok(n_rows)
    }

    /// All all printable ASCII, Latin-1 supplement letters
    fn is_valid_char(ch: char) -> bool {
       if ch.is_ascii_graphic() || ch == ' ' {
            return true;
        }
        if ('\u{00C0}'..='\u{00FF}').contains(&ch) && ch.is_alphabetic() {
            return true;
        }
        false
    }

    pub fn validate_string(s: &str) -> Result<(), String> {
        for ch in s.chars() {
            if !Self::is_valid_char(ch) {
                return Err(format!("Invalid character found: U+{:04X} '{}'", ch as u32, ch));
            }
        }
        Ok(())
    }

    fn qc_table_cells(&self) -> Result<(), String>{
        for col in &self.raw_table().table.columns {
            for cell in &col.values {
                if cell.starts_with(char::is_whitespace) {
                    return Err(format!("{}: leading whitespace - '{}'", col.header.original, cell));
                }
                if cell.ends_with(char::is_whitespace) {
                    return Err(format!("{}: trailing whitespace - '{}'", col.header.original, cell));
                    
                }
                for ch in cell.chars() {
                    if !Self::is_valid_char(ch) {
                        return Err(format!("{}: Invalid character: U+{:04X} '{}'", col.header.original, ch as u32, ch));
                    }
                }
            }
        }
        Ok(())
    }

    /// Check that the alleles in the rows have full variant objects in the maps
    fn qc_variants(&self) -> Result<(), String> {
        let allele_set: HashSet<String> = self
            .raw_table()
            .table
            .columns
            .iter()
            .filter(|col| col.header.column_type == EtlColumnType::Variant)
            .flat_map(|col| col.values.iter().cloned())
            .collect();
        // These alleles must be in either the HGVS or the SV map (i.e., validated)
        for allele in &allele_set {
            if ! self.raw_table().hgvs_variants.contains_key(allele) && 
                ! self.raw_table().structural_variants.contains_key(allele) {
                    return Err(format!("Unmapped allele: '{allele}'"));
                }
        }

        Ok(())
    }

    /// We need to have at least one of individualId and at least one HPO term.
    /// Everything else can in principle be added in the Cohort table page
    fn qc_check_required_columns(&self) -> Result<(), String> {
        let n_individual = self
            .raw_table()
            .table
            .columns
            .iter()
            .filter(|col| col.header.column_type == EtlColumnType::PatientId)
            .take(2) // we only care about 0, 1, or >1
            .count();

        match n_individual {
            0 => return Err("No patient identifier column found".to_string()),
            2 => return Err("Multiple patient identifier columns found".to_string()),
            _ => {}
        }
        let has_hpo = self
            .raw_table()
            .table
            .columns
            .iter()
            .any(|col| matches!(
                col.header.column_type,
                EtlColumnType::SingleHpoTerm | EtlColumnType::MultipleHpoTerm
            ));

        if !has_hpo {
            return Err("No HPO columns found".to_string());
        }

        Ok(())
    }

    fn qc_pmid(&self) -> Result<(), String> {
        let re = Regex::new(r"\bPMID:(\d+)\b").unwrap();
        let pmid_str = match &self.raw_table().pmid {
            Some(p) => p,
            None => return Err("No PMID found".to_string()),
        };
        if !re.is_match(pmid_str) {
            return Err(format!("Malformed PMID found '{}'", pmid_str));
        }
        let title_str = match  &self.raw_table().title  {
            Some(t) => t,
            None => return Err("No title found".to_string()),
        };
        if title_str.len() < 3 {
            return Err(format!("Malformed title: '{title_str}'"))
        }

        Ok(())
    }


    fn qc(&self) -> Result<(), String> {
        if self.raw_table().table.columns.is_empty() {
            return Err("EtlDto table with no columns".to_string());
        }
        for col in &self.raw_table().table.columns {
            if col.header.column_type == EtlColumnType::Raw {
                return Err(format!("'{}' column type not set (Raw)", col.header.original))
            }
        }
        self.qc_table_cells()?;
        self.qc_variants()?;
        self.qc_check_required_columns()?;
        self.qc_pmid()?;
        Ok(())
    }



     /// Note that only Mendelian is supported for Excel file bulk imports
    /// Ohter MOIs are too complicated to be reliably imported in this way.
    pub fn get_cohort_data(&self) -> Result<CohortData, String> {
        self.qc()?;
        let hpo_duplets = Self::all_hpo_duplets(&self);
        let arranged_duplets = hpo::arrange_hpo_duplets(self.hpo.clone(), &hpo_duplets)?;
        let disease = match &self.dto.disease {
            Some(d) => d.clone(),
            None => { return Err(format!("Cannot create CohortData if ETL does not have disease data"))},
        };
        let mut row_list: Vec<RowData> = Vec::new();
        let n_rows = self.get_row_count()?;
        for row_index in 0..n_rows {
            let row = self.get_row(row_index, &arranged_duplets, &disease)?;
            row_list.push(row);
        }
        Ok(CohortData { 
            cohort_type: CohortType::Mendelian, 
            disease_list: vec![disease], 
            hpo_headers: arranged_duplets, 
            rows: row_list, 
            hgvs_variants: self.dto.hgvs_variants.clone(), 
            structural_variants: self.dto.structural_variants.clone(), 
            phetools_schema_version: CohortData::phenopackets_schema_version(), 
            hpo_version: self.hpo.version().to_string(), 
            cohort_acronym: None 
        })
    }
  
}




impl fmt::Display for ColumnTableDto {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "File: {}", self.file_name)?;
        writeln!(f, "Columns:")?;

        for column in &self.columns {
            let first_value = column.values.first().cloned().unwrap_or_else(|| "<empty>".to_string());
            writeln!(f, "- {}: {}", column.header.original, first_value)?;
        }

        Ok(())
    }
}

