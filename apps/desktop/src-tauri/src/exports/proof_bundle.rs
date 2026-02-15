use crate::exports::fs::write_file;
use crate::exports::manifest::build_manifest;
use crate::policy::export_gate::{proof_bundle_gate, ExportGateInput};
use crate::util::canon_json::to_canonical_json;
use opscinema_export_manifest::{BundleType, ManifestWarning, ModelPin, PolicyAttestations};
use opscinema_types::{ExportResult, ExportWarning, Step};
use std::path::Path;
use uuid::Uuid;

pub fn export_proof_bundle(
    session_id: Uuid,
    steps: &[Step],
    warnings: &[ExportWarning],
    missing_evidence: Vec<String>,
    model_pins: Vec<ModelPin>,
    offline_policy_enforced: bool,
    output_dir: &Path,
) -> anyhow::Result<ExportResult> {
    proof_bundle_gate(&ExportGateInput {
        steps: steps.to_vec(),
        missing_evidence: missing_evidence.clone(),
        degraded_anchor_ids: vec![],
        warnings: warnings.to_vec(),
    })?;

    write_file(
        &output_dir.join("proof.json"),
        to_canonical_json(&serde_json::json!({"steps":steps,"warnings":warnings}))?.as_bytes(),
    )?;

    let manifest = build_manifest(
        output_dir,
        BundleType::ProofBundle,
        &session_id.to_string(),
        warnings
            .iter()
            .map(|w| ManifestWarning {
                code: w.code.clone(),
                message: w.message.clone(),
            })
            .collect(),
        PolicyAttestations {
            evidence_coverage_passed: missing_evidence.is_empty(),
            tutorial_strict_passed: true,
            offline_policy_enforced,
        },
        model_pins,
    )?;
    write_file(
        &output_dir.join("manifest.json"),
        to_canonical_json(&manifest)?.as_bytes(),
    )?;

    Ok(ExportResult {
        export_id: Uuid::new_v4(),
        output_path: output_dir.display().to_string(),
        bundle_hash: manifest.bundle_hash,
        warnings: warnings.to_vec(),
    })
}
