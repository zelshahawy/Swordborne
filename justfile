set dotenv-load

build-web:
    cargo build --profile wasm-release --target wasm32-unknown-unknown
    wasm-bindgen --no-typescript --out-name bevy_game --out-dir wasm --target web \
        target/wasm32-unknown-unknown/wasm-release/swordborne.wasm
    cp -r assets wasm/

serve-web: build-web
    npx serve wasm

build:
    cargo build --release
