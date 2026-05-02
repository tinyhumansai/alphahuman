#!/usr/bin/env bash
# Run `local-build-desktop.yml` under act — hits the same reusable
# build-desktop.yml that release-{staging,production}.yml use, but with no
# version bump, no tag, no push to upstream main, and no GH App token
# requirement. Iterate on the build matrix without burning patch numbers.
#
# Usage:
#   bash scripts/act-local-build.sh [extra act args]
# Examples:
#   bash scripts/act-local-build.sh --input build_ref=v0.53.6-staging
#   bash scripts/act-local-build.sh --matrix settings.target:x86_64-unknown-linux-gnu
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"

# Reuse act-staging.sh's setup half (regenerates .secrets / .vars / .actrc
# from scripts/ci-secrets.json) but discard its act invocation — we want
# a different entry point. --list is cheap and exits cleanly.
bash "${ROOT}/scripts/act-staging.sh" --list >/dev/null 2>&1 || true

GH_AUTH_TOKEN="$(gh auth token 2>/dev/null || true)"
if [ -n "$GH_AUTH_TOKEN" ]; then
  export GITHUB_TOKEN="$GH_AUTH_TOKEN"
fi

exec act workflow_dispatch \
  -W "${ROOT}/.github/workflows/local-build-desktop.yml" \
  --secret-file "${ROOT}/.secrets" \
  --var-file "${ROOT}/.vars" \
  --env GITHUB_REPOSITORY=tinyhumansai/openhuman \
  --env GITHUB_REPOSITORY_OWNER=tinyhumansai \
  "$@"
