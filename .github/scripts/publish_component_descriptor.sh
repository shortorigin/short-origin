#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 3 ]]; then
  echo "usage: publish_component_descriptor.sh <artifact-path> <repository> <comma-separated-tags>" >&2
  exit 1
fi

artifact_path="$1"
repository="$2"
tags="$3"

output="$(
  oras push \
    --artifact-type application/vnd.shortorigin.component.v1+json \
    --annotation "org.opencontainers.image.source=${GITHUB_SERVER_URL:-https://github.com}/${GITHUB_REPOSITORY:-shortorigin/short-origin}" \
    --annotation "org.opencontainers.image.revision=${GITHUB_SHA:-unknown}" \
    --format json \
    "${repository}:${tags}" \
    "${artifact_path}:application/vnd.shortorigin.component.v1+json"
)"

printf '%s' "${output}" | jq -r '.digest'
