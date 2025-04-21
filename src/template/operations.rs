

pub enum Operation {
    Clear,
    Edit,
    Trim,
    RemoveWhitespace,
    Yes,
    No,
    Na,
    Male,
    Female,
    Other,
    Unknown,
    Observed,
    Excluded,
}


impl Operation {
    pub fn as_str(&self) -> &str {
        match self {
            Operation::Clear =>  "clear",
            Operation::Edit =>  "edit",
            Operation::Trim => "trim",
            Operation::RemoveWhitespace => "remove whitespace",
            Operation::Yes => "yes",
            Operation::No => "no",
            Operation::Na => "na",
            Operation::Male => "M",
            Operation::Female => "F",
            Operation::Other => "O",
            Operation:: Unknown => "U",
            Operation::Observed => "observed",
            Operation::Excluded => "excluded",
        }
    }

    /// Try to create an Operation from a keyword
    pub fn from_keyword(s: &str) -> Option<Self> {
        match s {
            "clear" => Some(Operation::Clear),
            "trim" => Some(Operation::Trim),
            "remove whitespace" => Some(Operation::RemoveWhitespace),
            "yes" => Some(Operation::Yes),
            "no" => Some(Operation::No),
            "na" => Some(Operation::Na),
            "M" | "male" => Some(Operation::Male),
            "F" | "female" => Some(Operation::Female),
            "O" | "other" => Some(Operation::Other),
            "U" | "unknown" => Some(Operation::Unknown),
            "observed" => Some(Operation::Observed),
            "excluded" => Some(Operation::Excluded),
            _ => None,
        }
    }
    
}


