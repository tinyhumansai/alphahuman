#!/usr/bin/env bash
# tree-summarizer-run-all.sh — Run tree summarization for every memory namespace.
#
# Discovers namespaces by listing directories under the workspace's
# memory/namespaces/ folder, then runs the tree-summarizer for each one.
#
# Usage:
#   bash scripts/tree-summarizer-run-all.sh                 # run (drain buffer + summarize)
#   bash scripts/tree-summarizer-run-all.sh status           # show status for all trees
#   bash scripts/tree-summarizer-run-all.sh query [node_id]  # query all trees
#   bash scripts/tree-summarizer-run-all.sh rebuild          # rebuild all trees from leaves
#
# Options:
#   -v, --verbose    Enable debug logging
#   --workspace DIR  Override OPENHUMAN_WORKSPACE
#   --binary PATH    Override the openhuman-core binary path

set -euo pipefail

# ── Defaults ───────────────────────────────────────────────────────────

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

VERBOSE=""
SUBCOMMAND="run"
NODE_ID=""

# Resolve binary: staged sidecar → debug build → release build
resolve_binary() {
    local arch
    arch="$(uname -m)"
    case "$arch" in
        arm64|aarch64) arch="aarch64-apple-darwin" ;;
        x86_64)        arch="x86_64-apple-darwin"  ;;
        *)             arch="$arch-unknown-linux-gnu" ;;
    esac

    for bin in \
        "$REPO_ROOT/app/src-tauri/binaries/openhuman-core-$arch" \
        "$REPO_ROOT/target/debug/openhuman-core" \
        "$REPO_ROOT/target/release/openhuman-core"; do
        if [ -x "$bin" ]; then
            echo "$bin"
            return
        fi
    done

    echo >&2 "error: could not find openhuman-core binary. Build with: cargo build --bin openhuman-core"
    exit 1
}

OPENHUMAN_BIN="${OPENHUMAN_BIN:-$(resolve_binary)}"

# Resolve workspace: env var → active user → first user dir
resolve_workspace() {
    if [ -n "${OPENHUMAN_WORKSPACE:-}" ]; then
        echo "$OPENHUMAN_WORKSPACE"
        return
    fi

    # Try the active user workspace
    local active_user_file="$HOME/.openhuman/active_user.toml"
    if [ -f "$active_user_file" ]; then
        local user_id
        user_id=$(sed -n 's/^user_id *= *"\([^"]*\)".*/\1/p' "$active_user_file" 2>/dev/null || true)
        if [ -n "$user_id" ] && [ -d "$HOME/.openhuman/users/$user_id/workspace" ]; then
            echo "$HOME/.openhuman/users/$user_id/workspace"
            return
        fi
    fi

    # Fallback: first user directory with a workspace
    for user_dir in "$HOME"/.openhuman/users/*/; do
        if [ -d "${user_dir}workspace" ]; then
            echo "${user_dir}workspace"
            return
        fi
    done

    echo >&2 "error: could not resolve OPENHUMAN_WORKSPACE. Set it explicitly."
    exit 1
}

export OPENHUMAN_WORKSPACE="${OPENHUMAN_WORKSPACE:-$(resolve_workspace)}"

# ── Parse args ─────────────────────────────────────────────────────────

while [ $# -gt 0 ]; do
    case "$1" in
        -v|--verbose)
            VERBOSE="-v"
            shift
            ;;
        --workspace)
            export OPENHUMAN_WORKSPACE="$2"
            shift 2
            ;;
        --binary)
            OPENHUMAN_BIN="$2"
            shift 2
            ;;
        run|status|query|rebuild)
            SUBCOMMAND="$1"
            shift
            # For query, grab optional node_id
            if [ "$SUBCOMMAND" = "query" ] && [ $# -gt 0 ]; then
                case "$1" in
                    -*) ;;  # skip flags
                    *)  NODE_ID="$1"; shift ;;
                esac
            fi
            ;;
        -h|--help)
            sed -n '2,/^$/{ s/^# //; s/^#$//; p }' "$0"
            exit 0
            ;;
        *)
            echo >&2 "unknown argument: $1"
            exit 1
            ;;
    esac
done

# ── Discover namespaces ────────────────────────────────────────────────

NAMESPACES_DIR="$OPENHUMAN_WORKSPACE/memory/namespaces"

if [ ! -d "$NAMESPACES_DIR" ]; then
    echo "No namespaces directory found at $NAMESPACES_DIR"
    exit 0
fi

NAMESPACES=$(find "$NAMESPACES_DIR" -mindepth 1 -maxdepth 1 -type d | while read -r d; do basename "$d"; done | sort)

if [ -z "$NAMESPACES" ]; then
    echo "No memory namespaces found."
    exit 0
fi

NS_COUNT=$(echo "$NAMESPACES" | wc -l | tr -d ' ')
NS_LIST=$(echo "$NAMESPACES" | tr '\n' ' ')

echo "Found $NS_COUNT namespace(s): $NS_LIST"
echo "Workspace: $OPENHUMAN_WORKSPACE"
echo "Binary:    $OPENHUMAN_BIN"
echo "Command:   tree-summarizer $SUBCOMMAND"
echo "---"

# ── Strip ASCII art banner from output ─────────────────────────────────

strip_banner() {
    grep -v '▗\|▐\|▝\|▀\|█\|Contribute\|OpenHuman core' | grep -v '^[[:space:]]*$'
}

# ── Run for each namespace ─────────────────────────────────────────────

FAILED=0
SUCCEEDED=0

while IFS= read -r ns; do
    echo ""
    echo "=== [$ns] ==="

    args=("$SUBCOMMAND" "$ns")
    if [ "$SUBCOMMAND" = "query" ] && [ -n "$NODE_ID" ]; then
        args+=("$NODE_ID")
    fi
    if [ -n "$VERBOSE" ]; then
        args+=("$VERBOSE")
    fi

    if output=$("$OPENHUMAN_BIN" tree-summarizer "${args[@]}" 2>&1); then
        echo "$output" | strip_banner | head -40
        SUCCEEDED=$((SUCCEEDED + 1))
    else
        echo "$output" | strip_banner | tail -5
        echo "  ^^^ FAILED"
        FAILED=$((FAILED + 1))
    fi
done <<< "$NAMESPACES"

echo ""
echo "---"
echo "Done. $SUCCEEDED succeeded, $FAILED failed out of $NS_COUNT namespace(s)."

if [ "$FAILED" -gt 0 ]; then
    exit 1
fi
