#!/usr/bin/env bash
set -euo pipefail

STACK="${1:?stack required: dev, stage, or prod}"
if [[ "${STACK}" != "dev" && "${STACK}" != "stage" && "${STACK}" != "prod" ]]; then
  echo "invalid stack: ${STACK}. expected dev, stage, or prod"
  exit 1
fi

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PROJECT_DIR="${ROOT_DIR}/live"

cd "${PROJECT_DIR}"
npm install
pulumi stack select "${STACK}" --create
pulumi preview
