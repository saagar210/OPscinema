# OpsCinema Suite

Local-first macOS desktop suite scaffold implementing the codex contract pack.

## Setup

1. Install Rust stable toolchain.
2. Install Node 20+.
3. Install UI dependencies:
   - `npm --prefix apps/desktop/ui ci`

## Verification Commands

- `make verify`
- `make soak` (set `SOAK_SECS=<N>` for longer soak)
- `make package` (validates Tauri build path; falls back to runtime compile if `cargo tauri` is unavailable)
- `make bundle-verify-smoke`
- `make clean-local` (removes local caches/artifacts: `target`, UI `node_modules`, `.DS_Store`)
- `make release-hardening` (verify + soak + package)
- `make release-final` (release-hardening + bundle/export smoke checks)

## Make Targets

- `make verify` runs the canonical verification ladder and auto-builds UI `dist` before runtime compile checks when missing.
- `make soak` runs the capture soak validation (`SOAK_SECS=30` default).
- `make package` validates the Tauri build path.
- `make package-bundle` builds `app,dmg` bundle artifacts when `cargo tauri` is available.
- `make clean-local` removes local build/dependency caches and Finder metadata files.
- `make release-hardening` runs `verify + soak + package`.
- `make release-preflight` aliases `release-hardening`.
- `make release-final` runs `release-hardening + bundle-verify-smoke`.

## Artifact Verification

Use the backend export verifier for generated bundles:

- `export_verify_bundle` IPC command via app flow, or
- `make bundle-verify-smoke` for targeted tutorial/proof/runbook regression checks.

For release details, see `/Users/d/Projects/OPscinema/docs/release-checklist.md`.
For full release + notarization operations, see `/Users/d/Projects/OPscinema/docs/RELEASE_PROCESS.md`.
