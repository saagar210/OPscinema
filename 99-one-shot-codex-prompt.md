# One-Shot Codex Prompt (Copy/Paste)

Read all files under `codex/` in this pack, then implement Phases 0–11 per `codex/07-implementation-order-phases.md`.

Absolute rules:
- Offline by default; allowlist only.
- Append-only event log; never overwrite.
- Evidence-first structured text with evidence refs; export gate enforces.
- TutorialPack strict: no degraded anchors, no warnings.
- Deterministic exports: BLAKE3 + canonical JSON + versioned manifest + verify.
- Typed IPC locked Phase 0; no new commands.
- Step schema locked Phase 5; migrations required for changes.

Begin Phase 0 now and proceed sequentially, passing each phase gate before continuing.
