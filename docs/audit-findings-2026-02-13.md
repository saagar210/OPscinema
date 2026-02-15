# OpsCinema Audit Findings (2026-02-13)

## Scope
- Runtime lifecycle integrity (capture + jobs + event streams)
- Export/manifest verification fail-closed behavior
- Fixture hash governance and deterministic regression safety
- UI observability consistency for handoff flow

## Findings Summary
- `P0`: 0
- `P1`: 4 (all resolved)
- `P2`: 3 (all resolved in closeout follow-up)

## Resolved Findings

### P1-1 Capture lifecycle drift between loop state and authoritative status
- Risk: concurrent session start could slip past loop-only checks when capture status was still active.
- Fixes:
  - `/Users/d/Projects/OPscinema/apps/desktop/src-tauri/src/api/capture.rs`
    - conflict check now uses authoritative `capture_status` plus loop control state.
    - `capture_stop` clears loop slot and emits status via hook.
- Validation:
  - `phase2_capture_rejects_concurrent_session_start`
  - `phase2_capture_status_transitions_and_multi_keyframes`

### P1-2 Missing runtime emission path for background capture loop activity
- Risk: UI could appear stale while background capture continued.
- Fixes:
  - `/Users/d/Projects/OPscinema/apps/desktop/src-tauri/src/api/mod.rs`
    - added `capture_status_hook` to backend.
  - `/Users/d/Projects/OPscinema/apps/desktop/src-tauri/src/main.rs`
    - wired hook to runtime event bus (`capture_status`) for worker-thread emission.
  - `/Users/d/Projects/OPscinema/apps/desktop/src-tauri/src/api/capture.rs`
    - worker loop emits capture status snapshots during streaming capture.

### P1-3 Panic-prone assumptions in export path handling
- Risk: panic in production verify/build if path prefix assumptions fail.
- Fixes:
  - `/Users/d/Projects/OPscinema/apps/desktop/src-tauri/src/exports/manifest.rs`
    - replaced `strip_prefix(...).unwrap()` with typed error propagation.
  - `/Users/d/Projects/OPscinema/apps/desktop/src-tauri/src/exports/verify.rs`
    - replaced `strip_prefix(...).unwrap()` with typed error propagation.

### P1-4 Snapshot governance ambiguity for fixture hash updates
- Risk: accidental hash baseline drift without explicit acceptance intent.
- Fixes:
  - `/Users/d/Projects/OPscinema/docs/release-checklist.md`
    - documented explicit acceptance workflow using `OPSCINEMA_ACCEPT_FIXTURE_HASH=1`.

## Top P2 Resolved
- Improved job lifecycle status emission ordering based on persisted fields (`started_at` + terminal state).
  - `/Users/d/Projects/OPscinema/apps/desktop/src-tauri/src/api/tauri_commands.rs`

## P2 Closeout Follow-ups (Implemented)
1. Duplicate capture status emission mitigated by wrapper-side hook detection gate.
2. Long-duration capture soak test added (`phase11_capture_soak_stream_consistency`) and wired as optional CI workflow job.
3. UI capture stream telemetry added (`event_count`, `last_seq`, `gap_count`) for runtime observability.

## Residual Follow-ups (Non-blocking)
1. Keep optional soak run in release cadence (manual `make soak` or workflow_dispatch `run_soak=true`).
2. If expected fixture hashes are intentionally updated, require explicit acceptance flag and rerun verification without flag.

## Closeout Update (2026-02-15)
- Release hardening command path added:
  - `make release-hardening`
  - `make release-final`
- Packaging path validation added (`make package`) plus optional bundle artifact workflow.
- Additional verification coverage added for:
  - export verifier warning policy behavior (tutorial vs proof)
  - bundle hash mismatch detection
  - allowlist normalization and shell verifier destructive flag blocks
  - storage migration required-table bootstrap

Residual risks after closeout:
1. macOS code signing and notarization remain environment- and credential-dependent; release artifacts are buildable but notarization must be completed in a signing-capable pipeline.
2. Long-duration soak duration remains a release decision (`SOAK_SECS`) and should be increased for high-risk releases.

## Final Risk Closeout (2026-02-15)
- Addressed follow-up: long-duration soak run executed with `SOAK_SECS=60`.
- Addressed follow-up: app/dmg bundle generation validated using `make package-bundle`.
- Addressed follow-up: open Dependabot alert closed with rationale:
  - `glib` advisory GHSA-wrw7-89jp-8q8g is present only in Linux GTK/WebKit dependency graph (`cargo tree --target all`) and absent from macOS runtime graph.
  - Alert dismissed as `not_used` for this macOS-only release line.
- Remaining release dependency:
  - notarization/signing credentials and pipeline integration (operational prerequisite, not code defect).

## Verification Commands Executed
From:
- `/Users/d/Projects/OPscinema/.github/workflows/ci.yml`
- `/Users/d/Projects/OPscinema/README.md`
- `/Users/d/Projects/OPscinema/Makefile`

1. `cargo fmt --all -- --check`
2. `cargo clippy --workspace --all-targets -- -D warnings`
3. `cargo test --workspace`
4. `npm --prefix apps/desktop/ui run test`
5. `cargo check -p opscinema_desktop_backend --features runtime`
6. `cargo test -p opscinema_desktop_backend phase11_fixture_ -- --nocapture`
7. `make package`
8. `make bundle-verify-smoke`

All commands passed in this audit cycle.
