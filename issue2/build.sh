#!/bin/bash
# Builds crate_b (which links crate_a) as a macOS dylib, generates the C# bindings for both crates
# with uniffi-bindgen-cs, drops them into the C# project unmodified and compiles the project. The
# compile is expected to fail with the error described in README.md until uniffi-bindgen-cs is fixed.
#
# Usage: ./build.sh [--no-dotnet]
#   BINDGEN_CONFIG=uniffi.toml ./build.sh    passes a bindgen config (see README.md)

set -uo pipefail
cd "$(dirname "$0")"

LIB=target/release/libcrate_b.dylib
OUT_DIR=csharp/Generated
NATIVE_DIR=csharp/native

echo "== cargo build --release =="
cargo build --release || exit 1
[ -f "$LIB" ] || { echo "error: $LIB not found (this script expects macOS; adjust the extension for other OSes)" >&2; exit 1; }

echo "== uniffi-bindgen-cs ($(uniffi-bindgen-cs --version)) =="
CONFIG_ARGS=()
if [ -n "${BINDGEN_CONFIG:-}" ]; then
    CONFIG_ARGS=(--config "$BINDGEN_CONFIG")
    echo "using config $BINDGEN_CONFIG"
fi
# Library mode reads the UNIFFI_META_* symbols of every crate in the dylib and writes one file per
# crate. It runs `cargo metadata`, so the CWD must be inside this workspace.
rm -rf "$OUT_DIR"
mkdir -p "$OUT_DIR" "$NATIVE_DIR"
uniffi-bindgen-cs --library "$LIB" --out-dir "$OUT_DIR" ${CONFIG_ARGS[@]+"${CONFIG_ARGS[@]}"} || exit 1
cp "$LIB" "$NATIVE_DIR/"
echo "generated: $(ls "$OUT_DIR" | tr '\n' ' ')"

[ "${1:-}" = "--no-dotnet" ] && exit 0

echo "== dotnet build (fails until uniffi-bindgen-cs is fixed, see README.md) =="
dotnet build csharp -nologo -v quiet
