#!/usr/bin/env bash
set -euo pipefail

if [[ "$(uname)" == "Darwin" ]]; then
  echo "Building universal macOS binary (arm64 + x86_64)..."
  rustup target add aarch64-apple-darwin x86_64-apple-darwin

  cargo build --release --bin snemulator --target aarch64-apple-darwin
  cargo build --release --bin snemulator --target x86_64-apple-darwin

  mkdir -p target/release
  lipo -create -output target/release/snemulator \
    target/aarch64-apple-darwin/release/snemulator \
    target/x86_64-apple-darwin/release/snemulator

  echo "Universal binary created:"
  lipo -info target/release/snemulator
else
  cargo build --release --bin snemulator
fi