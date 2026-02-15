# OpsCinema Release Checklist

## Preflight
- Preferred one-command path:
  - `make release-hardening`
- Full release gate (includes export verification smoke):
  - `make release-final`
- Ensure `cargo fmt --all -- --check` passes.
- Ensure `cargo clippy --workspace --all-targets -- -D warnings` passes.
- Ensure `cargo test --workspace` passes.
- Ensure `make package` passes (tauri build path check).
- Confirm fixture hash regression test is stable.
- Confirm TutorialPack strict gate blocks degraded/warning/missing-evidence outputs.
- Soak validation:
  - `make soak`
  - `SOAK_SECS=60 make soak` (longer manual release cadence run)
  - CI `workflow_dispatch`: set `run_soak=true` and `soak_secs=<N>`

## Contract Locks
- IPC command set unchanged from Phase 0 lock.
- Step schema lock unchanged (`crates/opscinema_types/schema/step_model.v1.json`).
- Manifest version remains `1` unless compatibility logic is added.
- Storage migration protocol reviewed (`/Users/d/Projects/OPscinema/docs/storage-migrations.md`).

## Security and Policy
- Offline-by-default policy enabled in settings.
- Network allowlist changes reviewed.
- Verifier allowlist/timeout guardrails reviewed.
- CI deterministic provider mode enabled (`OPSCINEMA_PROVIDER_MODE=stub` for fixture runs).
- CI fixture stability mode enabled (`OPSCINEMA_DETERMINISTIC_IDS=1`, `OPSCINEMA_CAPTURE_BURST_FRAMES=1`).

## Artifact Validation
- `make bundle-verify-smoke` to run targeted tutorial/proof/runbook export-verify smoke tests.
- Run `export_verify_bundle` on tutorial/proof/runbook samples.
- Verify manifest hash and bundle hash recompute successfully.
- Verify generated text blocks include evidence refs.
- Open `player/index.html` from a tutorial export and confirm steps/evidence render for handoff readability.
- Refresh fixture hashes only with explicit acceptance intent:
  - `OPSCINEMA_ACCEPT_FIXTURE_HASH=1 cargo test -p opscinema_desktop_backend phase11_fixture_ -- --nocapture`
  - then rerun without the flag to confirm stability.

## Rollback Plan
- Revert to previous tagged release.
- Keep asset store and event DB read-only during rollback verification.
- Re-run fixture verification against rolled-back build.
- Run rollback drill on backup copy before production release:
  - `make verify`
  - `make bundle-verify-smoke`
  - `cargo test -p opscinema_desktop_backend phase11_fixture_ -- --nocapture`

## Definition of Done
- `make release-final` passes locally.
- Fixture hashes are unchanged (or updated only with explicit acceptance intent).
- Tutorial strict gate blocks degraded anchors, missing evidence, and warnings.
- Export verifier smoke checks pass for tutorial/proof/runbook paths.
- Storage migration protocol reviewed and rollback drill completed.

## Sign-off
Release candidate: `v0.2.1`

- Backend Platform Owner: `saagar210` (approved 2026-02-15)
- Capture/OCR Owner: `saagar210` (approved 2026-02-15)
- QA/Release Owner: `saagar210` (approved 2026-02-15)

## Release Evidence (2026-02-15)
- Long soak completed:
  - `SOAK_SECS=60 make soak`
- Full release gate completed:
  - `make release-final`
- macOS bundle artifacts generated:
  - `/Users/d/Projects/OPscinema/target/debug/bundle/macos/OpsCinema Suite.app`
  - `/Users/d/Projects/OPscinema/target/debug/bundle/dmg/OpsCinema Suite_0.1.0_aarch64.dmg`
- Open Dependabot alerts: `0`
