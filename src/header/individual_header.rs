use crate::{dto::cohort_dto::IndividualData, header::duplet_item::DupletItem, factory::individual_bundle::IndividualBundle};



#[derive(Clone, Debug)]
pub struct IndividualHeader {
    pub pmid: DupletItem,
    pub title: DupletItem,
    pub individual_id: DupletItem,
    pub comment: DupletItem,
    pub age_of_onset: DupletItem,
    pub age_at_last_encounter: DupletItem,
    pub deceased: DupletItem,
    pub sex: DupletItem
}


impl IndividualHeader {
    pub fn new() -> Self {
        Self { 
            pmid: DupletItem::pmid(),
            title: DupletItem::title(), 
            individual_id: DupletItem::individual_id(), 
            comment: DupletItem::comment(),
            age_of_onset: DupletItem::age_of_onset(), 
            age_at_last_encounter: DupletItem::age_at_last_encounter() ,
            deceased: DupletItem::deceased(),
            sex: DupletItem::sex()
        }
    }

     /// Check an individual bundle for errors.
    pub fn qc_bundle(&self, bundle: &IndividualBundle) -> Result<(), String> {
        self.qc_data(&bundle.pmid, &bundle.title, &bundle.individual_id, &bundle.comment, &bundle.age_of_onset, &bundle.age_at_last_encounter, &bundle.deceased, &bundle.sex)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn qc_data(&self, 
        pmid: &str, 
        title: &str, 
        individual_id: &str, 
        comment: &str,
        age_of_onset: &str, 
        age_at_last_encounter: &str, 
        deceased: &str, 
        sex: &str) 
    -> Result<(), String> {
        self.pmid.qc_data(pmid)?;
        self.title.qc_data(title)?;
        self.individual_id.qc_data(individual_id)?;
        self.comment.qc_data(comment)?;
        self.age_of_onset.qc_data(age_of_onset)?;
        self.age_at_last_encounter.qc_data(age_at_last_encounter)?;
        self.deceased.qc_data(deceased)?;
        self.sex.qc_data(sex)?;
        Ok(())
    }

}