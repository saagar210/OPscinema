# Security, Privacy, Allowlist Networking, Verifiers

Offline by default; all outbound traffic blocked unless allowlisted.
All network attempts must route through backend allowlist wrapper.

Verifier guardrails:
- no destructive writes
- bounded timeouts
- deny-by-default command allowlist
- attach results as evidence

Export policy gates:
- missing evidence refs -> hard block
- TutorialPack degraded anchors or warnings -> hard block
