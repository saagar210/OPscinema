# OpsCinema Suite

Local-first macOS desktop suite scaffold implementing the codex contract pack.

## Commands

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `npm --prefix apps/desktop/ui run test`
- `cargo check -p opscinema_desktop_backend --features runtime`
- `cargo test -p opscinema_desktop_backend phase11_fixture_ -- --nocapture`

## Make Targets

- `make verify` runs the canonical verification ladder.
- `make soak` runs the optional capture soak validation (`SOAK_SECS=30` default).
- `make release-preflight` runs canonical checks plus soak.
