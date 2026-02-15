use serde::Serialize;
use serde_json::{Map, Number, Value};

pub fn to_canonical_json<T: Serialize>(value: &T) -> serde_json::Result<String> {
    let value = serde_json::to_value(value)?;
    let normalized = normalize(value);
    serde_json::to_string(&normalized)
}

fn normalize(value: Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut entries: Vec<(String, Value)> =
                map.into_iter().map(|(k, v)| (k, normalize(v))).collect();
            entries.sort_by(|a, b| a.0.cmp(&b.0));
            let mut sorted = Map::new();
            for (k, v) in entries {
                sorted.insert(k, v);
            }
            Value::Object(sorted)
        }
        Value::Array(arr) => Value::Array(arr.into_iter().map(normalize).collect()),
        Value::Number(n) => normalize_number(n),
        other => other,
    }
}

fn normalize_number(n: Number) -> Value {
    if let Some(i) = n.as_i64() {
        Value::Number(Number::from(i))
    } else if let Some(u) = n.as_u64() {
        Value::Number(Number::from(u))
    } else if let Some(f) = n.as_f64() {
        if let Some(v) = Number::from_f64(f) {
            Value::Number(v)
        } else {
            // Non-finite numbers are not valid JSON numbers; preserve with string fallback.
            Value::String(f.to_string())
        }
    } else {
        Value::Number(n)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn canonicalizes_key_order() {
        let v = json!({"b":1,"a":{"d":2,"c":1}});
        let got = to_canonical_json(&v).expect("canon");
        assert_eq!(got, "{\"a\":{\"c\":1,\"d\":2},\"b\":1}");
    }
}
