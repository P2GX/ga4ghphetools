use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
#[serde(rename_all = "snake_case")]
pub enum AcmgPathogenicityClassification {
    #[serde(alias = "not provided", alias = "Not Provided")]
    NotProvided = 0,

    #[serde(alias = "Benign")]
    Benign = 1,

    #[serde(alias = "likely benign", alias = "Likely Benign")]
    LikelyBenign = 2,

    #[serde(alias = "uncertain significance", alias = "Uncertain Significance")]
    UncertainSignificance = 3,

    #[serde(alias = "likely pathogenic", alias = "Likely Pathogenic")]
    LikelyPathogenic = 4,

    #[serde(alias = "Pathogenic")]
    Pathogenic = 5,
}

impl AcmgPathogenicityClassification {
    pub fn from_str(acmg: &str) -> Self {
        match acmg.to_lowercase().as_str() {
            "benign" => Self::Benign,
            "likely benign" | "likely_benign" => Self::LikelyBenign,
            "uncertain significance" | "uncertain_significance" => Self::UncertainSignificance,
            "likely pathogenic" | "likely_pathogenic" => Self::LikelyPathogenic,
            "pathogenic" => Self::Pathogenic,
            _ => {
                eprintln!("Warning: Unrecognized ACMG category '{}'", acmg);
                Self::NotProvided
            }
        }
    }
}

impl fmt::Display for AcmgPathogenicityClassification {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::NotProvided => "not_provided",
            Self::Benign => "benign",
            Self::LikelyBenign => "likely_benign",
            Self::UncertainSignificance => "uncertain_significance",
            Self::LikelyPathogenic => "likely_pathogenic",
            Self::Pathogenic => "pathogenic",
        };
        write!(f, "{}", s)
    }
}


// region:    --- Tests

#[cfg(test)]
mod tests {
    type Error = Box<dyn std::error::Error>;
    type Result<T> = core::result::Result<T, Error>; // For tests.

    use super::*;

    #[test]
    fn test_name() -> Result<()> {
        let pathogenic = AcmgPathogenicityClassification::Pathogenic;
        assert_eq!("pathogenic", pathogenic.to_string());
    
        Ok(())
    }
}

// endregion: --- Tests