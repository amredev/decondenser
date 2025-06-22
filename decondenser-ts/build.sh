#!/usr/bin/env bash

set -euo pipefail

. "$(dirname "${BASH_SOURCE[0]}")/../scripts/utils/lib.sh"

MODE=${MODE:-dev}

cd "$(dirname "${BASH_SOURCE[0]}")"

# We don't have too many dependencies. The build in release mode is as fast as in dev mode.
step cargo component build --release --target wasm32-unknown-unknown -p decondenser-wasm

options=()

if [[ "$MODE" == "prod" ]]; then
    options+=(
        --optimize
        --minify
    )
fi

step npx jco transpile \
    "${options[@]}" \
    --tla-compat \
    --base64-cutoff 100000 \
    --out-dir ./dist \
    ../target/wasm32-unknown-unknown/release/decondenser_wasm.wasm
