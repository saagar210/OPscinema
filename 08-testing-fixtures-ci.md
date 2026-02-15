# Testing, Fixtures, CI (System-Level)

Golden fixture sessions are non-optional. Build an end-to-end pipeline test in CI:
fixture -> OCR -> EvidenceGraph -> Steps -> Anchors -> TutorialPack -> Verify.

Vision anchoring must be stubbed or use cached provider outputs in fixtures to preserve determinism.

Also enforce:
- IPC contract tests
- crash simulation tests
- export hash regression tests
