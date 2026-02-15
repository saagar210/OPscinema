# Exact File Structure and Module Boundaries (Authoritative)

## Root Boundaries (Locked)
- Backend: `apps/desktop/src-tauri/src/**`
- Frontend: `apps/desktop/ui/src/**`
- Shared types: `crates/opscinema_types/**`
- Export manifest schema: `crates/opscinema_export_manifest/**`
- Verifier SDK: `crates/opscinema_verifier_sdk/**`
- Typed IPC wiring: `crates/opscinema_ipc/**`

## Backend Layout (Mandatory)
Root: `apps/desktop/src-tauri/src/`

### IPC Surface (UI-callable only here)
- `api/mod.rs` (register commands)
- `api/app.rs` (build info, settings, permissions)
- `api/sessions.rs`
- `api/timeline.rs`
- `api/capture.rs`
- `api/ocr.rs`
- `api/evidence.rs`
- `api/steps.rs`
- `api/anchors.rs`
- `api/slicer.rs`
- `api/proof.rs`
- `api/model_dock.rs`
- `api/agent_plant.rs`
- `api/exports.rs`
- `api/jobs.rs`

Rule: no `tauri::command` outside `api/**`.

### Storage (ONLY place for DB + asset store IO)
- `storage/mod.rs` (facade)
- `storage/db.rs` (pool, migrations, tx helpers)
- `storage/schema/*.sql` (migration files)
- `storage/event_store.rs`
- `storage/asset_store.rs`
- `storage/index_fts.rs` (if used for OCR)
- `storage/repo_exports.rs`
- `storage/repo_jobs.rs`
- `storage/repo_models.rs`
- `storage/repo_verifiers.rs`

Rule: no direct SQLite access elsewhere.

### Policy (ALL enforcement centralized)
- `policy/mod.rs`
- `policy/network_allowlist.rs`
- `policy/export_gate.rs`
- `policy/permissions.rs`
- `policy/redaction.rs` (export redaction rules)

### Jobs
- `jobs/mod.rs`
- `jobs/runner.rs`
- `jobs/types.rs`
- `jobs/cancel.rs`

### Capture
- `capture/mod.rs`
- `capture/screen.rs` (ScreenCaptureKit bridge)
- `capture/input.rs` (click/keystroke capture)
- `capture/window_meta.rs`
- `capture/coord.rs` (single source of truth)

### OCR
- `ocr/mod.rs`
- `ocr/vision.rs` (Apple Vision bridge)
- `ocr/pipeline.rs`
- `ocr/index.rs`

### Evidence / Steps
- `evidence/mod.rs`
- `evidence/graph.rs`
- `evidence/query.rs`
- `evidence/coverage.rs`
- `steps/mod.rs`
- `steps/derive.rs`
- `steps/replay.rs` (rebuild from event log)
- `steps/edit_ops.rs`
- `steps/validate.rs`

### Anchors
- `anchors/mod.rs`
- `anchors/types.rs`
- `anchors/score.rs`
- `anchors/cache.rs`
- `anchors/drift.rs`
- `anchors/reacquire.rs`
- `anchors/providers/mod.rs`
- `anchors/providers/vision.rs`
- `anchors/debug.rs`

### Exports
- `exports/mod.rs`
- `exports/tutorial_pack.rs`
- `exports/proof_bundle.rs`
- `exports/runbook.rs`
- `exports/verify.rs`
- `exports/manifest.rs` (construct manifest from manifest crate)
- `exports/fs.rs` (safe filesystem operations; output writes)

### Verifiers
- `verifiers/mod.rs`
- `verifiers/runner.rs`
- `verifiers/builtins/shell.rs`
- `verifiers/builtins/file.rs`
- `verifiers/builtins/macos_settings.rs`
- `verifiers/builtins/network_allowlist.rs`

### Model Dock
- `model_dock/mod.rs`
- `model_dock/registry.rs`
- `model_dock/roles.rs`
- `model_dock/adapters/ollama.rs`
- `model_dock/adapters/mlx.rs`
- `model_dock/bench.rs`

### Agent Plant
- `agent_plant/mod.rs`
- `agent_plant/dag.rs`
- `agent_plant/transforms/*.rs`
- `agent_plant/diagnostics.rs`

### Utilities
- `util/canon_json.rs`
- `util/hash.rs`
- `util/time.rs` (UTC timestamps)
- `util/ids.rs` (uuid v5 helpers, deterministic IDs)
- `util/logging.rs`

### macOS Platform Bridge (Implementation Detail, Allowed)
- `platform/macos/mod.rs`
- `platform/macos/screencapturekit.*` (bridge wrapper)
- `platform/macos/vision_ocr.*` (bridge wrapper)

(Exact language/tooling is implementation detail; contract is that ScreenCaptureKit/Vision are used.)

## Frontend Layout (Mandatory)
Root: `apps/desktop/ui/src/`
- `ipc/` (generated typed IPC client)
- `app/` (router + navigation shell)
- `state/` (pure UI state; no persistence)
- `views/permissions/`
- `views/capture/` (timeline replay)
- `views/evidence/`
- `views/steps/`
- `views/anchors/`
- `views/slicer_studio/`
- `views/proof_ledger/`
- `views/model_dock/`
- `views/agent_plant/` (internal)
- `components/`

UI state management rule:
- UI caches via in-memory state only; authoritative state fetched from backend.
- Optimistic edits must use `base_seq` concurrency checks (see IPC docs).

## Shared Crates (Mandatory)
### `crates/opscinema_types`
- All cross-boundary types (requests/responses, models, enums, schema)
- Step Model JSON schema generation (locked Phase 5)

### `crates/opscinema_ipc`
- Command registry and request/response types (re-export from types)
- Codegen for TS bindings (deterministic)

### `crates/opscinema_export_manifest`
- Manifest types + JSON schema + compatibility rules

### `crates/opscinema_verifier_sdk`
- Interfaces only; no execution; safe-spec schemas

## Enforcement Checks (Required)
CI/static checks:
- forbid `tauri::command` outside `api/**`
- forbid `rusqlite`/`sqlx` usage outside `storage/**`
- forbid filesystem writes outside `storage/**` and `exports/**`
