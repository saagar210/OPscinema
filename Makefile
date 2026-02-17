SOAK_SECS ?= 30
OPSCINEMA_REQUIRE_TAURI_PACKAGE ?= 0
OPSCINEMA_TAURI_BUNDLES ?= app,dmg

.PHONY: fmt clippy test ui-test ui-dist run lean-dev runtime-check fixture-regression soak verify package package-bundle bundle-verify-smoke clean-heavy clean-local clean-full-local release-hardening release-preflight release-final check

fmt:
	cargo fmt --all -- --check

clippy:
	cargo clippy --workspace --all-targets -- -D warnings

test:
	cargo test --workspace

ui-test:
	npm --prefix apps/desktop/ui run test

ui-dist:
	@if [ -d apps/desktop/ui/dist ]; then \
		echo "UI dist exists; skipping build"; \
	else \
		echo "UI dist missing; building"; \
		npm --prefix apps/desktop/ui run build; \
	fi

run:
	npm --prefix apps/desktop/ui run build
	cargo run -p opscinema_desktop_backend --features runtime --bin opscinema-desktop

lean-dev:
	./scripts/lean-dev.sh

runtime-check: ui-dist
	cargo check -p opscinema_desktop_backend --features runtime

fixture-regression:
	cargo test -p opscinema_desktop_backend phase11_fixture_ -- --nocapture

soak:
	OPSCINEMA_ASSUME_PERMISSIONS=1 OPSCINEMA_PROVIDER_MODE=stub OPSCINEMA_CAPTURE_BURST_FRAMES=0 OPSCINEMA_SOAK_SECS=$(SOAK_SECS) \
		cargo test -p opscinema_desktop_backend phase11_capture_soak_stream_consistency -- --ignored --nocapture

verify: fmt clippy test ui-test runtime-check fixture-regression

package:
	npm --prefix apps/desktop/ui run build
	@if cargo tauri --help >/dev/null 2>&1; then \
		echo "Running tauri build path validation (--no-bundle)"; \
		cd apps/desktop/src-tauri && cargo tauri build --debug --features runtime --no-bundle; \
	elif [ "$(OPSCINEMA_REQUIRE_TAURI_PACKAGE)" = "1" ]; then \
		echo "cargo tauri CLI unavailable; install tauri-cli or unset OPSCINEMA_REQUIRE_TAURI_PACKAGE"; \
		exit 1; \
	else \
		echo "cargo tauri CLI unavailable; using runtime compile fallback"; \
		cargo check -p opscinema_desktop_backend --features runtime; \
	fi

package-bundle:
	npm --prefix apps/desktop/ui run build
	@if cargo tauri --help >/dev/null 2>&1; then \
		cd apps/desktop/src-tauri && cargo tauri build --debug --features runtime --bundles $(OPSCINEMA_TAURI_BUNDLES); \
	else \
		echo "cargo tauri CLI unavailable for bundle build"; \
		exit 1; \
	fi

bundle-verify-smoke:
	cargo test -p opscinema_desktop_backend phase11_fixture_pipeline_export_verify_and_hash_regression -- --nocapture
	cargo test -p opscinema_desktop_backend phase8_runbook_is_replayed_and_export_is_listed -- --nocapture
	cargo test -p opscinema_desktop_backend phase8_proof_and_runbook_exports_include_verifier_warnings -- --nocapture

clean-heavy:
	rm -rf target
	rm -rf apps/desktop/src-tauri/target
	rm -rf apps/desktop/ui/dist

clean-local: clean-full-local

clean-full-local: clean-heavy
	rm -rf apps/desktop/ui/node_modules
	find . -path './.git' -prune -o -name '.DS_Store' -type f -delete

release-hardening: verify soak package

release-preflight: release-hardening

release-final: release-hardening bundle-verify-smoke

check: verify
