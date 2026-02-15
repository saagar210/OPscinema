# OpsCinema Suite (Name TBD) — Codex Contract Pack (Authoritative)

## Purpose
This `codex/` pack is the **execution contract** for implementing OpsCinema Suite as specified in the pasted plan. It includes:
- architecture decisions + rationale
- exact repo/module boundaries
- data models + schemas + IPC contracts
- phased implementation order with explicit dependencies and gates
- error handling strategy and subsystem edge cases
- integration and testing strategy (fixtures + CI)
- explicit assumptions (kept minimal)

This pack is designed so an engineer (or Codex) can implement without follow-up questions.

---

## REVIEW GATE (Do Not Skip)

### Goal
Ship a **local-first macOS desktop suite** that records sessions (screen keyframes + optional input + metadata + derived OCR), builds an **append-only Event Log** and derived **Evidence Graph**, produces **evidence-linked Steps**, supports **Tier-2 Anchoring**, and exports deterministic **TutorialPacks** and **ProofBundles/Runbooks** with strict policy gates.

### Success Metrics (Measurable)
- Deterministic export hashes: same fixture inputs produce identical `bundle_hash` in CI.
- Evidence gate: **zero** exported generated blocks without evidence refs.
- Tutorial strictness: **zero** TutorialPack exports with degraded anchors.
- Capture correctness: click markers align to frames across Retina + multi-monitor in curated QA scenario.
- CI: end-to-end pipeline on golden fixtures green on every merge to main.

### Constraints (Non-Negotiable)
- Offline by default; network allowlist only.
- Event-sourced edits (never overwrite the system of record).
- Typed IPC mandatory; IPC command list locked Phase 0.
- Step Model JSON schema locked Phase 5.
- macOS capture uses ScreenCaptureKit; OCR uses Apple Vision.
- VLM grounding only on keyframes, cached, drift-aware; strict provider schema validation.
- Agent Plant internal only; explicit invocation; no background autonomy.

### Must / Should / Could (Scope Control)
**Must**
- Event Log + Evidence Graph + Evidence Coverage gate
- Step model + event-sourced editing + schema lock + replay determinism
- Tier-2 anchoring + drift + reacquire + strict tutorial export block
- Deterministic exports (BLAKE3 + canonical JSON + versioned manifests + round-trip verify)
- Safe verifiers + proof/runbook exports
- Model Dock (Ollama + MLX) + export pinning

**Should**
- Multi-monitor capture, coordinate normalization tooling and tests
- Golden fixture library + CI pipeline
- Redaction workflows for exports

**Could (Explicitly deferred unless pulled in)**
- Terminal capture
- Filesystem diffs beyond minimal “frame diff” segmentation heuristic

### Stop/Go
**Stop** if any of these occur:
- UI can bypass backend storage/policy boundaries.
- TutorialPack export succeeds with degraded anchors and/or warnings.
- Any generated text can reach export without evidence refs.
- IPC types leak `any` into UI bindings.

**Go** when Phase gates pass (see `codex/07-implementation-order-phases.md`).

---

## Hard Rules (Enforced)
- UI never touches filesystem directly; all persistence and exports go through backend.
- Only `apps/desktop/src-tauri/src/api/**` is callable from UI (typed IPC).
- Only `apps/desktop/src-tauri/src/storage/**` reads/writes DB or asset store.
- Any generation/export must go through Evidence Coverage gate.
- No background autonomous Agent Plant behavior.

---

## Files in this Pack
- `codex/01-architecture-and-rationale.md`
- `codex/02-repo-structure-and-module-boundaries.md`
- `codex/03-contracts-ipc-errors-jobs.md`
- `codex/04-data-models-events-evidence-steps-anchors.md`
- `codex/05-storage-assets-determinism.md`
- `codex/06-export-manifest-and-bundles.md`
- `codex/07-implementation-order-phases.md`
- `codex/08-testing-fixtures-ci.md`
- `codex/09-security-privacy-verifiers.md`
- `codex/98-vp-stamp.md`
- `codex/99-one-shot-codex-prompt.md`
