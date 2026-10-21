#!/bin/sh
# Builds calcpad-wasm (the engine) and calcpad-docx-wasm (the Word export, loaded on demand)
# for the browser into web/calcpad-web/public/wasm.
# Requires the wasm32-unknown-unknown target, the wasm-bindgen CLI (same version as the
# wasm-bindgen crate) and, optionally, wasm-opt (binaryen).
set -e
cd "$(dirname "$0")/.."
mkdir -p target/tmp && export TMPDIR="$PWD/target/tmp"
OUT=web/calcpad-web/public/wasm
cargo build -p calcpad-wasm -p calcpad-docx-wasm --target wasm32-unknown-unknown --profile wasm
for m in calcpad_wasm calcpad_docx_wasm; do
  wasm-bindgen target/wasm32-unknown-unknown/wasm/$m.wasm --target web --no-typescript --out-dir "$OUT"
  if command -v wasm-opt >/dev/null 2>&1; then
    wasm-opt -Oz --enable-bulk-memory --enable-nontrapping-float-to-int --enable-sign-ext --enable-mutable-globals \
      "$OUT/${m}_bg.wasm" -o "$OUT/${m}_bg.wasm"
  fi
  raw=$(wc -c < "$OUT/${m}_bg.wasm")
  gz=$(gzip -9 -c "$OUT/${m}_bg.wasm" | wc -c)
  echo "${m}_bg.wasm: $raw bytes raw, $gz bytes gzipped"
done
