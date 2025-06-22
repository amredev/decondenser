#!/usr/bin/env bash

set -euo pipefail

. "$(dirname "$(readlink -f "${BASH_SOURCE[0]}")")/../scripts/utils/lib.sh"

# Start the container even if installation fails
step npm install --workspaces --include-workspace-root || true

# Install the pre-commit hook. It's a symlink, to make sure it stays always up-to-date.
step ln -sf ../../.githooks/pre-commit .git/hooks/pre-commit
