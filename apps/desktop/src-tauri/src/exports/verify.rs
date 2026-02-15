use crate::exports::fs::list_files_sorted;
use crate::util::hash::blake3_hex;
use opscinema_export_manifest::{BundleType, ExportManifestV1};
use opscinema_types::ExportVerifyResponse;
use std::path::Path;

pub fn verify_bundle(root: &Path) -> anyhow::Result<ExportVerifyResponse> {
    let manifest_path = root.join("manifest.json");
    let manifest_raw = std::fs::read_to_string(&manifest_path)?;
    let manifest: ExportManifestV1 = serde_json::from_str(&manifest_raw)?;

    let mut issues = Vec::new();
    if manifest.manifest_version != 1 {
        issues.push(format!(
            "unsupported manifest_version {}",
            manifest.manifest_version
        ));
    }
    if !manifest.policy.evidence_coverage_passed {
        issues.push("policy attestation failed: evidence coverage".to_string());
    }
    if !manifest.policy.offline_policy_enforced {
        issues.push("policy attestation failed: offline policy".to_string());
    }
    if matches!(manifest.bundle_type, BundleType::TutorialPack) {
        if !manifest.warnings.is_empty() {
            issues.push("tutorial bundle contains warnings".to_string());
        }
        if !manifest.policy.tutorial_strict_passed {
            issues.push("policy attestation failed: tutorial strictness".to_string());
        }
    }

    for file in &manifest.files {
        let p = root.join(&file.path);
        if !p.exists() {
            issues.push(format!("missing file {}", file.path));
            continue;
        }
        let bytes = std::fs::read(&p)?;
        let hash = blake3_hex(&bytes);
        if hash != file.hash_blake3 {
            issues.push(format!("hash mismatch {}", file.path));
        }
    }

    let mut tuples = Vec::new();
    for file in &manifest.files {
        tuples.push((file.path.clone(), file.hash_blake3.clone()));
    }
    tuples.sort();
    let expected_bundle_hash = opscinema_export_manifest::compute_bundle_hash(&tuples);
    if expected_bundle_hash != manifest.bundle_hash {
        issues.push("bundle_hash mismatch".to_string());
    }

    // Ensure no undeclared files except manifest itself.
    let all_files = list_files_sorted(root)?;
    for p in all_files {
        let rel = p
            .strip_prefix(root)
            .map_err(|e| anyhow::anyhow!("bundle path escaping root: {e}"))?
            .to_string_lossy()
            .to_string();
        if rel == "manifest.json" {
            continue;
        }
        if !manifest.files.iter().any(|f| f.path == rel) {
            issues.push(format!("undeclared file {}", rel));
        }
    }

    Ok(ExportVerifyResponse {
        valid: issues.is_empty(),
        issues,
    })
}
