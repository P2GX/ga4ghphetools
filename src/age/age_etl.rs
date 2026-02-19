use std::collections::{HashMap, HashSet};
use once_cell::sync::Lazy;
use regex::Regex;


static ALLOWED_AGE_LABELS: Lazy<HashSet<String>> = Lazy::new(|| {
    [
        "Late onset",
        "Middle age onset",
        "Young adult onset",
        "Late young adult onset",
        "Intermediate young adult onset",
        "Early young adult onset",
        "Adult onset",
        "Juvenile onset",
        "Childhood onset",
        "Infantile onset",
        "Neonatal onset",
        "Congenital onset",
        "Antenatal onset",
        "Embryonal onset",
        "Fetal onset",
        "Late first trimester onset",
        "Second trimester onset",
        "Third trimester onset",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect()
});

// For the translation mapping
static AGE_TERM_MAP: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert("antenatal", "Antenatal onset");
    m.insert("neonate", "Neonatal onset");
    m.insert("neonatal", "Neonatal onset");
    m.insert("birth", "Congenital onset");
    m.insert("congenital", "Congenital onset");
    m.insert("childhood", "Childhood onset");
    m.insert("adult", "Adult onset");
    m.insert("unk", "na");
    m.insert("na", "na");
    m
});

static RE_YEAR: Lazy<Regex> = Lazy::new(|| {
    // Finds digits followed by y/yr/year
    Regex::new(r"(?i)(\d+(?:\.\d+)?)\s*y").unwrap()
});

static RE_MONTH: Lazy<Regex> = Lazy::new(|| {
    // Finds digits followed by m/mo/month
    Regex::new(r"(?i)(\d+(?:\.\d+)?)\s*m").unwrap()
});

static RE_WEEK: Lazy<Regex> = Lazy::new(|| {
    // Finds digits followed by w/wk/week
    Regex::new(r"(?i)(\d+(?:\.\d+)?)\s*w").unwrap()
});

static RE_DAY: Lazy<Regex> = Lazy::new(|| {
    // Finds digits followed by d/day
    Regex::new(r"(?i)(\d+)\s*d").unwrap()
});

pub fn map_age_string_to_symbolic(input: &str) -> Option<String> {
    let lower = input.to_lowercase();
    AGE_TERM_MAP.get(lower.as_str())
        .copied()
        .map(|s| s.to_string())
        .or_else(|| {
            ALLOWED_AGE_LABELS.contains(input)
                .then(|| input.to_string())
        })
}

pub fn map_ymd_to_iso(input: &str) -> Option<String> {
    let y_val = RE_YEAR.captures(input).and_then(|c| c[1].parse::<f64>().ok());
    let m_val = RE_MONTH.captures(input).and_then(|c| c[1].parse::<f64>().ok());
    let w_val = RE_WEEK.captures(input).and_then(|c| c[1].parse::<f64>().ok());
    let d_val = RE_DAY.captures(input).and_then(|c| c[1].parse::<f64>().ok());

    if y_val.is_none() && m_val.is_none() && w_val.is_none() && d_val.is_none() {
        return None;
    }

    // 3. Logic: Convert fractional years to months and weeks to days
    let raw_y = y_val.unwrap_or(0.0);
    let years = raw_y.floor() as i32;
    let months_from_y = ((raw_y - raw_y.floor()) * 12.0).round() as i32;
    
    let total_months = (m_val.unwrap_or(0.0).floor() as i32) + months_from_y;
    
    // Convert weeks to days: 1 week = 7 days
    let days_from_w = (w_val.unwrap_or(0.0) * 7.0).round() as i32;
    let total_days = (d_val.unwrap_or(0.0).floor() as i32) + days_from_w;
    let mut res = String::from("P");
    if years > 0 { res.push_str(&format!("{}Y", years)); }
    if total_months > 0 { res.push_str(&format!("{}M", total_months)); }
    if total_days > 0 { res.push_str(&format!("{}D", total_days)); }

    // If everything was 0 (e.g., "0 years"), return "P0D" or None
    if res == "P" { None } else { Some(res) }
}




#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;


    #[rstest]
    // The columns: input string, expected Option<String>
    #[case("neonate", Some("Neonatal onset".to_string()))]
    #[case("Unk", Some("na".to_string()))]
    #[case("Adult onset", Some("Adult onset".to_string()))]
    #[case("antenatal", Some("Antenatal onset".to_string()))]
    #[case("na", Some("na".to_string()))]
    #[case("random_junk", None)]
    #[case("", None)]
    fn test_map_age_string_to_symbolic(#[case] input: &str, #[case] expected: Option<String>) {
        assert_eq!(map_age_string_to_symbolic(input), expected);
    }

    #[rstest]
    #[case("1y9m", Some("P1Y9M"))]
    #[case("5y6m", Some("P5Y6M"))]
    #[case("5y6m3d", Some("P5Y6M3D"))]
    #[case("2 weeks", Some("P14D"))]
    #[case("1 week 3 days", Some("P10D"))]
    #[case("1.5 weeks", Some("P11D"))] // 10.5 rounded to 11 days
    #[case("1 month, 2 weeks", Some("P1M14D"))]
    #[case("G20w", Some("P140D"))] // Handles gestational style if it matches the week regex
    fn test_weeks_and_mixed(#[case] input: &str, #[case] expected: Option<&str>) {
        assert_eq!(map_ymd_to_iso(input).as_deref(), expected);
    }

     #[rstest]
    #[case("5y6m", Some("P5Y6M"))] // 10.5 rounded to 10 or 11 depending on rounding pref
 // Handles gestational style if it matches the week regex
    fn test_dd(#[case] input: &str, #[case] expected: Option<&str>) {
        assert_eq!(map_ymd_to_iso(input).as_deref(), expected);
    }

}