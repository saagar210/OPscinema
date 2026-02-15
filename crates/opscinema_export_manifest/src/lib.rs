use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BundleType {
    TutorialPack,
    ProofBundle,
    Runbook,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ManifestWarning {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ManifestFileEntry {
    pub path: String,
    pub hash_blake3: String,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct PolicyAttestations {
    pub evidence_coverage_passed: bool,
    pub tutorial_strict_passed: bool,
    pub offline_policy_enforced: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ModelPin {
    pub role: String,
    pub model_id: String,
    pub digest: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ExportManifestV1 {
    pub manifest_version: u32,
    pub bundle_type: BundleType,
    pub session_id: String,
    pub created_at_utc: String,
    pub files: Vec<ManifestFileEntry>,
    pub warnings: Vec<ManifestWarning>,
    pub policy: PolicyAttestations,
    pub model_pins: Vec<ModelPin>,
    pub manifest_hash: String,
    pub bundle_hash: String,
}

pub fn compute_bundle_hash(entries_sorted: &[(String, String)]) -> String {
    let mut s = String::new();
    for (path, hash) in entries_sorted {
        s.push_str(path);
        s.push('\n');
        s.push_str(hash);
        s.push('\n');
    }
    blake3::hash(s.as_bytes()).to_hex().to_string()
}
