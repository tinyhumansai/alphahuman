# Architecture overview

## System architecture

OpenHuman’s desktop UI is a **React 19** app (`app/src/`) that:

- Uses **Redux Toolkit** with persistence for session-related state
- Connects to the backend with **REST** (`apiClient`) and **Socket.io** (`socketService`)
- Calls the **Rust core** process over HTTP via **`coreRpcClient`** / Tauri **`core_rpc_relay`** (JSON-RPC methods implemented in repo root `src/openhuman/`, exposed through `core_server`)
- Loads **AI prompts** from bundled `src/openhuman/agent/prompts` (repo root) and from Tauri **`ai_get_config`** when packaged
- Uses a **minimal MCP-style** helper layer under `lib/mcp/` (transport, validation) — not a large in-repo Telegram MCP tool bundle

## Entry points

| File                    | Purpose                                                                              |
| ----------------------- | ------------------------------------------------------------------------------------ |
| `app/src/main.tsx`      | React root, Sentry boundary, store, global styles                                    |
| `app/src/App.tsx`       | Provider chain: Redux → PersistGate → User → Socket → AI → Skill → Router            |
| `app/src/AppRoutes.tsx` | `HashRouter` routes, `ProtectedRoute` / `PublicRoute`, onboarding and mnemonic gates |

## Provider chain

```
Redux Provider
  └─ PersistGate
      └─ UserProvider
          └─ SocketProvider
              └─ AIProvider
                  └─ SkillProvider
                      └─ HashRouter
                          └─ AppRoutes (pages + settings)
```

**Why this order**

1. Redux is outermost for `useAppSelector` / dispatch everywhere.
2. `PersistGate` rehydrates persisted slices before children assume stable auth.
3. `SocketProvider` uses the auth token for Socket.io.
4. `AIProvider` / `SkillProvider` wrap features that depend on socket and store state.
5. `HashRouter` supplies navigation to all routes.

## Module relationships (simplified)

```
App.tsx
  ├─ Redux store + persistor
  ├─ UserProvider — user profile / workspace context
  ├─ SocketProvider — connects socketService when token present
  ├─ AIProvider — AI session / memory client coordination
  ├─ SkillProvider — skills catalog and sync
  └─ AppRoutes
       ├─ PublicRoute — e.g. Welcome on `/`
       ├─ ProtectedRoute — onboarding, home, skills, settings, …
       └─ DefaultRedirect — unauthenticated users
```

## Services layer (conceptual)

```
services/
  ├─ apiClient        → REST to VITE_BACKEND_URL with JWT from Redux
  ├─ socketService    → Socket.io; realtime + MCP-style envelopes
  └─ coreRpcClient    → HTTP to local openhuman core (JSON-RPC), used with Tauri relay
```

## Related docs

- Rust architecture: [`../ARCHITECTURE.md`](../ARCHITECTURE.md)
- Tauri shell: [`../src-tauri/01-architecture.md`](../src-tauri/01-architecture.md)
