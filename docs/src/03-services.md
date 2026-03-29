# Services Layer

The application uses singleton services for external communication. This prevents connection leaks and provides consistent API access.

## Service architecture

```
app/src/services/
  ├─ apiClient (HTTP REST)
  │   ├─ reads auth.token from Redux
  │   └─ calls VITE_BACKEND_URL (see utils/config.ts)
  ├─ socketService (Socket.io)
  │   ├─ web: JS client
  │   └─ Tauri: coordinates with Rust-side socket via utils/tauriSocket.ts
  ├─ coreRpcClient.ts
  │   └─ invoke('core_rpc_relay', …) → local openhuman core (JSON-RPC)
  └─ services/api/* — domain REST modules (auth, user, teams, …)
```

## API Client (`services/apiClient.ts`)

HTTP REST client for backend communication.

### Features

- Fetch-based implementation
- Auto-injects JWT from Redux store
- Typed request/response handling
- Error handling with typed errors

### Usage

```typescript
import apiClient from "../services/apiClient";

// GET request
const user = await apiClient.get<User>("/users/me");

// POST request
const result = await apiClient.post<LoginResponse>("/auth/login", {
  email,
  password,
});

// With custom headers
const data = await apiClient.get<Data>("/endpoint", {
  headers: { "X-Custom": "value" },
});
```

### Configuration

Reads `VITE_BACKEND_URL` from environment or uses default:

```typescript
const BACKEND_URL =
  import.meta.env.VITE_BACKEND_URL || "https://api.example.com";
```

## API Endpoints (`services/api/`)

### Auth API (`services/api/authApi.ts`)

Authentication-related endpoints.

```typescript
import { authApi } from "../services/api/authApi";

// Login
const { token, user } = await authApi.login(credentials);

// Token exchange (for deep link flow)
const { sessionToken, user } = await authApi.exchangeToken(loginToken);

// Logout
await authApi.logout();
```

### User API (`services/api/userApi.ts`)

User profile endpoints.

```typescript
import { userApi } from "../services/api/userApi";

// Get current user
const user = await userApi.getCurrentUser();

// Update profile
const updated = await userApi.updateProfile({ firstName, lastName });

// Get settings
const settings = await userApi.getSettings();
```

## Socket Service (`services/socketService.ts`)

Socket.io client singleton for real-time communication.

### Features

- Singleton pattern - single connection per app
- Auth token passed in socket `auth` object
- Transports: polling first, then WebSocket upgrade
- Auto-reconnection handling

### API

```typescript
import socketService from "../services/socketService";

// Connect with auth token
socketService.connect(token);

// Disconnect
socketService.disconnect();

// Emit event
socketService.emit("event-name", data);

// Listen for events
socketService.on("event-name", (data) => {
  // Handle event
});

// Remove listener
socketService.off("event-name", handler);

// One-time listener
socketService.once("event-name", (data) => {
  // Handle once
});

// Get socket instance
const socket = socketService.getSocket();

// Check connection status
const isConnected = socketService.isConnected();
```

### Connection Flow

```typescript
// In SocketProvider.tsx
useEffect(() => {
  if (token) {
    socketService.connect(token);

    socketService.on("connect", () => {
      dispatch(setSocketStatus({ userId, status: "connected" }));
      dispatch(setSocketId({ userId, socketId: socket.id }));
      // Initialize MCP server
      initMCPServer(socketService.getSocket());
    });

    socketService.on("disconnect", () => {
      dispatch(setSocketStatus({ userId, status: "disconnected" }));
    });
  }

  return () => {
    socketService.disconnect();
  };
}, [token]);
```

### Configuration

```typescript
const socket = io(BACKEND_URL, {
  auth: { token },
  transports: ["polling", "websocket"],
  reconnection: true,
  reconnectionAttempts: 5,
  reconnectionDelay: 1000,
});
```

### Socket event contract (Tauri)

In Tauri mode, connection and events are bridged through **`utils/tauriSocket.ts`** (`setupTauriSocketListeners`, `connectRustSocket`, etc.). See `providers/SocketProvider.tsx` for the full flow (including daemon lifecycle hooks).

## Core RPC (`services/coreRpcClient.ts`)

The desktop app runs a separate **`openhuman`** Rust binary (staged under `app/src-tauri/binaries/`). The UI calls JSON-RPC methods on that process through Tauri:

```typescript
import { callCoreRpc } from "../services/coreRpcClient";

const result = await callCoreRpc<MyType>({
  method: "some.openhuman.method",
  params: {
    /* … */
  },
  serviceManaged: false, // true if the relay should ensure the systemd/launchd-style service
});
```

Implementation: `invoke('core_rpc_relay', { request: { method, params, serviceManaged } })` → `app/src-tauri/src/commands/core_relay.rs` → HTTP client in `app/src-tauri/src/core_rpc.rs`.

## Service integration with providers

### SocketProvider

`app/src/providers/SocketProvider.tsx` connects when `auth.token` is present. In **Tauri**, it prefers the Rust-backed socket path; in **web**, it uses the JS Socket.io client. See the source for logging and `useDaemonLifecycle` integration.

### UserProvider, AIProvider, SkillProvider

These wrap user profile loading, AI/memory client coordination, and skills catalog/sync. They sit **inside** `PersistGate` and **outside** or alongside the router as shown in `App.tsx`.

## Best Practices

1. **Use singletons** - Never create multiple service instances
2. **Store sessions in Redux** - Not localStorage
3. **Clean up on unmount** - Disconnect in useEffect cleanup
4. **Handle errors gracefully** - Retry for transient failures
5. **Pass auth via proper channels** - Socket auth object, not query string

---

_Previous: [State Management](./02-state-management.md) | Next: [MCP System](./04-mcp-system.md)_
