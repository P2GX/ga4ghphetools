
use std::{collections::{HashMap, HashSet}, fs::File, io::{BufWriter, Write}, str::FromStr, sync::Arc};
use ontolius::{Identified, TermId, ontology::{HierarchyQueries, HierarchyWalks, OntologyTerms, csr::FullCsrOntology}, term::MinimalTerm};
use crate::dto::{cohort_dto::CohortData, hpo_term_dto::HpoTermDuplet};

/// Count the observed/excluded observations for an HPO term in a cohort
#[derive(Clone, Debug)]
pub struct TermCounter {
    /// The HPO Term that was observed in a cohort
    hpo_duplet: HpoTermDuplet,
    /// The number of times the term was observed
    observed: usize,
    /// The number of times the term was explicitly measured (observed + excluded)
    measured: usize
}

impl TermCounter {
    /// Initialize a TermCounter object with zero counts
    pub fn new(duplet: &HpoTermDuplet) -> Self {
        Self { 
            hpo_duplet: duplet.clone(), 
            observed: 0, 
            measured: 0
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
    pub fn from_pair(
        counter_1: TermCounter,
        counter_2: TermCounter
    ) -> Result<Self, String> {
            let total_o = counter_1.observed + counter_2.observed;
            let total_m = counter_1.measured + counter_2.measured;
            if counter_1.hpo_duplet != counter_2.hpo_duplet {
                return Err(format!("Mismatching HPO duplets Counter 1: {} and Counter 2: {}", 
                            counter_1.hpo_duplet.hpo_label(), counter_2.hpo_duplet.hpo_label()));
            }
            Ok(Self {
                hpo_duplet: counter_1.hpo_duplet.clone(),
                observed: total_o,
                measured: total_m
            })
    }
}

/// The counts from two cohorts and the total
pub struct RowCounter {
    counter_1: TermCounter,
    counter_2: TermCounter,
    total: TermCounter
}


impl RowCounter {
    pub fn new(counter_1: TermCounter, counter_2: TermCounter) -> Result<Self, String> {
        if counter_1.hpo_duplet.hpo_id() != counter_2.hpo_duplet.hpo_id() {
            return Err(format!("Hpo terms must match but we got {} and {}", 
                counter_1.hpo_duplet.hpo_label(), counter_2.hpo_duplet.hpo_label()));
        }
        Ok(Self {
            counter_1: counter_1.clone(),
            counter_2: counter_2.clone(),
            total: TermCounter::from_pair(counter_1, counter_2)?
        })
    }

    fn get_formated_counts(counter: &TermCounter) -> String {
        if counter.measured == 0 {
            return "0/0".to_string();
        }
        let percent = 100.0 * (counter.observed as f64) / (counter.measured as f64);
        format!("{}/{} ({:.1}%)", counter.observed, counter.measured,percent)
    }

    fn hpo_label(&self) -> &str {
        self.counter_1.hpo_duplet.hpo_label()
    }

    fn hpo_id(&self) -> &str {
        self.counter_1.hpo_duplet.hpo_id()
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
            t
        ]
    }

    pub fn get_header() -> Vec<String> {
        vec!["HPO".to_string(),
        "HPO.id".to_string(),
        "Cohort 1".to_string(),
        "Cohort 2".to_string(),
        "Total".to_string()]
    }

    pub fn over_threshold(&self, threshold: usize) -> bool {
        return self.total.measured >= threshold;
    }
}

/// All of the counts for a top level term (Organ), e.g., Abnormality of the liver
pub struct CategoryCounter {
    top_term: HpoTermDuplet,
    row_counter_list: Vec<RowCounter>
}


impl CategoryCounter {
    pub fn new(duplet: &HpoTermDuplet) -> Self {
        Self { 
            top_term: duplet.clone(), 
            row_counter_list: Vec::new()
        }
    }

    pub fn add(&mut self, row_counter: RowCounter) {
        self.row_counter_list.push(row_counter);
    }

    pub fn over_threshold(&self, threshold: usize) -> bool {
        return self.row_counter_list.iter().any(|c| c.over_threshold(threshold));
    }

    pub fn get_subheader(&self) -> Vec<String> {
        vec![self.top_term.hpo_label().to_string(),
            self.top_term.hpo_id().to_string(),
            String::default(),
            String::default(),
            String::default()]
    }
}



pub struct TableCompare {
    category_map: HashMap<HpoTermDuplet, CategoryCounter>,
    hpo: Arc<FullCsrOntology>
}


impl TableCompare {

    pub fn new(cohort_1: CohortData, cohort_2: CohortData, hpo: Arc<FullCsrOntology>) -> Result<Self, String> {
        let cohort_1_map = TableCompare::term_count_map(cohort_1);
        let cohort_2_map = TableCompare::term_count_map(cohort_2);
        let category_map = Self::create_category_map(cohort_1_map, cohort_2_map, hpo.clone())?;

        Ok(Self {
            category_map,
            hpo: hpo.clone()
        })
    }


    fn term_count_map(cohort: CohortData) -> HashMap<HpoTermDuplet, TermCounter> {
        let mut count_map: HashMap<HpoTermDuplet, TermCounter> = HashMap::new();
        for (i, term_duplet) in cohort.hpo_headers.iter().enumerate() {
            let mut term_counter = TermCounter::new(term_duplet);
            for row in &cohort.rows {
                match & row.hpo_data[i] {
                    crate::dto::hpo_term_dto::CellValue::Observed => term_counter.increment_observed(),
                    crate::dto::hpo_term_dto::CellValue::Excluded => term_counter.increment_excluded(),
                    crate::dto::hpo_term_dto::CellValue::Na => {},
                    crate::dto::hpo_term_dto::CellValue::OnsetAge(_) => term_counter.increment_observed(),
                    crate::dto::hpo_term_dto::CellValue::Modifier(_) => term_counter.increment_observed(),
                }
            }
            count_map.insert(term_duplet.clone(), term_counter);
        }
        count_map
    }

    pub fn create_category_map(
        cohort_1_map: HashMap<HpoTermDuplet, TermCounter>,
        cohort_2_map: HashMap<HpoTermDuplet, TermCounter>,
        hpo: Arc<FullCsrOntology>
    ) -> Result<HashMap<HpoTermDuplet, CategoryCounter>, String>{
        let phenotypic_abn = TermId::from_str("HP:0000118").unwrap(); 
        let mut hpo_id_set: HashSet<HpoTermDuplet> = HashSet::new();
        hpo_id_set.extend(cohort_1_map.keys().cloned());
        hpo_id_set.extend(cohort_2_map.keys().cloned());
        let mut top_level_terms: HashSet<HpoTermDuplet> = HashSet::new();
        let mut category_map: HashMap<HpoTermDuplet, CategoryCounter> = HashMap::new();
        for hpo_id in hpo.iter_child_ids(&phenotypic_abn) {
            match hpo.term_by_id(hpo_id) {
                Some(hpo_term) => {
                    let duplet = HpoTermDuplet::new(hpo_term.name(), hpo_term.identifier().to_string());
                    top_level_terms.insert(duplet);
                },
                None => {return Err(format!("Could not find term for {}", hpo_id));},
            }
        }
        for hpo_duplet in hpo_id_set {
            let hpo_tid: TermId = hpo_duplet.to_term_id()?;
            for top_duplet in &top_level_terms {
                let top_tid: TermId = top_duplet.to_term_id()?;
                if hpo.is_descendant_of(&hpo_tid, &top_tid) {
                    let category_counter = category_map.entry(top_duplet.clone()).or_insert(CategoryCounter::new(top_duplet));
                    let term_ctr_1: TermCounter = match cohort_1_map.get(&hpo_duplet) {
                        Some(term_counter) => term_counter.clone(),
                        None => TermCounter::default(&hpo_duplet)   
                    };
                    let term_ctr_2: TermCounter = match cohort_2_map.get(&hpo_duplet) {
                        Some(term_counter) => term_counter.clone(),
                        None => TermCounter::default(&hpo_duplet)   
                    };
                    let row_counter = RowCounter::new(term_ctr_1, term_ctr_2)?;
                    category_counter.add(row_counter)
                }
            }
        }
        Ok(category_map)

    }


    pub fn output_table(&self,  output_path: &str, threshold: usize) -> Result<(), String>{
        let file = File::create(output_path).map_err(|e|e.to_string())?;
        let mut writer = BufWriter::new(file);
        let header =  RowCounter::get_header().join("\t");
        writeln!(writer, "{}",header).map_err(|e|e.to_string())?;
        for (key, value) in &self.category_map {
            if ! value.over_threshold(threshold) {
                continue;
            }
            let sub_header = value.get_subheader().join("\t");
            writeln!(writer, "{}", sub_header).map_err(|e|e.to_string())?;
            for row in &value.row_counter_list {
                if ! row.over_threshold(threshold) {
                    continue;
                }
                let row_str =row.get_row().join("\t");
                writeln!(writer, "{}", row_str).map_err(|e|e.to_string())?;
            }
        }
        Ok(())
    }



}

