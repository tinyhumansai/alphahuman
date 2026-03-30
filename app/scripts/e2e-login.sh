#!/usr/bin/env bash
# Run E2E login flow tests only. See app/scripts/e2e-run-spec.sh.
set -euo pipefail
exec "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/e2e-run-spec.sh" "test/e2e/specs/login-flow.spec.ts" "login"
