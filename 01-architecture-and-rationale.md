# Architecture Decisions and Rationale (Locked)

## 1) Tech Stack (Locked)
**Decision:** Tauri v2 + Rust backend + React/TypeScript UI.  
**Rationale:** Local-first desktop app with strong system integration (macOS capture, permissions) while maintaining a web-grade UI. Rust backend centralizes persistence/policy/determinism.

## 2) Ownership Boundary (Locked)
**Decision:** Backend owns persistence + policy + capture + compute. UI is a view/editor only.  
**Rationale:** Enforces offline policy, deterministic exports, and event-sourced integrity. Prevents UI-side filesystem/network leaks.

## 3) System of Record (Locked)
**Decision:** Append-only Event Log as source of truth; derived Evidence Graph is computed.  
**Rationale:** Enables full replay determinism, auditability, and “never overwrite” constraints. Derived views can be rebuilt and validated.

## 4) Storage (Locked)
**Decision:** SQLite + content-addressed asset store (BLAKE3 hash-path).  
**Rationale:** SQLite provides durable local metadata and event sequencing; content-addressed assets guarantee integrity and support deterministic exports.

## 5) Deterministic Exports (Locked)
**Decision:** BLAKE3 hashing, canonical JSON, versioned manifests, round-trip verification.  
**Rationale:** Proof-grade exports require tamper evidence, repeatable hashing, and explicit compatibility.

## 6) Evidence-first Generation (Locked)
**Decision:** Generated text is structured (blocks) and MUST include evidence refs; export gate enforces.  
**Rationale:** Prevents ungrounded outputs from being treated as authoritative. Keeps derivations auditable.

## 7) Tutorial Strictness (Locked)
**Decision:** TutorialPack exports hard-block on degraded anchors; no “degraded anchors” allowed in exported packs.  
**Rationale:** Tutorials must be followable without ambiguity; any drift must be resolved before export.

## 8) ProofBundle Warnings (Locked)
**Decision:** ProofBundle exports may include warnings, but warnings must be explicit and recorded.  
**Rationale:** Proof exports may represent historical imperfect capture while preserving truth and audit trails.

## 9) Tier-2 Anchoring (Locked)
**Decision:** Hybrid deterministic anchoring + VLM only on keyframes; cached; drift-aware reacquire path.  
**Rationale:** Anchors must be stable and explainable. VLM is bounded and not used as continuous tracking.

## 10) Model Dock (Locked)
**Decision:** First-class adapters for Ollama + MLX. Model role routing and benchmark storage. Export pinning required.  
**Rationale:** Reproducibility and sane local model ops; model identity must be recorded with outputs.

## 11) Agent Plant (Locked)
**Decision:** Internal bounded DAG transforms invoked explicitly; never autonomous background activity.  
**Rationale:** Adds quality amplification while avoiding uncontrolled changes; preserves “evidence-first” posture.

## 12) Typed IPC Lock (Locked)
**Decision:** Typed IPC is mandatory; command list locked in Phase 0.  
**Rationale:** Prevents contract drift and untyped backdoors. Enables safe parallel development.

## 13) Step Schema Lock (Locked)
**Decision:** Step Model JSON schema locked in Phase 5; changes require versioned migration.  
**Rationale:** Steps are exported artifacts; schema stability is required for long-lived proof/tutorial packs.
