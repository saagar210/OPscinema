# Storage Migration and Downgrade Protocol

## Scope
This protocol governs schema changes under `/Users/d/Projects/OPscinema/apps/desktop/src-tauri/src/storage/schema`.

## Rules
- Migrations are append-only and ordered.
- Existing migration files are immutable after merge.
- Runtime must execute all pending migrations before serving IPC.
- Downgrades are not automatic; rollback uses app binary rollback with data compatibility checks.

## Safe Change Types
- Additive tables/columns with defaults.
- New indexes.
- New snapshot/cache tables that can be rebuilt from event log.

## Unsafe Change Types (Require Compatibility Plan)
- Dropping columns/tables.
- Changing semantic meaning of existing fields.
- Re-encoding canonical JSON payloads.

## Downgrade Strategy
1. Stop writes.
2. Export and verify latest bundles.
3. Roll back binary only if schema is backward-compatible.
4. If incompatible, restore DB backup taken before migration.
5. Re-run fixture verification before re-enabling writes.

## Pre-merge Checklist
- Add migration SQL file.
- Add migration test coverage for new schema objects.
- Confirm required base tables exist after migration bootstrap (`phase11_storage_migration_creates_required_tables`).
- Confirm event log replay still reconstructs derived state.
- Confirm export hash determinism is unchanged for existing fixtures.

## Rollback Drill (Release Hardening Cadence)
1. Run `make verify` and `make bundle-verify-smoke` before migration rollout.
2. Execute migration on a copy of a populated DB.
3. Run fixture regression check:
   - `cargo test -p opscinema_desktop_backend phase11_fixture_ -- --nocapture`
4. Simulate rollback with previous binary and confirm DB compatibility assumptions hold.
