#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

TMP_BASE="${TMPDIR:-/tmp}"
LEAN_ROOT="$(mktemp -d "${TMP_BASE%/}/opscinema-lean.XXXXXX")"
LEAN_CARGO_TARGET_DIR="$LEAN_ROOT/cargo-target"
LEAN_XDG_CACHE_HOME="$LEAN_ROOT/xdg-cache"

cleanup() {
  local exit_code=$?

  # Keep dependencies for speed, but drop heavy reproducible outputs.
  rm -rf "$ROOT_DIR/apps/desktop/ui/dist"
  rm -rf "$ROOT_DIR/apps/desktop/src-tauri/target"
  rm -rf "$LEAN_ROOT"

  exit "$exit_code"
}

trap cleanup EXIT INT TERM

mkdir -p "$LEAN_CARGO_TARGET_DIR" "$LEAN_XDG_CACHE_HOME"

npm --prefix apps/desktop/ui run build

CARGO_TARGET_DIR="$LEAN_CARGO_TARGET_DIR" \
XDG_CACHE_HOME="$LEAN_XDG_CACHE_HOME" \
cargo run -p opscinema_desktop_backend --features runtime --bin opscinema-desktop
