#!/usr/bin/env bash
# Load key-value JSON into environment variables.
# Usage:
#   source scripts/load-env-json.sh path/to/file.json
#   eval "$(scripts/load-env-json.sh path/to/file.json)"
# Optional jq filter to select object (default: .):
#   source scripts/load-env-json.sh ci-secrets.json '.secrets + .vars'

set -e
FILE="${1:?Usage: $0 <file.json> [jq-filter]}"
FILTER="${2:-.}"

if [[ ! -f "$FILE" ]]; then
  echo "File not found: $FILE" >&2
  exit 1
fi

if ! command -v jq &>/dev/null; then
  echo "jq is required" >&2
  exit 1
fi

exports=$(jq -r "${FILTER} | to_entries | .[] | \"export \(.key)=\(.value | @sh)\"" "$FILE")

if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
  echo "$exports"
else
  eval "$exports"
fi
