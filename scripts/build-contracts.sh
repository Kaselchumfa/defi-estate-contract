#!/usr/bin/env bash
# Build every Soroban contract to optimized WASM.
set -euo pipefail
cd "$(dirname "$0")/.."

if command -v stellar >/dev/null 2>&1; then
  stellar contract build
else
  rustup target add wasm32v1-none >/dev/null
  cargo build --target wasm32v1-none --release
fi
ls -la target/wasm32v1-none/release/*.wasm
