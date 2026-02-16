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
- `make clean-heavy` (removes heavy build artifacts only: `target`, `apps/desktop/src-tauri/target`, UI `dist`)
- `make clean-full-local` (removes reproducible local caches/artifacts: heavy artifacts + UI `node_modules` + `.DS_Store`)
- `make release-hardening` (verify + soak + package)
- `make release-final` (release-hardening + bundle/export smoke checks)

## Dev Modes

### Normal dev

- `make run`
- Behavior: builds UI `dist`, then starts desktop runtime.
- Tradeoff: fastest repeat startup after first compile, but Rust build artifacts persist in `target` and consume significant disk.

### Lean dev (low disk mode)

- `make lean-dev`
- Behavior: builds UI and starts desktop runtime normally, but uses temporary cache dirs (`CARGO_TARGET_DIR`, `XDG_CACHE_HOME`) and auto-cleans heavy artifacts when the app exits.
- Tradeoff: much lower persistent disk usage, but slower startup because Rust recompiles into an ephemeral target directory on each run.

## Make Targets

- `make run` builds UI and starts the desktop runtime (`cargo run` with runtime feature).
- `make lean-dev` starts a low-disk run using `scripts/lean-dev.sh`.
- `make verify` runs the canonical verification ladder and auto-builds UI `dist` before runtime compile checks when missing.
- `make soak` runs the capture soak validation (`SOAK_SECS=30` default).
- `make package` validates the Tauri build path.
- `make package-bundle` builds `app,dmg` bundle artifacts when `cargo tauri` is available.
- `make clean-heavy` removes heavy build artifacts only.
- `make clean-full-local` removes all reproducible local caches/artifacts.
- `make clean-local` aliases `make clean-full-local`.
- `make release-hardening` runs `verify + soak + package`.
- `make release-preflight` aliases `release-hardening`.
- `make release-final` runs `release-hardening + bundle-verify-smoke`.

## Artifact Verification

Use the backend export verifier for generated bundles:

- `export_verify_bundle` IPC command via app flow, or
- `make bundle-verify-smoke` for targeted tutorial/proof/runbook regression checks.

For release details, see `/Users/d/Projects/OPscinema/docs/release-checklist.md`.
For full release + notarization operations, see `/Users/d/Projects/OPscinema/docs/RELEASE_PROCESS.md`.
