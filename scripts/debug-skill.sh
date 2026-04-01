#!/usr/bin/env bash
#
# debug-skill.sh — Run the skills debug E2E test against a real skill.
#
# Usage:
#   bash scripts/debug-skill.sh                          # test example-skill (auto-find dir)
#   bash scripts/debug-skill.sh gmail                    # test a specific skill
#   bash scripts/debug-skill.sh gmail /path/to/skills    # explicit skills dir
#   bash scripts/debug-skill.sh gmail "" get-emails '{"query":"test"}'
#
# Environment variables (override via export or inline):
#   SKILL_DEBUG_ID        — skill ID (default: example-skill)
#   SKILL_DEBUG_DIR       — path to skills dir containing skill folders
#   SKILL_DEBUG_TOOL      — tool name to call (default: first tool)
#   SKILL_DEBUG_TOOL_ARGS — JSON args for the tool (default: "{}")
#   SKILL_DEBUG_VERBOSE   — "1" for verbose logging
#   RUST_LOG              — Rust log filter (default: info)
#
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Parse positional args
SKILL_ID="${1:-${SKILL_DEBUG_ID:-example-skill}}"
SKILLS_DIR="${2:-${SKILL_DEBUG_DIR:-}}"
TOOL_NAME="${3:-${SKILL_DEBUG_TOOL:-}}"
TOOL_ARGS="${4:-${SKILL_DEBUG_TOOL_ARGS:-}}"

export SKILL_DEBUG_ID="$SKILL_ID"
[ -n "$SKILLS_DIR" ] && export SKILL_DEBUG_DIR="$SKILLS_DIR"
[ -n "$TOOL_NAME" ] && export SKILL_DEBUG_TOOL="$TOOL_NAME"
[ -n "$TOOL_ARGS" ] && export SKILL_DEBUG_TOOL_ARGS="$TOOL_ARGS"

# Default log level
export RUST_LOG="${RUST_LOG:-info}"

echo "╔══════════════════════════════════════════════════════╗"
echo "║  Skills Debug Runner                                 ║"
echo "╠══════════════════════════════════════════════════════╣"
echo "║  Skill:     $SKILL_ID"
echo "║  Skills dir: ${SKILL_DEBUG_DIR:-<auto-detect>}"
echo "║  Tool:       ${SKILL_DEBUG_TOOL:-<first available>}"
echo "║  Tool args:  ${SKILL_DEBUG_TOOL_ARGS:-{}}"
echo "║  RUST_LOG:   $RUST_LOG"
echo "╚══════════════════════════════════════════════════════╝"
echo ""

cd "$REPO_ROOT"

# Run just the full lifecycle test by default, with output
cargo test --test skills_debug_e2e skill_full_lifecycle -- --nocapture 2>&1

echo ""
echo "Done. To run all skill tests (including edge cases):"
echo "  cargo test --test skills_debug_e2e -- --nocapture"
echo ""
echo "To test a specific skill with verbose logging:"
echo "  SKILL_DEBUG_ID=gmail SKILL_DEBUG_VERBOSE=1 RUST_LOG=debug bash scripts/debug-skill.sh"
