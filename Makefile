SOAK_SECS ?= 30

.PHONY: fmt clippy test ui-test runtime-check fixture-regression soak verify release-preflight check

fmt:
	cargo fmt --all -- --check

clippy:
	cargo clippy --workspace --all-targets -- -D warnings

test:
	cargo test --workspace

ui-test:
	npm --prefix apps/desktop/ui run test

runtime-check:
	cargo check -p opscinema_desktop_backend --features runtime

fixture-regression:
	cargo test -p opscinema_desktop_backend phase11_fixture_ -- --nocapture

soak:
	OPSCINEMA_ASSUME_PERMISSIONS=1 OPSCINEMA_PROVIDER_MODE=stub OPSCINEMA_CAPTURE_BURST_FRAMES=0 OPSCINEMA_SOAK_SECS=$(SOAK_SECS) \
		cargo test -p opscinema_desktop_backend phase11_capture_soak_stream_consistency -- --ignored --nocapture

verify: fmt clippy test ui-test runtime-check fixture-regression

release-preflight: verify soak

check: verify
