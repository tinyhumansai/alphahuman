#!/usr/bin/env bash
# review.sh <pr-number>
# Sync the PR locally, then hand off to the pr-reviewer agent to produce a
# CodeRabbit-style review, post it, and approve the PR if it looks good.

set -euo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib.sh
source "$here/lib.sh"

require git gh jq claude
require_pr_number "${1:-}"
sync_pr "$1"

claude "I've already checked out branch pr/$REVIEW_PR with main merged in and \
upstream tracking set (repo: $REVIEW_REPO_RESOLVED). Use the pr-reviewer agent \
to produce a CodeRabbit-style review of PR #$REVIEW_PR and publish review \
comments. After the review is posted and if the changes look acceptable \
overall, approve the PR with \`gh pr review $REVIEW_PR -R $REVIEW_REPO_RESOLVED --approve\`. \
If blocking issues remain, request changes instead of approving."
