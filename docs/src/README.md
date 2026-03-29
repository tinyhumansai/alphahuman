# Source code documentation (`app/src/`)

This section describes the **OpenHuman** desktop app frontend: the Vite + React tree under **`app/src/`** in the monorepo (Yarn workspace `openhuman-app`).

## Quick reference

| Document                                      | Description                                         |
| --------------------------------------------- | --------------------------------------------------- |
| [Architecture overview](./01-architecture.md) | Entry points, provider chain, module relationships  |
| [State management](./02-state-management.md)  | Redux Toolkit, persistence, selectors               |
| [Services layer](./03-services.md)            | API client, Socket.io, core RPC client              |
| [MCP utilities](./04-mcp-system.md)           | Transport and types (no bundled Telegram tool pack) |
| [Pages & routing](./05-pages-routing.md)      | `HashRouter`, guards, main routes                   |
| [Components](./06-components.md)              | UI and settings patterns                            |
| [Providers](./07-providers.md)                | User, Socket, AI, Skill providers                   |
| [Hooks & utils](./08-hooks-utils.md)          | Shared hooks and helpers                            |

## Scale (approximate)

| Metric                                  | Value                                                                       |
| --------------------------------------- | --------------------------------------------------------------------------- |
| TypeScript / TSX files under `app/src/` | ~285 (run `find app/src -name '*.ts' -o -name '*.tsx' \| wc -l` to refresh) |
| Test runner                             | Vitest (`app/test/vitest.config.ts`)                                        |

## Directory layout (high level)

```
app/src/
├── App.tsx                 # Provider chain + HashRouter shell
├── AppRoutes.tsx           # Route table + guards
├── main.tsx                # Entry (Sentry, store, styles)
├── store/                  # Redux slices and selectors
├── providers/              # UserProvider, SocketProvider, AIProvider, SkillProvider
├── services/               # apiClient, socketService, coreRpcClient, api/*
├── lib/                    # AI loaders, MCP helpers, skills sync, etc.
├── pages/                  # Route-level screens
├── components/             # Shared UI
├── hooks/                  # App hooks
├── utils/                  # Config, Tauri helpers, routing utilities
└── assets/                 # Icons and static assets
```

## Architectural decisions

1. **HashRouter** — Fits Tauri and deep-link flows better than browser history in many desktop setups.
2. **Redux Toolkit + persist** — Centralized state; selective persistence for auth and related slices.
3. **Core RPC client** — Business logic and skills run in the **`openhuman`** Rust sidecar; the UI calls it via HTTP (`core_rpc_relay` / `coreRpcClient`), not only REST.
4. **No MTProto provider** — The current tree does not ship a `TelegramProvider` or `mtprotoService`; any Telegram mentions may be legacy UI or future channels work.

## Getting started

1. Read [Architecture overview](./01-architecture.md).
2. Skim [State management](./02-state-management.md) and [Services](./03-services.md).
3. Use [Pages & routing](./05-pages-routing.md) when changing navigation.

---

_Documentation for the `app/src/` tree; paths are relative to the repository root._
