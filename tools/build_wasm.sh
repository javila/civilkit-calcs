#!/bin/sh
# Builds calcpad-wasm for the browser into web/calcpad-web/public/wasm.
# Requires the wasm32-unknown-unknown target, the wasm-bindgen CLI (same version as the
# wasm-bindgen crate) and, optionally, wasm-opt (binaryen).
set -e
cd "$(dirname "$0")/.."
mkdir -p target/tmp && export TMPDIR="$PWD/target/tmp"
OUT=web/calcpad-web/public/wasm
cargo build -p calcpad-wasm --target wasm32-unknown-unknown --profile wasm
wasm-bindgen target/wasm32-unknown-unknown/wasm/calcpad_wasm.wasm --target web --no-typescript --out-dir "$OUT"
if command -v wasm-opt >/dev/null 2>&1; then
  wasm-opt -Oz --enable-bulk-memory --enable-nontrapping-float-to-int --enable-sign-ext --enable-mutable-globals \
    "$OUT/calcpad_wasm_bg.wasm" -o "$OUT/calcpad_wasm_bg.wasm"
fi
raw=$(wc -c < "$OUT/calcpad_wasm_bg.wasm")
gz=$(gzip -9 -c "$OUT/calcpad_wasm_bg.wasm" | wc -c)
echo "calcpad_wasm_bg.wasm: $raw bytes raw, $gz bytes gzipped"
