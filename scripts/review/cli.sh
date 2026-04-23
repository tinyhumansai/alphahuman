#!/usr/bin/env bash
# Dispatcher for `yarn review <cmd> <args…>`.
# Commands: sync | review | fix | merge

set -euo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

usage() {
  cat <<EOF
Usage: yarn review <command> <pr-number> [args]

Commands:
  sync    <pr>                            Check out PR as pr/<num>, merge main, wire remotes
  review  <pr>                            Sync + pr-reviewer agent (review, comment, approve)
  fix     <pr>                            Sync + pr-reviewer (apply fixes) + pr-manager-lite (push)
  merge   <pr> [--squash|--merge|--rebase]  Merge via gh (default --squash, deletes branch)

Env:
  REVIEW_REPO=owner/name  Override target repo (default: derived from upstream remote)
EOF
}

cmd="${1:-}"
if [ -z "$cmd" ] || [ "$cmd" = "-h" ] || [ "$cmd" = "--help" ]; then
  usage
  exit 0
fi
shift

case "$cmd" in
  sync|review|fix|merge)
    exec "$here/${cmd}.sh" "$@"
    ;;
  *)
    echo "[review] unknown command: $cmd" >&2
    usage >&2
    exit 1
    ;;
esac
