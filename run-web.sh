#!/bin/bash

cargo build --release --target wasm32-unknown-unknown

wasm-bindgen --out-dir ./out/ --target web ./target/wasm32-unknown-unknown/release/explosive.wasm

npx serve .
