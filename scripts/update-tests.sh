#!/usr/bin/env bash

set -euo pipefail

. "$(dirname "${BASH_SOURCE[0]}")/utils/lib.sh"

UPDATE_EXPECT=1 step cargo test -p decondenser --lib
