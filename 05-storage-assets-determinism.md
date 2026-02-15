# Storage, Asset Store, Canonical JSON, Deterministic Hashing (Phase 1)

## 1) SQLite Schema (Authoritative Minimum)
Tables required by plan:
- sessions
- events
- assets
- jobs
- ocr_blocks
- steps_snapshot (cache only)
- anchors_snapshot (cache only)
- verifiers
- verifier_runs
- exports
- models, model_roles, benchmarks

All DB writes happen in `apps/desktop/src-tauri/src/storage/**`.

## 2) Append-only Event Store API (Backend Internal)
`append_event(session_id, event_type, payload_struct) -> (event_id, seq, head_hash)`
- canonicalize payload to `payload_canon_json`
- compute `event_hash` chained from `prev_event_hash`
- insert into `events`
- update session head in `sessions`

`query_events(session_id, filters, paging) -> Vec<EventRow>`
- stable ordering by `seq` ascending

Crash safety:
- if event references assets, asset row must exist before event commit.

## 3) Asset Store (Content-addressed)
Asset ID:
- `asset_id = BLAKE3(bytes).hex_lower()`

Path:
- `{app_data}/opscinema/assets/{a0a1}/{a2a3}/{asset_id}`

Write protocol:
1) write to temp
2) fsync
3) atomic rename to final hash path
4) insert into assets table

## 4) Canonical JSON (Locked Phase 1)
Canonical JSON rules:
- UTF-8
- object keys sorted
- no insignificant whitespace
- numbers normalized; no NaN/Inf
- `\n` line endings
- timestamps ISO8601 UTC `Z`

All stored payloads are canonical JSON:
- events.payload_canon_json
- manifest.json
- artifacts JSON
- schema JSON

## 5) Deterministic Exports (Hashing)
- BLAKE3 for:
  - assets
  - manifest_hash
  - file hashes
  - bundle_hash

Bundle hash computation:
- enumerate all files in bundle, path-sorted
- compute file hash of bytes
- compute bundle_hash as BLAKE3 over concatenated tuples `(path + "\n" + file_hash + "\n")` in sorted order

Round-trip verification:
- validate manifest schema
- recompute hashes
- enforce policy gates recorded in manifest

## 6) Crash Simulation Test (Phase 1 Gate)
Test harness must simulate:
- crash after asset write but before event append
- crash after event insert but before transaction commit

Post-restart invariants:
- DB consistent
- no event references missing asset
- orphans may exist and are GC-able

## 7) GC for Orphan Assets (Explicit)
GC is user-invoked:
- scan references from:
  - events payloads (parse for AssetId fields)
  - exports manifest asset lists
  - verifier_runs logs
  - snapshots
- delete unreferenced assets (dry-run supported)
- append a `StorageGcRan` event (optional, for audit)
