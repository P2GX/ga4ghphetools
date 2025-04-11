#[derive(Debug, Clone)]
pub struct SimpleTerm {
    pub term_id: String,
    pub label: String,
}

impl SimpleTerm {
    pub fn new<S: Into<String>>(tid: S, lab: S) -> Self {
        SimpleTerm {
            term_id: tid.into(),
            label: lab.into(),
        }
    }

    pub fn term_id(&self) -> String {
        self.term_id.clone()
    }

    pub fn label(&self) -> String {
        self.label.clone()
    }
}
