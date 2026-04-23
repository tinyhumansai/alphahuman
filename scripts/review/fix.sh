#!/usr/bin/env bash
# fix.sh <pr-number>
# Sync the PR, run pr-reviewer to identify issues and apply fixes, then hand
# off to pr-manager-lite to run the quality suite, commit, and push.

set -euo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib.sh
source "$here/lib.sh"

require git gh jq claude
require_pr_number "${1:-}"
sync_pr "$1"

claude "I've already checked out branch pr/$REVIEW_PR with main merged in and \
upstream tracking set (repo: $REVIEW_REPO_RESOLVED). Use the pr-reviewer agent \
to review PR #$REVIEW_PR and fix the issues it finds. Then use the \
pr-manager-lite agent to run the quality suite, commit, and push the changes \
back to the PR branch."
