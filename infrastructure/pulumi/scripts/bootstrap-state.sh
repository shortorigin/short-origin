#!/usr/bin/env bash
set -euo pipefail

STACK="${1:-bootstrap}"
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PROJECT_DIR="${ROOT_DIR}/bootstrap-state"

cd "${PROJECT_DIR}"
npm install
pulumi stack select "${STACK}" --create
pulumi preview

echo "Run 'pulumi up' in ${PROJECT_DIR} after preview validation."
