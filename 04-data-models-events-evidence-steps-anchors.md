# Data Models, Event Payloads, Deterministic IDs (Critical Fixes)

This file resolves two common failure modes that were implicit in the original phase outline:
1) **Stable identifiers** for steps/evidence/anchors across replay.
2) **Event payload completeness** so replay reconstructs exact state without re-derivation ambiguity.

## 1) ID Strategy (Minimal Assumption, Required for Replay)
### Event IDs
- `event_id` is generated once at append time (UUID v4).
- `seq` is monotonic per session and assigned by DB at append.

### Evidence IDs (Deterministic)
Evidence must remain stable across rebuilds. Evidence IDs are derived deterministically:

`evidence_id = uuid_v5(EVIDENCE_NAMESPACE_UUID, "{session_id}:{kind}:{source_id}")`

Where:
- `kind` is stable string (e.g., `FrameKeyframe`, `Click`, `OcrSpan`, `VerifierResult`, `AnchorObservation`)
- `source_id` is stable:
  - frame: `event_id` of `KeyframeCaptured`
  - click: `event_id` of `ClickCaptured`
  - ocr span: `ocr_block_id`
  - verifier result: `run_id`
  - anchor observation: `anchor_id` or `anchor_observation_asset_id`

This guarantees that evidence refs stored in structured text remain resolvable after restart/replay.

### Step IDs (Stability Requirement)
Steps must not be regenerated differently on replay. Therefore:
- The **initial step set** must be persisted as an event payload, including step IDs.
- Edits reference those IDs.

Step IDs are generated once when step candidates are created:
- `step_id` = UUID v4 assigned during candidate generation job at commit time.

### Anchor IDs (Stability Requirement)
Anchor candidates must be persisted with IDs so drift/reacquire history is auditable.
- `anchor_id` = UUID v4 assigned during anchor candidate generation.

## 2) Event Payloads (Authoritative)
All event payload JSON is canonicalized per `codex/05-storage-assets-determinism.md`.

### Capture Events
**KeyframeCaptured**
- `frame_ms: i64`
- `asset_id: AssetId`
- `display_id: string`
- `pixel_w: u32`, `pixel_h: u32`
- `scale_factor: f32`
- `cursor_pos_norm?: {x: f32, y: f32}`

**ClickCaptured**
- `frame_ms`
- `button`
- `pos_norm: {x,y}`
- `display_id`
- `window_ref?: {bundle_id?, title?, bounds_norm?}`

**WindowMetaCaptured**
- `frame_ms`
- `frontmost_bundle_id?`
- `frontmost_title?`
- `bounds_norm?: {x,y,w,h}`

### OCR Events
**OcrBlocksPersisted**
- `frame_event_id: EventId`
- `frame_ms`
- `ocr_asset_id: AssetId` (canonical JSON of blocks for that frame)
- `blocks: [{ocr_block_id, bbox_norm, text, confidence, language?}]`

### Steps Events (Critical)
**StepsCandidatesGenerated**
- `schema_version: u32` (step schema version)
- `steps: Step[]` (full initial set including step_ids, order_index, structured text, evidence refs, branches, risk tags)
This event is the authoritative “initial step set.” Replay starts from this payload, then applies edits.

**StepEditApplied**
- `base_seq: i64` (seq at time UI initiated edit)
- `op: StepEditOp` (one of the edit operations)
- `applied_at: utc timestamp`

### Anchors Events
**AnchorCandidatesGenerated**
- `step_id`
- `anchors: AnchorCandidate[]` (includes anchor_id, kind, target_signature, initial locators, confidence, provenance)
Candidates exist even before resolution.

**AnchorResolved**
- `anchor_id`
- `resolved_locators: EvidenceLocator[]`
- `confidence`
- `provenance`
- `supporting_evidence_ids: EvidenceId[]`
- `provider_output_asset_id?: AssetId` (for VisionAnchor)

**AnchorDegraded**
- `anchor_id`
- `reason_code`
- `details`
- `last_verified_locators`
- `degraded_at`

**AnchorManuallySet**
- `anchor_id`
- `locators`
- `manual_note?`

### Exports Events
**ExportCreated**
- `export_id`
- `bundle_type`
- `output_path` (user-selected)
- `manifest_asset_id`
- `bundle_hash`
- `warnings: Warning[]` (possibly empty; TutorialPack should be empty for strictness)
- `created_at`

**ExportWarningRecorded**
- `export_id`
- `warning: Warning`

### Verifier Events
**VerifierRunCompleted**
- `run_id`
- `verifier_id`
- `status`
- `result_asset_id`
- `logs_asset_id?`
- `evidence_ids: EvidenceId[]`

## 3) Structured Text (Evidence-first, Enforced)
All generated text is represented as `StructuredText { blocks: TextBlock[] }`.

Hard rule for export:
- If `provenance == generated`, `evidence_refs` MUST be non-empty.

## 4) Evidence Locator Schema (Explicit)
`EvidenceLocator`:
- `locator_type: "timeline" | "frame_bbox" | "ocr_bbox" | "anchor_bbox" | "verifier_log" | "file_path"`
- `asset_id?: AssetId`
- `frame_ms?: i64`
- `bbox_norm?: {x,y,w,h}` (0..1)
- `text_offset?: {start: u32, end: u32}`
- `note?: string`

## 5) Step Schema Lock Process (Phase 5 Gate)
At Phase 5 acceptance:
- Generate JSON schema from Rust type definitions.
- Commit it under:
  - `crates/opscinema_types/schema/step_model.v1.json`
- Add a test asserting generated schema matches committed schema byte-for-byte.
- Validate all exported steps against the committed schema.
