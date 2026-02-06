#!/bin/sh

set -ex

wasm-pack build --target web --release
rm pkg/.gitignore

# time wasm-opt -O3 pkg/verse_bg.wasm -o pkg/verse_bg.wasm
