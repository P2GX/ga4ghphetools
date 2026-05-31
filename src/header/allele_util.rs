use std::{collections::HashSet, sync::LazyLock};





pub static ALLOWED_STRUCTURAL_PREFIX: LazyLock<HashSet<String>> = LazyLock::new(|| {
    HashSet::from([
        "DEL".to_string(),
        "DUP".to_string(),
        "INV".to_string(),
        "INS".to_string(),
        "TRANSL".to_string(),
    ])
});

/// We will be sending all HGVS variants to variant validator. Here, we just do 
/// a rough screening to reject some obvious mistakes.
/// - must start with c./n.
/// - must not contain whitespace
/// - must contain at least one digit
/// - If has '>', must have bases before and after
/// - If ins insertion, must have bases after 'ins'
pub fn is_plausible_hgvs(hgvs: &str) -> bool {
    if !(hgvs.starts_with("c.") || hgvs.starts_with("n.")) {
        return false;
    }
    if hgvs.contains(char::is_whitespace) {
        return false;
    }
    if !hgvs.chars().any(|c| c.is_ascii_digit()) {
        return false;
    }
    if let Some(pos) = hgvs.find('>') {
        // get the characters before and after '>'
        let (before, after) = (&hgvs[..pos], &hgvs[pos + 1..]);
        if !before.chars().rev().take_while(|c| c.is_ascii_alphabetic()).all(|c| "ACGT".contains(c)) {
            return false;
        }
        if !after.chars().take_while(|c| c.is_ascii_alphabetic()).all(|c| "ACGT".contains(c)) {
            return false;
        }
    }

    if let Some(pos) = hgvs.find("ins") {
        let after = &hgvs[pos + 3..]; // after 'ins'
        if after.is_empty() || !after.chars().all(|c| "ACGT".contains(c)) {
            return false;
        }
    }
    true
}







#[cfg(test)]
mod tests {

    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("c.6231dup", true)]
    #[case("c.6231_6233dup", true)]
    #[case("c.1932T>A", true)]
    #[case("c.417_418insA", true)]
    #[case("c.112_115delinsG", true)]
    #[case("c.76_78del", true)]  // you allow just 'del' in your logic
    #[case("c.76A>G", true)]
    #[case("c.1177del", true)]
    #[case("c.76_78ins", false)] // missing inserted sequence
    #[case("g.123456A>T", false)] // wrong prefix
    #[case("c.", false)]          // incomplete
    #[case("c.-19_*21del", true)]
    #[case("c.1630+1G>A", true)]
    fn test_check_valid_hgvs(#[case] input: &str, #[case] should_pass: bool) {
        let result = is_plausible_hgvs(input);
        assert_eq!(result, should_pass, "Failed on input: {}", input);
    }


   
    
}

// endregion: --- Testsq