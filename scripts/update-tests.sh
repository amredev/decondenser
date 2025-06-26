#!/usr/bin/env bash

set -euo pipefail

. "$(dirname "${BASH_SOURCE[0]}")/utils/lib.sh"

step cargo test --all-features -p decondenser --test '*' -- snapshot_tests --nocapture
