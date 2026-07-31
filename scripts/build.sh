#!/usr/bin/env bash
set -euo pipefail

# Resolve the repo root based on this script's own location, regardless
# of what directory it was invoked from (cargo-packager runs this from
# the crate directory, not the workspace root).
cd "$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# Pin the target dir explicitly. cargo-packager sets its own env vars
# when running this script (e.g. CARGO_PACKAGER_FORMATS), and may also
# set CARGO_TARGET_DIR, which would silently redirect cargo's output
# somewhere other than the plain ./target we expect below.
export CARGO_TARGET_DIR="$PWD/target"

if [[ "$(uname)" == "Darwin" ]]; then
  echo "Repo root: $PWD"
  echo "CARGO_TARGET_DIR: $CARGO_TARGET_DIR"

  if [[ -n "${MACOS_TARGET:-}" ]]; then
    # CI path: build only the single architecture this job was assigned.
    echo "Building single-architecture macOS binary for $MACOS_TARGET..."
    rustup target add "$MACOS_TARGET"
    cargo build --release --bin snemulator --target "$MACOS_TARGET"

    BIN="$CARGO_TARGET_DIR/$MACOS_TARGET/release/snemulator"
    if [[ ! -f "$BIN" ]]; then
      echo "ERROR: expected binary not found at $BIN"
      echo "Searching for it elsewhere under the repo..."
      find . -type f -name snemulator -not -path '*/incremental/*' 2>/dev/null || true
      exit 1
    fi

    mkdir -p "$CARGO_TARGET_DIR/release"
    cp "$BIN" "$CARGO_TARGET_DIR/release/snemulator"
  else
    # Local dev path: build a universal binary covering both architectures.
    echo "Building universal macOS binary (arm64 + x86_64)..."
    rustup target add aarch64-apple-darwin x86_64-apple-darwin

    cargo build --release --bin snemulator --target aarch64-apple-darwin
    cargo build --release --bin snemulator --target x86_64-apple-darwin

    AARCH64_BIN="$CARGO_TARGET_DIR/aarch64-apple-darwin/release/snemulator"
    X86_64_BIN="$CARGO_TARGET_DIR/x86_64-apple-darwin/release/snemulator"

    for f in "$AARCH64_BIN" "$X86_64_BIN"; do
      if [[ ! -f "$f" ]]; then
        echo "ERROR: expected binary not found at $f"
        echo "Searching for it elsewhere under the repo..."
        find . -type f -name snemulator -not -path '*/incremental/*' 2>/dev/null || true
        exit 1
      fi
    done

    mkdir -p "$CARGO_TARGET_DIR/release"
    lipo -create -output "$CARGO_TARGET_DIR/release/snemulator" "$AARCH64_BIN" "$X86_64_BIN"

    echo "Universal binary created:"
    lipo -info "$CARGO_TARGET_DIR/release/snemulator"
  fi
else
  cargo build --release --bin snemulator
fi