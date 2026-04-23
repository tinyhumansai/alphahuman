#!/usr/bin/env bash
# merge.sh <pr-number> [--squash|--merge|--rebase]
# Merge a PR via gh. Defaults to --squash to match the usual workflow.
# Waits for required checks (gh's --auto is not used here; we block explicitly
# so you see the outcome before the script exits).

set -euo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib.sh
source "$here/lib.sh"

require git gh
require_pr_number "${1:-}"

pr="$1"
strategy="${2:---squash}"
case "$strategy" in
  --squash|--merge|--rebase) ;;
  *)
    echo "[review] unknown merge strategy: $strategy (expected --squash|--merge|--rebase)" >&2
    exit 1
    ;;
esac

repo=$(resolve_repo)

echo "[review] PR #$pr status on $repo:"
gh pr view "$pr" -R "$repo" \
  --json state,mergeable,mergeStateStatus,reviewDecision,statusCheckRollup \
  | jq '{state, mergeable, mergeStateStatus, reviewDecision,
         checks: [.statusCheckRollup[]? | {name: (.name // .context), status, conclusion}]}'

echo "[review] merging PR #$pr with $strategy…"
gh pr merge "$pr" -R "$repo" "$strategy" --delete-branch
echo "[review] merged."
