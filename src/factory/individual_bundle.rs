/// IndividualBundle
/// Used to ingest the legacy Excel pyphetools templates
/// We will refactor this to use the IndividualDto
use std::sync::Arc;
use once_cell::sync::Lazy;

use crate::{dto::cohort_dto::IndividualData, header::individual_header::IndividualHeader};


static SHARED_HEADER: Lazy<Arc<IndividualHeader>> = Lazy::new(|| {
    Arc::new(IndividualHeader::new())
});

#[derive(Clone, Debug)]
pub struct IndividualBundle {
    header: Arc<IndividualHeader>,
    pub(crate) pmid: String,
    pub(crate) title: String,
    pub(crate) individual_id: String,
    pub(crate) comment: String,
    pub(crate) age_of_onset: String,
    pub(crate) age_at_last_encounter: String,
    pub(crate) deceased: String,
    pub(crate) sex: String
}



impl IndividualBundle {
    pub fn new(
        pmid: &str, 
        title: &str,
        individual_id: &str,
        comment: &str,
        age_of_onset: &str,
        age_at_last_encounter: &str,
        deceased: &str,
        sex: &str) 
    -> Self {
        Self { 
            header: SHARED_HEADER.clone(), 
            pmid: pmid.trim().to_string(), 
            title: title.trim().to_string(), 
            individual_id: individual_id.trim().to_string(), 
            comment: comment.trim().to_string(),
            age_of_onset: age_of_onset.trim().to_string(),
            age_at_last_encounter: age_at_last_encounter.trim().to_string(),
            deceased: deceased.trim().to_string(),
            sex: sex.trim().to_string()
        }
    }

    /// Start idx is the index of the first demographic entry.
    /// We should consider changing the format to put the demographics right after individual.
    pub fn from_row(
        row: &Vec<String>,
        start_idx: usize
    ) -> std::result::Result<Self, String> {
        let  i = start_idx;
        let bundle = Self::new(&row[0], &row[1], &row[2], &row[3], &row[i], &row[i+1], &row[i+2], &row[i+3]);
        bundle.do_qc()?;
        Ok(bundle)
    }

    pub fn do_qc(&self) -> Result<(), String> {
        self.header.qc_bundle(self)
    }

    pub fn pmid(&self) -> &str {
        &self.pmid
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn individual_id(&self) -> &str {
        &self.individual_id
    }

    pub fn comment(&self) -> &str {
        &self.comment
    }

    pub fn age_of_onset(&self) -> &str {
        &self.age_of_onset
    }

    pub fn age_at_last_encounter(&self) -> &str {
        &self.age_at_last_encounter
    }

    pub fn deceased(&self) -> &str {
        &self.deceased
    }

    pub fn sex(&self) -> &str {
        &self.sex
    }

    pub fn from_dto(dto: IndividualData) -> Self {
        Self { 
            header: SHARED_HEADER.clone(), 
            pmid: dto.pmid, 
            title: dto.title, 
            individual_id: dto.individual_id, 
            comment: dto.comment, 
            age_of_onset: dto.age_of_onset, 
            age_at_last_encounter: dto.age_at_last_encounter, 
            deceased: dto.deceased, 
            sex: dto.sex 
        }
    }

}

#[cfg(test)]
mod test {
    use rstest::{fixture, rstest};
    use crate::factory::individual_bundle::IndividualBundle;


    #[fixture]
    fn pmid() -> &'static str {
        return "PMID:29482508";
    }

    #[fixture]
    fn title() -> &'static str {
        return "Difficult diagnosis and genetic analysis of fibrodysplasia ossificans progressiva: a case report";
    }
  #[fixture]
    fn individual_id() -> &'static str {
        return "individual A";
    }

    #[fixture]
    fn comment() -> &'static str {
        return "comment";
    }

    #[fixture]
    fn age_of_onset() -> &'static str {
        return "P2Y";
    }

    #[fixture]
    fn age_at_last_encounter() -> &'static str {
        return "Adult onset";
    }

    #[fixture]
    fn deceased() ->  &'static str {
        return "no";
    }

      #[fixture]
    fn sex() -> &'static str {
        return "F";
    }
    

    #[rstest]
    fn test_valid_individual_bundle(pmid: &str,
        title: &str,
        individual_id: &str,
        comment: &str,
        age_of_onset: &str,
        age_at_last_encounter: &str,
        deceased: &str,
        sex: &str,) {
            let ib = IndividualBundle::new(pmid, title, individual_id, comment, age_of_onset, age_at_last_encounter, deceased, sex);
            let result = ib.do_qc();
            assert!(result.is_ok());
    }


    #[rstest]
    #[case("PMID29482508", "Invalid CURIE with no colon: 'PMID29482508'")]
    #[case("PMID: 29482508", "Contains stray whitespace: 'PMID: 29482508'")]
    #[case("", "Empty CURIE")]
    fn test_malformed_pmid(
        pmid: &str,
        title: &str,
        individual_id: &str,
        comment: &str,
        age_of_onset: &str,
        age_at_last_encounter: &str,
        deceased: &str,
        sex: &str,
        #[case] entry: &str,
        #[case] expected_error_msg: &str) 
    {
        let ib = IndividualBundle::new(entry, title, individual_id, comment, age_of_onset, age_at_last_encounter, deceased, sex);
        let result = ib.do_qc();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(expected_error_msg, err);
    
    }

     #[rstest]
     #[case("Difficult diagnosis and genetic analysis of fibrodysplasia  ossificans progressiva: a case report", 
        "Consecutive whitespace in 'Difficult diagnosis and genetic analysis of fibrodysplasia  ossificans progressiva: a case report'")]
    fn test_malformed_title(
        pmid: &str,
        title: &str,
        individual_id: &str,
        comment: &str,
        age_of_onset: &str,
        age_at_last_encounter: &str,
        deceased: &str,
        sex: &str,
        #[case] entry: &str,
        #[case] expected_error_msg: &str) 
    {
        let ib = IndividualBundle::new(pmid, entry, individual_id, comment, age_of_onset, age_at_last_encounter, deceased, sex);
        let result = ib.do_qc();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(expected_error_msg, err);
    
    }

    #[rstest]
    #[case("individual\\1", "Forbidden character '\\' found in label 'individual\\1'")]
    #[case("individual  A", "Consecutive whitespace in 'individual  A'")]
    fn test_malformed_individual(
        pmid: &str,
        title: &str,
        individual_id: &str,
        comment: &str,
        age_of_onset: &str,
        age_at_last_encounter: &str,
        deceased: &str,
        sex: &str,
        #[case] entry: &str,
        #[case] expected_error_msg: &str) 
    {
        let ib = IndividualBundle::new(pmid, title, entry, comment, age_of_onset, age_at_last_encounter, deceased, sex);
        let result = ib.do_qc();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(expected_error_msg, err);
    
    }

    #[rstest]
    #[case("P2", "Malformed age string 'P2'")]
    fn test_malformed_age_of_onset(
        pmid: &str,
        title: &str,
        individual_id: &str,
        comment: &str,
        age_of_onset: &str,
        age_at_last_encounter: &str,
        deceased: &str,
        sex: &str,
        #[case] entry: &str,
        #[case] expected_error_msg: &str) 
    {
        let ib = IndividualBundle::new(pmid, title, individual_id, comment, entry, age_at_last_encounter, deceased, sex);
        let result = ib.do_qc();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(expected_error_msg, err);
    }

    #[rstest]
    #[case("P22", "Malformed age string 'P22'")]
    #[case("Adultonset", "Malformed age string 'Adultonset'")]
    fn test_malformed_age_at_last_encounter(
        pmid: &str,
        title: &str,
        individual_id: &str,
        comment: &str,
        age_of_onset: &str,
        age_at_last_encounter: &str,
        deceased: &str,
        sex: &str,
        #[case] entry: &str,
        #[case] expected_error_msg: &str) 
    {
        let ib = IndividualBundle::new(pmid, title, individual_id, comment, age_of_onset, entry, deceased, sex);
        let result = ib.do_qc();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(expected_error_msg, err);
    }

    #[rstest]
    #[case("?", "Malformed deceased entry: '?'")]
    #[case("alive", "Malformed deceased entry: 'alive'")]
    fn test_malformed_deceased(
        pmid: &str,
        title: &str,
        individual_id: &str,
        comment: &str,
        age_of_onset: &str,
        age_at_last_encounter: &str,
        deceased: &str,
        sex: &str,
        #[case] entry: &str,
        #[case] expected_error_msg: &str) 
    {
        let ib = IndividualBundle::new(pmid, title, individual_id, comment, age_of_onset, age_at_last_encounter, entry, sex);
        let result = ib.do_qc();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(expected_error_msg, err);
    }

    #[rstest]
    #[case("male", "Malformed sex entry: 'male'")]
    #[case("f", "Malformed sex entry: 'f'")]
    fn test_malformed_sex(
        pmid: &str,
        title: &str,
        individual_id: &str,
        comment: &str,
        age_of_onset: &str,
        age_at_last_encounter: &str,
        deceased: &str,
        sex: &str,
        #[case] entry: &str,
        #[case] expected_error_msg: &str) 
    {
        let ib = IndividualBundle::new(pmid, title, individual_id, comment, age_of_onset, age_at_last_encounter, deceased, entry);
        let result = ib.do_qc();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(expected_error_msg, err);
    }
}
