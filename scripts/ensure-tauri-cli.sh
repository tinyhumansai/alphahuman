#!/usr/bin/env bash
# Ensure the vendored CEF-aware tauri-cli is installed as `cargo-tauri`.
#
# The stock `@tauri-apps/cli` / upstream `tauri-cli` does NOT know how to bundle
# the CEF (Chromium Embedded Framework) runtime into the `.app` bundle's
# `Contents/Frameworks/` — so running `cargo tauri dev` with it produces an
# `OpenHuman.app` that panics at startup inside
# `cef::library_loader::LibraryLoader::new(...)` with:
#   "No such file or directory" (Os { code: 2 })
#
# The vendored fork at `app/src-tauri/vendor/tauri-cef/crates/tauri-cli` has the
# CEF bundler logic. Install it once and cargo will use it for every
# `cargo tauri ...` invocation.
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]:-$0}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
VENDOR_CLI="$ROOT_DIR/app/src-tauri/vendor/tauri-cef/crates/tauri-cli"
VENDOR_CARGO_TOML="$VENDOR_CLI/Cargo.toml"

if [[ ! -f "$VENDOR_CARGO_TOML" ]]; then
  echo "[ensure-tauri-cli] vendored tauri-cli not found at $VENDOR_CLI" >&2
  echo "[ensure-tauri-cli] did you forget to init the submodule? try:" >&2
  echo "    git submodule update --init --recursive" >&2
  exit 1
fi

# Detect whether the currently installed cargo-tauri came from our vendored path.
CRATES_TOML="${CARGO_HOME:-$HOME/.cargo}/.crates.toml"
if [[ -f "$CRATES_TOML" ]] && grep -q "tauri-cli.*$VENDOR_CLI" "$CRATES_TOML" 2>/dev/null; then
  # Already installed from this exact path. Cargo won't rebuild unless sources change.
  exit 0
fi

echo "[ensure-tauri-cli] installing vendored CEF-aware tauri-cli from $VENDOR_CLI"
echo "[ensure-tauri-cli] (first install only — takes a few minutes; subsequent runs are instant)"
cargo install --locked --path "$VENDOR_CLI"
