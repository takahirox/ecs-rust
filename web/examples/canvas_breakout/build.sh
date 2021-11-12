cargo build --release --lib --target wasm32-unknown-unknown
wasm-bindgen ./target/wasm32-unknown-unknown/release/ecs_rust_web_example_breakout.wasm --out-dir ./ --target web --no-typescript
