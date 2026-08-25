mod common;


use serde_json::Value;





pub fn strip_phenopacket_defaults(root: &mut Value) {
    // Top-level `subject`
    if let Value::Object(root_map) = root {
        if let Some(Value::Object(subject)) = root_map.get_mut("subject") {
            // Remove karyotypic_sex if it's the unknown/default
            let drop_karyotype = match subject.get("karyotypic_sex") {
                Some(Value::String(s)) if s == "UNKNOWN_KARYOTYPE" => true,
                Some(Value::Number(n)) if n.as_i64() == Some(0) => true,
                _ => false,
            };
            if drop_karyotype {
                subject.remove("karyotypic_sex");
            }

            // If you truly want to drop survival_time_in_days==0 from subject (enable if applicable)
            if let Some(Value::Number(n)) = subject.get("survival_time_in_days") {
                if n.as_i64() == Some(0) {
                    subject.remove("survival_time_in_days");
                }
            }

            // If your schema puts survival time inside a nested object (uncomment as needed)
            if let Some(Value::Object(vs)) = subject.get_mut("vital_status") {
                if let Some(Value::Number(n)) = vs.get("survival_time_in_days") {
                    if n.as_i64() == Some(0) {
                        vs.remove("survival_time_in_days");
                    }
                }
            }
        }
    }
}


