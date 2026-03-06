#!/usr/bin/env bash
set -euo pipefail

output_path="${GITHUB_OUTPUT:?GITHUB_OUTPUT is required}"
event_name="${GITHUB_EVENT_NAME:?GITHUB_EVENT_NAME is required}"
event_path="${GITHUB_EVENT_PATH:?GITHUB_EVENT_PATH is required}"

mapfile -t all_nomad_jobs < <(find infrastructure/nomad/jobs -type f -name '*.nomad.hcl' | sort)

if [[ "${event_name}" == "merge_group" ]]; then
  rust_changed=true
  nomad_changed=true
  pulumi_changed=true
  changed_files="$(git ls-files)"
  nomad_files=("${all_nomad_jobs[@]}")
else
  case "${event_name}" in
    pull_request)
      base_sha="$(jq -r '.pull_request.base.sha' "${event_path}")"
      head_sha="$(jq -r '.pull_request.head.sha' "${event_path}")"
      ;;
    push)
      base_sha="$(jq -r '.before' "${event_path}")"
      head_sha="$(jq -r '.after' "${event_path}")"
      ;;
    *)
      base_sha=""
      head_sha=""
      ;;
  esac

  if [[ -z "${base_sha}" || "${base_sha}" == "0000000000000000000000000000000000000000" ]]; then
    changed_files="$(git ls-files)"
  else
    changed_files="$(git diff --name-only "${base_sha}" "${head_sha}")"
  fi

  rust_changed=false
  nomad_changed=false
  pulumi_changed=false
  nomad_files=()

  while IFS= read -r file; do
    [[ -z "${file}" ]] && continue

    case "${file}" in
      .cargo/*|AGENTS.md|Cargo.toml|Cargo.lock|shared/*|enterprise/*|schemas/*|platform/*|services/*|workflows/*|ui/*|xtask/*|agents/*|.github/workflows/*|.github/scripts/*)
        rust_changed=true
        ;;
    esac

    case "${file}" in
      infrastructure/nomad/*.hcl|infrastructure/nomad/**/*.hcl)
        nomad_changed=true
        nomad_files+=("${file}")
        ;;
    esac

    case "${file}" in
      infrastructure/pulumi/*|infrastructure/pulumi/**/*)
        pulumi_changed=true
        ;;
    esac
  done <<< "${changed_files}"
fi

{
  echo "rust=${rust_changed}"
  echo "nomad=${nomad_changed}"
  echo "pulumi=${pulumi_changed}"
  echo "nomad_files<<__NOMAD__"
  for file in "${nomad_files[@]}"; do
    echo "${file}"
  done
  echo "__NOMAD__"
  echo "changed_files<<__FILES__"
  printf '%s\n' "${changed_files}"
  echo "__FILES__"
} >> "${output_path}"
