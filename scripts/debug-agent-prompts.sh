#!/usr/bin/env bash
#
# debug-agent-prompts.sh — Dump the exact system prompt the context engine
# would produce for every built-in agent (plus the main / orchestrator
# agent), so prompt-engineering changes can be reviewed in one place.
#
# Each prompt is written to a numbered file under the output directory
# along with a side-car `.meta.txt` containing the metadata banner
# (agent id, model, tool count, cache boundary, …) that the CLI prints
# to stderr. Useful workflow:
#
#   bash scripts/debug-agent-prompts.sh
#   diff -u prompts.before/integrations_agent.md prompts.after/integrations_agent.md
#
# The dumper runs against the real session construction path
# (`Agent::from_config_for_agent` → `Agent::build_system_prompt`), so the
# Composio surface reflects the signed-in user's actual integrations.
# If you need the toolkit list populated, sign in via the desktop app or
# point `OPENHUMAN_WORKSPACE` at a workspace that already holds the
# connection state.
#
# The dumper runs against the currently-logged-in user's workspace
# (`$OPENHUMAN_WORKSPACE`, falling back to `~/.openhuman/workspace`) so
# onboarding-generated files like `PROFILE.md` appear in the dump. Export
# `OPENHUMAN_WORKSPACE=<path>` before running if you want to target a
# different workspace.
#
# Usage:
#   bash scripts/debug-agent-prompts.sh [--out <dir>] [--with-tools] [-v]
#
# The output directory is wiped and recreated at the start of each run
# so the snapshot only reflects the current agent set — stale files from
# an earlier run cannot hide a regression.
#
# Defaults:
#   --out          ./prompt-dumps   (deleted + recreated each run)
#   --with-tools   OFF (pass to also list each agent's tool names)
#

set -euo pipefail

# ── Locate repo root + binary ─────────────────────────────────────────────
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
BIN="${REPO_ROOT}/target/debug/openhuman-core"

# Load the repo .env so staging/prod backend URLs, API keys, and the
# Composio toggle reach the dumped prompts. `Config::load_or_init`
# calls `apply_env_overrides` after reading from disk, so any variable
# exported here wins over whatever is baked into the workspace config.
# Mirrors `yarn tauri dev`, which sources the same file via
# `scripts/load-dotenv.sh` before launching the sidecar.
if [[ -f "${REPO_ROOT}/.env" ]]; then
  echo "[debug-agent-prompts] loading env from ${REPO_ROOT}/.env" >&2
  # shellcheck disable=SC1091
  source "${SCRIPT_DIR}/load-dotenv.sh" "${REPO_ROOT}/.env"
fi

# The project's CLI logger writes to stdout (not stderr), so any
# `RUST_LOG` value inherited from `.env` (typically `info`) would
# interleave log lines into the JSON/prompt payloads this script
# expects on stdout. Force quiet unless the caller passed `-v` — in
# which case the later `--verbose` flag restores debug logging.
export RUST_LOG=error

# Always run `cargo build` — it no-ops when the binary is already
# up-to-date, and re-links quickly when it isn't. The old `-x` existence
# check let a stale debug binary survive across agent-registry changes
# (e.g. new entries in `agents::BUILTINS`), which made this script
# silently skip newly added agents like `welcome`.
echo "[debug-agent-prompts] building openhuman-core (no-op if up-to-date) …" >&2
( cd "${REPO_ROOT}" && cargo build --manifest-path Cargo.toml --bin openhuman-core >&2 )

# ── Parse flags ───────────────────────────────────────────────────────────
OUT_DIR=""
WITH_TOOLS=0
VERBOSE_FLAG=()

while [[ $# -gt 0 ]]; do
  case "$1" in
    --out)
      if [[ -z "${2-}" ]] || [[ "${2-}" == -* ]]; then
        echo "[debug-agent-prompts] missing value for --out" >&2
        exit 64
      fi
      OUT_DIR="$2"
      shift 2
      ;;
    --with-tools)
      WITH_TOOLS=1
      shift
      ;;
    -v|--verbose)
      VERBOSE_FLAG=(-v)
      shift
      ;;
    -h|--help)
      sed -n '2,38p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'
      exit 0
      ;;
    *)
      echo "[debug-agent-prompts] unknown flag: $1" >&2
      exit 64
      ;;
  esac
done

if [[ -z "${OUT_DIR}" ]]; then
  OUT_DIR="${REPO_ROOT}/prompt-dumps"
fi

# ── Validate & canonicalize OUT_DIR before `rm -rf` ─────────────────────
# The output directory is wiped at the start of each run. Literal string
# matching against "/" / $HOME / $REPO_ROOT is not enough on its own:
# trailing slashes, ".", "..", or symlinked paths can slip past and
# trigger `rm -rf` on a sensitive target. So:
#
#   1. Reject obviously bad inputs up-front ("", ".", "..", relative).
#   2. Canonicalize OUT_DIR and REPO_ROOT via `realpath` (falling back
#      to python when realpath is unavailable on barebones macOS).
#   3. Match the canonicalized form against the disallow list.
#   4. Only then `rm -rf` the canonicalized path.
case "${OUT_DIR}" in
  "" | "." | "..")
    echo "[debug-agent-prompts] refusing to wipe --out='${OUT_DIR}' (relative/empty)" >&2
    exit 64
    ;;
esac
if [[ "${OUT_DIR}" != /* ]]; then
  echo "[debug-agent-prompts] --out must be an absolute path (starts with '/'), got '${OUT_DIR}'" >&2
  exit 64
fi

canonicalize() {
  local p="$1"
  # `realpath` is GNU + modern macOS (coreutils), and `readlink -f` on
  # Linux. Try both; if neither resolves the path (target missing) we
  # fall back to python3, which handles symlinks even for non-existent
  # leaves via `os.path.realpath`.
  if command -v realpath >/dev/null 2>&1; then
    realpath -m -- "${p}" 2>/dev/null && return 0
  fi
  if command -v readlink >/dev/null 2>&1 && readlink -f / >/dev/null 2>&1; then
    readlink -f -- "${p}" 2>/dev/null && return 0
  fi
  python3 -c 'import os,sys; print(os.path.realpath(sys.argv[1]))' "${p}"
}

resolved_out="$(canonicalize "${OUT_DIR}")"
resolved_repo="$(canonicalize "${REPO_ROOT}")"
resolved_home="$(canonicalize "${HOME}")"

if [[ -z "${resolved_out}" ]]; then
  echo "[debug-agent-prompts] failed to canonicalize --out='${OUT_DIR}'" >&2
  exit 64
fi
case "${resolved_out}" in
  "/" | "${resolved_home}" | "${resolved_repo}")
    echo "[debug-agent-prompts] refusing to wipe --out (resolves to ${resolved_out})" >&2
    exit 64
    ;;
esac

# Use the canonicalized path from here on so every subsequent command
# (rm, mkdir, per-agent dump writes) operates on the same resolved
# target — no symlink window between validation and deletion.
OUT_DIR="${resolved_out}"
rm -rf "${OUT_DIR}"
mkdir -p "${OUT_DIR}"

# Workspace resolution is owned by `Config::load_or_init` inside the
# binary: it reads `~/.openhuman/active_user.toml`, falls back to the
# persisted workspace marker, then to the pre-login user directory. We
# only pass `--workspace` when the caller has explicitly exported one
# (an empty `OPENHUMAN_WORKSPACE=` in `.env` counts as unset — the
# binary's resolver is what we want in that case).
#
# Previously this script duplicated the resolution in shell and guessed
# wrong when the user's active install used a multi-user layout under
# `~/.openhuman/users/<user_id>/workspace` without a top-level
# `active_user.toml`, causing the dumper to bail with "workspace not
# found". Delegating to the binary removes that divergence and makes
# `.env` (including `OPENHUMAN_APP_ENV=staging`) take effect
# automatically.
WORKSPACE_OVERRIDE=""
if [[ -n "${OPENHUMAN_WORKSPACE:-}" ]]; then
  WORKSPACE_OVERRIDE="${OPENHUMAN_WORKSPACE}"
fi

echo "[debug-agent-prompts] output dir : ${OUT_DIR}" >&2
if [[ -n "${WORKSPACE_OVERRIDE}" ]]; then
  echo "[debug-agent-prompts] workspace  : ${WORKSPACE_OVERRIDE} (OPENHUMAN_WORKSPACE override)" >&2
else
  echo "[debug-agent-prompts] workspace  : <resolved by Config::load_or_init>" >&2
fi
if [[ -n "${OPENHUMAN_APP_ENV:-}" ]]; then
  echo "[debug-agent-prompts] app env    : ${OPENHUMAN_APP_ENV}" >&2
fi
if [[ -n "${OPENHUMAN_BASE_URL:-}" ]]; then
  echo "[debug-agent-prompts] base url   : ${OPENHUMAN_BASE_URL}" >&2
fi
echo >&2

# ── Discover agent ids from `agent list --json` ───────────────────────────
# `mapfile` is bash 4+, but macOS ships bash 3 — use a portable
# read-while-IFS loop instead so the script works out of the box on a
# vanilla `/bin/bash`.
LIST_ARGS=(agent list --json)
if [[ -n "${WORKSPACE_OVERRIDE}" ]]; then
  LIST_ARGS+=(--workspace "${WORKSPACE_OVERRIDE}")
fi
AGENT_LIST_JSON="$("${BIN}" "${LIST_ARGS[@]}" 2>/dev/null)"
AGENT_IDS=()
while IFS= read -r line; do
  [[ -n "${line}" ]] && AGENT_IDS+=("${line}")
done < <(printf '%s' "${AGENT_LIST_JSON}" | python3 -c '
import json, sys
for entry in json.load(sys.stdin):
    aid = entry.get("id", "")
    # The synthetic `fork` definition replays the parent verbatim and
    # has no standalone prompt — skip it.
    if aid and aid != "fork":
        print(aid)
')

# Every registered agent — orchestrator included. There's no
# "main" alias anymore: the dumper treats the orchestrator as just
# another agent, which keeps the per-agent render pipeline uniform.
TARGETS=("${AGENT_IDS[@]}")

# ── Build common dump-prompt flag list ────────────────────────────────────
DUMP_FLAGS=()
if [[ -n "${WORKSPACE_OVERRIDE}" ]]; then
  DUMP_FLAGS+=(--workspace "${WORKSPACE_OVERRIDE}")
fi
if [[ ${WITH_TOOLS} -eq 1 ]]; then
  DUMP_FLAGS+=(--with-tools)
fi
if [[ ${#VERBOSE_FLAG[@]} -gt 0 ]]; then
  DUMP_FLAGS+=("${VERBOSE_FLAG[@]}")
fi

# ── Dump every target ─────────────────────────────────────────────────────
INDEX=0
SUMMARY=""
for AGENT in "${TARGETS[@]}"; do
  INDEX=$((INDEX + 1))
  SAFE_NAME="$(printf '%s' "${AGENT}" | tr -c 'A-Za-z0-9._-' '_')"
  PROMPT_PATH="${OUT_DIR}/${INDEX}_${SAFE_NAME}.md"
  META_PATH="${OUT_DIR}/${INDEX}_${SAFE_NAME}.meta.txt"

  printf '[debug-agent-prompts] %-20s → %s\n' "${AGENT}" "${PROMPT_PATH}" >&2
  if "${BIN}" agent dump-prompt --agent "${AGENT}" ${DUMP_FLAGS[@]+"${DUMP_FLAGS[@]}"} \
        > "${PROMPT_PATH}" 2> "${META_PATH}"; then
    LINES="$(wc -l < "${PROMPT_PATH}" | tr -d ' ')"
    TOOL_COUNT="$(grep -E '^tool_count:' "${META_PATH}" | awk '{print $2}')"
    SKILL_COUNT="$(grep -E '^skill_tools:' "${META_PATH}" | awk '{print $2}')"
    SUMMARY+="$(printf '%-20s lines=%-5s tools=%-4s skill=%-4s\n' \
        "${AGENT}" "${LINES}" "${TOOL_COUNT:-?}" "${SKILL_COUNT:-?}")
"
  else
    echo "[debug-agent-prompts]   ✘ failed to dump ${AGENT} (see ${META_PATH})" >&2
    SUMMARY+="$(printf '%-20s FAILED — see %s\n' "${AGENT}" "${META_PATH}")
"
  fi
done

# ── Write a summary index file alongside the dumps ────────────────────────
SUMMARY_PATH="${OUT_DIR}/SUMMARY.txt"
{
  echo "OpenHuman agent prompt dump summary"
  echo "Generated: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
  if [[ -n "${WORKSPACE_OVERRIDE}" ]]; then
    echo "Workspace: ${WORKSPACE_OVERRIDE} (OPENHUMAN_WORKSPACE override)"
  else
    echo "Workspace: <resolved by Config::load_or_init>"
  fi
  if [[ -n "${OPENHUMAN_APP_ENV:-}" ]]; then
    echo "App env: ${OPENHUMAN_APP_ENV}"
  fi
  if [[ -n "${OPENHUMAN_BASE_URL:-}" ]]; then
    echo "Base URL: ${OPENHUMAN_BASE_URL}"
  fi
  echo
  echo "${SUMMARY}"
} > "${SUMMARY_PATH}"

echo >&2
echo "[debug-agent-prompts] done — ${INDEX} prompts dumped" >&2
echo "[debug-agent-prompts] summary  : ${SUMMARY_PATH}" >&2
