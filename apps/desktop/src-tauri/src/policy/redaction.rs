use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RedactionRule {
    pub pattern: String,
    pub replacement: String,
}

pub fn apply_redactions(input: &str, rules: &[RedactionRule]) -> String {
    let mut out = input.to_string();
    for rule in rules {
        out = out.replace(&rule.pattern, &rule.replacement);
    }
    out
}
