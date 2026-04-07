# Pre-Commit Checks

## Husky hooks

The project uses Husky git hooks in the `app` workspace. These run automatically on `git commit` and `git push`:

- **Pre-commit**: Runs lint-staged (ESLint + Prettier on staged files)
- **Pre-push**: Runs TypeScript compilation check

If a hook fails, the commit/push is blocked. Fix the issue and retry.

## Manual checks before committing

Always run these before staging and committing:

### 1. TypeScript

```bash
yarn typecheck
```

### 2. ESLint

```bash
yarn lint
```

Fix errors. If auto-fixable:

```bash
yarn lint --fix
```

### 3. Prettier

```bash
yarn format:check
```

If failures:

```bash
yarn format
```

### 4. Rust (if Rust files changed)

```bash
cargo fmt --manifest-path Cargo.toml --all --check
cargo check --manifest-path Cargo.toml
```

For Tauri shell changes:

```bash
cargo fmt --manifest-path app/src-tauri/Cargo.toml --all --check
cargo check --manifest-path app/src-tauri/Cargo.toml
```

## Build verification

```bash
yarn build
```

For full desktop app verification:

```bash
yarn tauri dev
```

## Staging files

Be specific about what you stage. Avoid `git add -A` or `git add .` which can accidentally include:
- `.env` files
- Unrelated lock file changes
- Debug artifacts

Instead:

```bash
git add file1.ts file2.tsx file3.md
```

## Commit message style

Follow the project's conventional commit style:

```
type(scope): short description

# Examples:
refactor: replace dynamic imports with static imports in app/src
fix: resolve CORS error in ServiceBlockingGate RPC calls
feat: add team management panel to settings
```

Common types: `feat`, `fix`, `refactor`, `chore`, `docs`, `test`

## Update memory

Before committing, use the **memory-keeper** agent to update `.claude/memory.md` with any important learnings from the session:

> "Use memory-keeper to update memory"

This file lives at `.claude/memory.md` and serves as quick-reference institutional knowledge for anyone starting with Claude on this project. It captures fixes, gotchas, strict rules, and environment tips.

## Checklist

Before every commit:
- [ ] `yarn typecheck` passes
- [ ] `yarn lint` passes (0 errors)
- [ ] `yarn format:check` passes
- [ ] `yarn build` passes
- [ ] Only relevant files staged
- [ ] Commit message follows convention
- [ ] If closing an issue, include `Closes #<number>` in commit body
- [ ] `.claude/memory.md` updated with session learnings (via memory-keeper agent)

---

## GitHub Actions checks on pull requests

These jobs run on every PR to `main` (see `.github/workflows/`). Matching them locally before you push reduces CI churn.

### Workflow: `typecheck.yml` — **Type Check**

| Check name | What runs |
|------------|-----------|
| **Type Check TypeScript** | `yarn workspace openhuman-app compile` (same as `tsc --noEmit` in `app/`), then `yarn workspace openhuman-app format:check` (Prettier + Rust `cargo fmt` check for app manifests), then `yarn workspace openhuman-app lint` (ESLint on `app/`). CI sets `NODE_ENV=test`. |

| **Rust Quality (fmt + clippy)** | In container `ghcr.io/tinyhumansai/openhuman_ci:rust-1.93.0`: `cargo fmt --all -- --check`, `cargo clippy -p openhuman`. |

### Workflow: `test.yml` — **Test**

| Check name | What runs |
|------------|-----------|
| **Frontend Unit Tests** | `yarn test:coverage` (Vitest from repo root; coverage artifact uploaded). `NODE_ENV=test`. |
| **Rust Tests + Quality** | Same fmt + clippy as above, then `cargo test -p openhuman`, `cargo build --profile ci --target x86_64-unknown-linux-gnu --bin openhuman-core`, stage sidecar under `app/src-tauri/binaries/`, then `cargo test --manifest-path app/src-tauri/Cargo.toml`. |
| **E2E (Linux / tauri-driver)** | `yarn install`, `yarn workspace openhuman-app test:e2e:build`, stage sidecar next to debug binary, then core E2E specs via `app/scripts/e2e-run-spec.sh` (login-flow, smoke, navigation, telegram-flow) under Xvfb. |

### Workflow: `build.yml` — **Build**

| Check name | What runs |
|------------|-----------|
| **Build Tauri App** | In Rust CI container: `yarn install`, `cargo build` sidecar for Linux, stage binary, then `yarn tauri build` with `.deb` bundle from `app/` (production-oriented env vars in workflow). |

### Workflow: `installer-smoke.yml` — **Installer Smoke**

| Check name | What runs |
|------------|-----------|
| **Smoke install.sh (ubuntu-22.04)** | `bash scripts/install.sh --dry-run --verbose` |
| **Smoke install.sh (macos-latest)** | Same |

### Not PR-gated by default

- **`build-windows.yml`**, **`release.yml`**, **`docker-ci-image.yml`**, **`release-packages.yml`**: `workflow_dispatch` or other triggers — not listed as standard PR checks.

### Quick local parity (approximate)

From repository root after `yarn install`:

```bash
NODE_ENV=test yarn workspace openhuman-app compile
NODE_ENV=test yarn workspace openhuman-app format:check
NODE_ENV=test yarn workspace openhuman-app lint
yarn test:coverage
```

Rust (Linux CI container or local toolchain aligned with `rust-toolchain.toml`):

```bash
cargo fmt --all -- --check
cargo clippy -p openhuman
cargo test -p openhuman
```

Optional deeper checks: `yarn workspace openhuman-app test:e2e:build` and E2E scripts (heavy); `bash scripts/install.sh --dry-run --verbose`.
