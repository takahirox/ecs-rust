RUSTFLAGS=--cfg=web_sys_unstable_apis cargo build --no-default-features --release --lib --target wasm32-unknown-unknown 
wasm-bindgen ./target/wasm32-unknown-unknown/release/ecs_rust_web_example_webgpu_boxes.wasm --out-dir ./ --target web --no-typescript
