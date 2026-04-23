#!/usr/bin/env bash
set -euo pipefail

if ! command -v rustup >/dev/null 2>&1 || ! command -v cargo >/dev/null 2>&1; then
  echo "Rust toolchain not found. Installing rustup..."
  if command -v curl >/dev/null 2>&1; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
  elif command -v wget >/dev/null 2>&1; then
    wget -qO- https://sh.rustup.rs | sh -s -- -y --profile minimal
  else
    echo "Missing curl/wget to install rustup." >&2
    exit 1
  fi
fi

export PATH="$HOME/.cargo/bin:$PATH"

echo "Installing wasm target..."
rustup target add wasm32-unknown-unknown

if ! command -v wasm-bindgen >/dev/null 2>&1; then
  echo "Installing wasm-bindgen-cli..."
  cargo install wasm-bindgen-cli --version 0.2.118 --locked
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
