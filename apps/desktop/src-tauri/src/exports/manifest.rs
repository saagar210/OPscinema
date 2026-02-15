use crate::exports::fs::list_files_sorted;
use crate::util::canon_json::to_canonical_json;
use crate::util::hash::blake3_hex;
use opscinema_export_manifest::{
    compute_bundle_hash, BundleType, ExportManifestV1, ManifestFileEntry, ManifestWarning,
    ModelPin, PolicyAttestations,
};
use std::path::Path;

pub fn build_manifest(
    root: &Path,
    bundle_type: BundleType,
    session_id: &str,
    warnings: Vec<ManifestWarning>,
    policy: PolicyAttestations,
    model_pins: Vec<ModelPin>,
) -> anyhow::Result<ExportManifestV1> {
    let files = list_files_sorted(root)?;
    let mut entries = Vec::new();
    for path in files {
        let rel = path
            .strip_prefix(root)
            .map_err(|e| anyhow::anyhow!("manifest path escaping root: {e}"))?
            .to_string_lossy()
            .to_string();
        if rel == "manifest.json" {
            continue;
        }
        let bytes = std::fs::read(&path)?;
        entries.push(ManifestFileEntry {
            path: rel,
            hash_blake3: blake3_hex(&bytes),
            size_bytes: bytes.len() as u64,
        });
    }

    entries.sort_by(|a, b| a.path.cmp(&b.path));
    let tuples = entries
        .iter()
        .map(|e| (e.path.clone(), e.hash_blake3.clone()))
        .collect::<Vec<_>>();
    let bundle_hash = compute_bundle_hash(&tuples);

    let mut manifest = ExportManifestV1 {
        manifest_version: 1,
        bundle_type,
        session_id: session_id.to_string(),
        created_at_utc: crate::util::time::now_utc_iso(),
        files: entries,
        warnings,
        policy,
        model_pins,
        manifest_hash: String::new(),
        bundle_hash,
    };

    let json = to_canonical_json(&manifest)?;
    manifest.manifest_hash = blake3_hex(json.as_bytes());
    Ok(manifest)
}
