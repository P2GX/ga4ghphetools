//! TableCompare: Compare two sets of phenopacketes with respect to the distribution of HPO terms
use crate::dto::{
    cohort_dto::CohortData,
    hpo_term_dto::{CellValue, HpoTermDuplet},
};
use ontolius::{
    ontology::{csr::FullCsrOntology, HierarchyQueries, HierarchyWalks, OntologyTerms},
    term::MinimalTerm,
    Identified, TermId,
};
use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::{BufWriter, Write},
    str::FromStr,
    sync::Arc,
};


/// SImple struct to keep track of PMID references
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct Citation {
    pmid: String,
    title: String
}

impl Citation {
    pub fn new (pmid: String, title: String) -> Self {
        Self {
            pmid,
            title
        }
    }

    pub fn pmid(&self) -> &str {
        &self.pmid
    }

    pub fn title(&self) -> &str {
        &self.title
    }
}

/// Count the observed/excluded observations for an HPO term in a cohort
#[derive(Clone, Debug)]
pub struct TermCounter {
    /// The HPO Term that was observed in a cohort
    hpo_duplet: HpoTermDuplet,
    /// The number of times the term was observed
    observed: usize,
    /// The number of times the term was explicitly measured (observed + excluded)
    measured: usize,
}

impl TermCounter {
    /// Initialize a TermCounter object with zero counts
    pub fn new(duplet: &HpoTermDuplet) -> Self {
        Self {
            hpo_duplet: duplet.clone(),
            observed: 0,
            measured: 0,
        }
    }

    pub fn increment_observed(&mut self) {
        self.observed += 1;
        self.measured += 1;
    }

    pub fn increment_excluded(&mut self) {
        self.measured += 1;
    }

    pub fn default(duplet: &HpoTermDuplet) -> Self {
        Self::new(duplet)
    }

    /// Combine the counts from two cohorts to get the totals
    pub fn from_pair(counter_1: TermCounter, counter_2: TermCounter) -> Result<Self, String> {
        let total_o = counter_1.observed + counter_2.observed;
        let total_m = counter_1.measured + counter_2.measured;
        if counter_1.hpo_duplet != counter_2.hpo_duplet {
            return Err(format!(
                "Mismatching HPO duplets Counter 1: {} and Counter 2: {}",
                counter_1.hpo_duplet.hpo_label(),
                counter_2.hpo_duplet.hpo_label()
            ));
        }
        Ok(Self {
            hpo_duplet: counter_1.hpo_duplet.clone(),
            observed: total_o,
            measured: total_m,
        })
    }
}

/// The counts from two cohorts and the total
#[derive(Clone, Debug)]
pub struct RowCounter {
    counter_1: TermCounter,
    counter_2: TermCounter,
    total: TermCounter,
}

impl RowCounter {
    pub fn new(counter_1: TermCounter, counter_2: TermCounter) -> Result<Self, String> {
        if counter_1.hpo_duplet.hpo_id() != counter_2.hpo_duplet.hpo_id() {
            return Err(format!(
                "Hpo terms must match but we got {} and {}",
                counter_1.hpo_duplet.hpo_label(),
                counter_2.hpo_duplet.hpo_label()
            ));
        }
        Ok(Self {
            counter_1: counter_1.clone(),
            counter_2: counter_2.clone(),
            total: TermCounter::from_pair(counter_1, counter_2)?,
        })
    }

    fn get_formated_counts(counter: &TermCounter) -> String {
        if counter.measured == 0 {
            return "0/0".to_string();
        }
        let percent = 100.0 * (counter.observed as f64) / (counter.measured as f64);
        format!(
            "{}/{} ({:.1}%)",
            counter.observed, counter.measured, percent
        )
    }

    fn hpo_label(&self) -> &str {
        self.counter_1.hpo_duplet.hpo_label()
    }

    fn hpo_id(&self) -> &str {
        self.counter_1.hpo_duplet.hpo_id()
    }

    pub(crate) fn duplet(&self) -> &HpoTermDuplet {
        &self.counter_1.hpo_duplet
    }

    pub fn get_row(&self) -> Vec<String> {
        let c1 = Self::get_formated_counts(&self.counter_1);
        let c2 = Self::get_formated_counts(&self.counter_2);
        let t = Self::get_formated_counts(&self.total);
        vec![
            self.hpo_label().to_string(),
            self.hpo_id().to_string(),
            c1,
            c2,
            t,
        ]
    }

    pub fn get_header() -> Vec<String> {
        vec![
            "HPO".to_string(),
            "HPO.id".to_string(),
            "Cohort 1".to_string(),
            "Cohort 2".to_string(),
            "Total".to_string(),
        ]
    }

    pub fn over_threshold(&self, threshold: usize) -> bool {
        return self.total.measured >= threshold;
    }
}

/// All of the counts for a top level term (Organ), e.g., Abnormality of the liver
#[derive(Clone, Debug)]
pub struct CategoryCounter {
    top_term: HpoTermDuplet,
    pub(crate) row_counter_list: Vec<RowCounter>,
}

impl CategoryCounter {
    pub fn new(duplet: &HpoTermDuplet) -> Self {
        Self {
            top_term: duplet.clone(),
            row_counter_list: Vec::new(),
        }
    }

    pub fn add(&mut self, row_counter: RowCounter) {
        self.row_counter_list.push(row_counter);
    }

    pub fn over_threshold(&self, threshold: usize) -> bool {
        self
            .row_counter_list
            .iter()
            .any(|c| c.over_threshold(threshold))
    }

    pub fn get_subheader(&self) -> Vec<String> {
        vec![
            self.top_term.hpo_label().to_string(),
            self.top_term.hpo_id().to_string(),
            String::default(),
            String::default(),
            String::default(),
        ]
    }
}

/// Structure to coordinate the comparison of the term distribution of two cohorts
pub struct TableCompare {
    /// Key: An HPO term; Value: A [`CategoryCounter`] object that holds counts for each of the two cohorts and the total
    category_map: HashMap<HpoTermDuplet, CategoryCounter>,
    /// Key, an HPO Term; Value: total number of times the term was mentioned explicitly in both groups
    total_term_count_map: HashMap<HpoTermDuplet, usize>,
    /// List of PMID references used in the cohort
    citation_list: Vec<Citation>,
    /// Reference to the Human Phenotype Ontology
    hpo: Arc<FullCsrOntology>,
}

impl TableCompare {
    pub fn new(
        cohort_1: CohortData,
        cohort_2: CohortData,
        hpo: Arc<FullCsrOntology>,
    ) -> Result<Self, String> {
        let citations = Self::extract_pmid_citations(&cohort_1, &cohort_2);
        let total_term_count_map = Self::calculate_total_counts(&cohort_1, &cohort_2);
        let cohort_1_map = TableCompare::term_count_map(cohort_1, hpo.clone())?;
        let cohort_2_map = TableCompare::term_count_map(cohort_2, hpo.clone())?;
        let category_map = Self::create_category_map(cohort_1_map, cohort_2_map, hpo.clone())?;
      
        Ok(Self {
            category_map,
            total_term_count_map,
            citation_list: citations,
            hpo: hpo.clone(),
        })
    }

    /// Get a Map with key, an HpoTermDuplet, value -- number of times term is annotated in cohort
    /// We include ancestors of direct terms
    fn term_count_map(
        cohort: CohortData,
        hpo: Arc<FullCsrOntology>,
    ) -> Result<HashMap<HpoTermDuplet, TermCounter>, String> {
        let mut count_map: HashMap<HpoTermDuplet, TermCounter> = HashMap::new();
        for (i, term_duplet) in cohort.hpo_headers.iter().enumerate() {
            // Make sure direct entry is represented in the map
            count_map
                .entry(term_duplet.clone())
                .or_insert_with(|| TermCounter::new(term_duplet));
            for row in &cohort.rows {
                let increment_action = match &row.hpo_data[i] {
                    CellValue::Observed | CellValue::OnsetAge(_) | CellValue::Modifier(_) => {
                        TermCounter::increment_observed
                    }
                    CellValue::Excluded => TermCounter::increment_excluded,
                    CellValue::Na => continue,
                };
                increment_action(count_map.get_mut(term_duplet).unwrap());
                // increment ancestors
                let term_id = TermId::from_str(&term_duplet.hpo_id).map_err(|e| e.to_string())?;
                for ancestor_id in hpo.iter_ancestor_ids(&term_id) {
                    if let Some(anc_term) = hpo.term_by_id(ancestor_id) {
                        let ancestor_duplet =
                            HpoTermDuplet::new(anc_term.name(), anc_term.identifier().to_string());
                        let counter = count_map
                            .entry(ancestor_duplet.clone())
                            .or_insert_with(|| TermCounter::new(&ancestor_duplet));
                        increment_action(counter);
                    }
                }
            }
        }
        Ok(count_map)
    }

    /// Get total number of times a term was used explicitly in the annotations of both cohorts (how many individuals)
    fn calculate_total_counts(
        cohort_1: &CohortData,
        cohort_2: &CohortData,
    ) -> HashMap<HpoTermDuplet, usize> {
        let mut count_map: HashMap<HpoTermDuplet, usize> = HashMap::new();
        for (i, term_duplet) in cohort_1.hpo_headers.iter().enumerate() {
            for row in &cohort_1.rows {
                if row.hpo_data[i] == crate::dto::hpo_term_dto::CellValue::Na {
                    continue;
                }
                *count_map.entry(term_duplet.clone()).or_insert(0) += 1;
            }
        }
        for (i, term_duplet) in cohort_2.hpo_headers.iter().enumerate() {
            for row in &cohort_2.rows {
                if row.hpo_data[i] == crate::dto::hpo_term_dto::CellValue::Na {
                    continue;
                }
                *count_map.entry(term_duplet.clone()).or_insert(0) += 1;
            }
        }
        count_map
    }

    fn extract_pmid_citations(cohort_1: &CohortData, cohort_2: &CohortData) 
        -> Vec<Citation> {
        let mut citation_set: HashSet<Citation> = HashSet::new();
        for row in cohort_1.rows.iter() {
            let pmid = row.individual_data.pmid.clone();
            let title = row.individual_data.title.clone();
            let citation = Citation::new(pmid, title);
            citation_set.insert(citation);
        };
        for row in cohort_2.rows.iter() {
            let pmid = row.individual_data.pmid.clone();
            let title = row.individual_data.title.clone();
            let citation = Citation::new(pmid, title);
            citation_set.insert(citation);
        };
        let citation_list: Vec<Citation> = citation_set.into_iter().collect();
        citation_list
    }

    pub fn get_citations(&self) -> Vec<Citation> {
        self.citation_list.clone()
    }


    pub fn create_category_map(
        cohort_1_map: HashMap<HpoTermDuplet, TermCounter>,
        cohort_2_map: HashMap<HpoTermDuplet, TermCounter>,
        hpo: Arc<FullCsrOntology>,
    ) -> Result<HashMap<HpoTermDuplet, CategoryCounter>, String> {
        let phenotypic_abn = TermId::from_str("HP:0000118").unwrap();
        let mut hpo_id_set: HashSet<HpoTermDuplet> = HashSet::new();
        hpo_id_set.extend(cohort_1_map.keys().cloned());
        hpo_id_set.extend(cohort_2_map.keys().cloned());
        let mut top_level_terms: HashSet<HpoTermDuplet> = HashSet::new();
        let mut category_map: HashMap<HpoTermDuplet, CategoryCounter> = HashMap::new();
        for hpo_id in hpo.iter_child_ids(&phenotypic_abn) {
            match hpo.term_by_id(hpo_id) {
                Some(hpo_term) => {
                    let duplet =
                        HpoTermDuplet::new(hpo_term.name(), hpo_term.identifier().to_string());
                    top_level_terms.insert(duplet);
                }
                None => {
                    return Err(format!("Could not find term for {}", hpo_id));
                }
            }
        }
        for hpo_duplet in hpo_id_set {
            let hpo_tid: TermId = hpo_duplet.to_term_id()?;
            for top_duplet in &top_level_terms {
                let top_tid: TermId = top_duplet.to_term_id()?;
                if hpo.is_descendant_of(&hpo_tid, &top_tid) {
                    let category_counter = category_map
                        .entry(top_duplet.clone())
                        .or_insert(CategoryCounter::new(top_duplet));
                    let term_ctr_1: TermCounter = match cohort_1_map.get(&hpo_duplet) {
                        Some(term_counter) => term_counter.clone(),
                        None => TermCounter::default(&hpo_duplet),
                    };
                    let term_ctr_2: TermCounter = match cohort_2_map.get(&hpo_duplet) {
                        Some(term_counter) => term_counter.clone(),
                        None => TermCounter::default(&hpo_duplet),
                    };
                    let row_counter = RowCounter::new(term_ctr_1, term_ctr_2)?;
                    category_counter.add(row_counter)
                }
            }
        }
        Ok(category_map)
    }

    pub fn get_category_counters(&self) -> Vec<CategoryCounter> {
        self.category_map.values().cloned().collect()
    }

    pub(crate) fn get_count(&self, duplet: &HpoTermDuplet) -> usize {
        match self.total_term_count_map.get(duplet) {
            Some(c) => *c,
            None => 0,
        }
    }


    pub fn output_table(&self, output_path: &str, threshold: usize) -> Result<(), String> {
        let file = File::create(output_path).map_err(|e| e.to_string())?;
        let mut writer = BufWriter::new(file);
        let header = RowCounter::get_header().join("\t");
        writeln!(writer, "{}", header).map_err(|e| e.to_string())?;
        for cat_counter in self.category_map.values() {
            if !cat_counter.over_threshold(threshold) {
                continue;
            }
            let sub_header = cat_counter.get_subheader().join("\t");
            writeln!(writer, "{}", sub_header).map_err(|e| e.to_string())?;
            for row in &cat_counter.row_counter_list {
                if let Some(count) = self.total_term_count_map.get(row.duplet()) {
                    if *count > threshold {
                        let row_str = row.get_row().join("\t");
                        writeln!(writer, "{}", row_str).map_err(|e| e.to_string())?;
                    }
                }
                if !row.over_threshold(threshold) {
                    continue;
                }
            }
        }
        Ok(())
    }
}
