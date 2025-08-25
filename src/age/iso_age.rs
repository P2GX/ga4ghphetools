use once_cell::sync::Lazy;
use regex::Regex;




static ISO8601_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^P(?:(\d+)Y)?(?:(\d+)M)?(?:(\d+)D)?$").unwrap()
});

pub struct Iso8601Age{}

impl Iso8601Age {
      pub fn is_valid(cell_value: &str) -> bool {
        ISO8601_RE.is_match(cell_value)
    }

}