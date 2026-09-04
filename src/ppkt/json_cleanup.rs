use serde_json;

/// Removes fields from a serialized Phenopacket that serde populated with a
/// protobuf default value rather than a value you actually set, so the
/// output isn't cluttered with misleading zeros/placeholders.
///
/// Requires `serde_json`'s `preserve_order` feature; without it, object key
/// order isn't stable and this function's removals could appear to reorder
/// unrelated top-level fields.
///
/// Currently strips two fields, both nested under `subject`:
/// - `subject.karyotypicSex`, when it's `"UNKNOWN_KARYOTYPE"` or `0` — the
///   protobuf enum's zero value, output even though no karyotype was set.
/// - `subject.vitalStatus.survivalTimeInDays`, when it's `0` — this appears
///   whenever `vitalStatus` is set at all (e.g. just to record `DECEASED`),
///   even if no survival time was actually recorded, and a literal `0`
///   would incorrectly claim the subject died on day zero.
///
/// Any other field of `root`, including `subject` itself, is left untouched.
pub fn strip_phenopacket_defaults(root: &mut serde_json::Value) {
    if let serde_json::Value::Object(root_map) = root {
        if let Some(serde_json::Value::Object(subject)) = root_map.get_mut("subject") {
            let drop_karyotype = match subject.get("karyotypicSex") {
                Some(serde_json::Value::String(s)) if s == "UNKNOWN_KARYOTYPE" => true,
                Some(serde_json::Value::Number(n)) if n.as_i64() == Some(0) => true,
                _ => false,
            };
            if drop_karyotype {
                subject.remove("karyotypicSex");
            }

            if let Some(serde_json::Value::Object(vs)) = subject.get_mut("vitalStatus") {
                if let Some(serde_json::Value::Number(n)) = vs.get("survivalTimeInDays") {
                    if n.as_i64() == Some(0) {
                        vs.remove("survivalTimeInDays");
                    }
                }
            }
        }
    }
}