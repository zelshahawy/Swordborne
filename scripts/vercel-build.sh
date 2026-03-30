#!/usr/bin/env bash
set -euo pipefail

echo "Installing wasm target..."
rustup target add wasm32-unknown-unknown

if ! command -v wasm-bindgen >/dev/null 2>&1; then
  echo "Installing wasm-bindgen-cli..."
  cargo install wasm-bindgen-cli --version 0.2.114 --locked
fi

echo "Building Rust WASM binary..."
cargo build --profile wasm-release --target wasm32-unknown-unknown

echo "Running wasm-bindgen..."
wasm-bindgen \
  --no-typescript \
  --out-name bevy_game \
  --out-dir wasm \
  --target web \
  target/wasm32-unknown-unknown/wasm-release/swordborne.wasm

echo "Copying assets..."
cp -r assets wasm/
