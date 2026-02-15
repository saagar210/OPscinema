# Export Manifest Schema and Bundle Layout

Exports are deterministic bundles with:
- `manifest.json` (canonical JSON, versioned)
- assets and artifacts
- verification rules enforced by backend

Manifest types live in `crates/opscinema_export_manifest/**`.

## 1) Manifest Versioning
- `manifest_version: 1` for initial implementation
- breaking changes require version bump and compatibility rules in verifier

## 2) Manifest JSON (Authoritative v1)
(See the pack for the full JSON structure; implement as schema-validated canonical JSON.)

Rules:
- TutorialPack: warnings MUST be empty; strict anchor gate must pass.
- ProofBundle: warnings allowed but must be explicit and recorded.
- Runbook: warnings limited; never allow missing evidence refs for generated text.

## 3) Bundle Layouts
- TutorialPack includes tutorial.json + offline player + referenced assets.
- ProofBundle includes proof view + verifier logs + optional redaction report.
- Runbook includes runbook JSON and verifier specs.

## 4) Verification
`export_verify_bundle` validates schema, recomputes hashes, and enforces policy attestations.
