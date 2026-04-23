# scripts/review

Helpers for working through PRs on this repo. Runnable directly — no zshrc
integration needed.

| Script       | What it does                                                                 |
| ------------ | ---------------------------------------------------------------------------- |
| `sync.sh`    | Fetch PR head, check out as `pr/<num>`, merge `main`, wire push/upstream.    |
| `review.sh`  | `sync` + hand off to the `pr-reviewer` agent to review, comment, and approve.|
| `fix.sh`     | `sync` + `pr-reviewer` (apply fixes) + `pr-manager-lite` (commit & push).    |
| `merge.sh`   | Show PR status and merge via `gh pr merge` (default `--squash`).             |

## Usage

Via yarn (preferred):

```sh
yarn review sync 123
yarn review review 123
yarn review fix 123
yarn review merge 123              # --squash
yarn review merge 123 --rebase
yarn review --help
```

Or invoke the scripts directly:

```sh
scripts/review/sync.sh 123
scripts/review/review.sh 123
scripts/review/fix.sh 123
scripts/review/merge.sh 123
```

## Config

- Repo is derived from the `upstream` remote (falls back to `origin`). Override
  with `REVIEW_REPO=owner/name`.
- Requires `git`, `gh`, `jq`. `review.sh` and `fix.sh` also require `claude`.
