#!/bin/sh
# Builds the wasm modules for the browser into web/calcpad-web/public/wasm:
#   calcpad-wasm              the calculation engine
#   calcpad-docx-wasm         the Word export (loaded on demand)
#   calcpad-highlighter-wasm  the editor's highlighting, linting and snippets (loaded on first use)
#   calcpad-import-xmcd-wasm  the worksheet importer (loaded when a worksheet is imported)
# Requires the wasm32-unknown-unknown target, the wasm-bindgen CLI (same version as the
# wasm-bindgen crate) and, optionally, wasm-opt (binaryen).
# Usage: tools/build_wasm.sh [engine|docx|highlighter|import]   (default: all)
set -e
cd "$(dirname "$0")/.."
mkdir -p target/tmp && export TMPDIR="$PWD/target/tmp"
OUT=web/calcpad-web/public/wasm
# Panic messages embed source paths; keep the builder's home folder out of the shipped modules.
export RUSTFLAGS="${RUSTFLAGS:-} --remap-path-prefix=$HOME=/build"

for m in calcpad_wasm calcpad_docx_wasm calcpad_highlighter_wasm calcpad_import_xmcd_wasm; do
  case $m in
    calcpad_wasm) crate=calcpad-wasm; profile=wasm ;;
    calcpad_docx_wasm) crate=calcpad-docx-wasm; profile=wasm-size ;;
    calcpad_highlighter_wasm) crate=calcpad-highlighter-wasm; profile=wasm-size ;;
    calcpad_import_xmcd_wasm) crate=calcpad-import-xmcd-wasm; profile=wasm-size ;;
  esac
  case "${1:-all}" in
    all) ;;
    engine) [ $m = calcpad_wasm ] || continue ;;
    docx) [ $m = calcpad_docx_wasm ] || continue ;;
    highlighter) [ $m = calcpad_highlighter_wasm ] || continue ;;
    import) [ $m = calcpad_import_xmcd_wasm ] || continue ;;
  esac
  cargo build -p $crate --target wasm32-unknown-unknown --profile $profile
  wasm-bindgen target/wasm32-unknown-unknown/$profile/$m.wasm --target web --no-typescript --out-dir "$OUT"
  if command -v wasm-opt >/dev/null 2>&1; then
    wasm-opt -Oz --enable-bulk-memory --enable-nontrapping-float-to-int --enable-sign-ext --enable-mutable-globals \
      "$OUT/${m}_bg.wasm" -o "$OUT/${m}_bg.wasm"
  fi
  raw=$(wc -c < "$OUT/${m}_bg.wasm")
  gz=$(gzip -9 -c "$OUT/${m}_bg.wasm" | wc -c)
  echo "${m}_bg.wasm: $raw bytes raw, $gz bytes gzipped"
done
