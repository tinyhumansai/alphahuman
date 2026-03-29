# Providers

React context providers manage service lifecycle and provide shared state.

## Provider chain

The providers wrap the application in a specific order (`app/src/App.tsx`):

```tsx
<Sentry.ErrorBoundary>
  <Provider store={store}>
    <PersistGate persistor={persistor} onBeforeLift={...}>
      <UserProvider>
        <SocketProvider>
          <AIProvider>
            <SkillProvider>
              <Router>
                <AppRoutes />
              </Router>
            </SkillProvider>
          </AIProvider>
        </SocketProvider>
      </UserProvider>
    </PersistGate>
  </Provider>
</Sentry.ErrorBoundary>
```

(`Router` is `HashRouter` from `react-router-dom`.)

**Order matters because:**

1. Redux is outermost for store access.
2. `PersistGate` rehydrates persisted slices before children rely on auth.
3. `SocketProvider` uses the JWT from the store.
4. `AIProvider` / `SkillProvider` depend on socket and store-backed features.
5. The router supplies navigation to all routes.

## SocketProvider (`app/src/providers/SocketProvider.tsx`)

Manages realtime connectivity: **web** uses the JS Socket.io client; **Tauri** bridges to the Rust socket via `utils/tauriSocket.ts` and reports status back to Redux.

### Responsibilities

- Connect when `auth.token` is available; disconnect when cleared
- In Tauri: install listeners once, connect Rust socket, coordinate daemon lifecycle (`useDaemonLifecycle`)
- Update Redux socket slice / connection status

### Implementation

See **`app/src/providers/SocketProvider.tsx`**. The file branches on **`isTauri()`**: web mode uses `socketService` directly; Tauri sets up `tauriSocket` listeners and `connectRustSocket` / `disconnectRustSocket`. Do not treat the pseudocode below as the live implementation.

### Usage

```typescript
import { useSocket } from '../providers/SocketProvider';

function MyComponent() {
  const { socket, isConnected, emit, on, off } = useSocket();

  useEffect(() => {
    const handler = (data) => console.log('Received:', data);
    on('event-name', handler);
    return () => off('event-name', handler);
  }, [on, off]);

  const sendMessage = () => {
    emit('send-message', { text: 'Hello!' });
  };

  return (
    <div>
      <span>Status: {isConnected ? 'Connected' : 'Disconnected'}</span>
      <button onClick={sendMessage}>Send</button>
    </div>
  );
}
```

## AIProvider (`app/src/providers/AIProvider.tsx`)

Initializes **memory**, **sessions**, **tool registry** (including memory + web-search tools), **entity manager**, **LLM / embedding providers**, and **constitution** loading. Exposes `useAI()` for children. Heavy logic lives under `app/src/lib/ai/`.

## SkillProvider (`app/src/providers/SkillProvider.tsx`)

On mount (when authenticated), discovers skills from the **QuickJS** skills engine via Tauri helpers (`runtimeDiscoverSkills`), syncs manifests into Redux, listens for skill-related Tauri events, and can auto-start configured skills in development.

## UserProvider (`providers/UserProvider.tsx`)

Minimal user context provider (most user state is in Redux).

### Responsibilities

- Legacy user context for compatibility
- May be deprecated in favor of Redux

### Implementation

```typescript
interface UserContextValue {
  user: User | null;
  loading: boolean;
}

export function UserProvider({ children }) {
  const user = useAppSelector((state) => state.user.profile);
  const loading = useAppSelector((state) => state.user.loading);

  return (
    <UserContext.Provider value={{ user, loading }}>
      {children}
    </UserContext.Provider>
  );
}
```

### Usage

```typescript
import { useUserContext } from '../providers/UserProvider';

function Header() {
  const { user, loading } = useUserContext();

  if (loading) return <Skeleton />;
  if (!user) return null;

  return <span>Welcome, {user.firstName}</span>;
}
```

## Provider Patterns

### Effect-Based Lifecycle

Providers use `useEffect` to manage service lifecycle:

```typescript
useEffect(() => {
  // Setup on mount or dependency change
  service.connect();

  // Cleanup on unmount or dependency change
  return () => {
    service.disconnect();
  };
}, [dependencies]);
```

### Redux Integration

Providers read from and dispatch to Redux:

```typescript
// Read state
const token = useAppSelector((state) => state.auth.token);

// Dispatch actions
const dispatch = useAppDispatch();
dispatch(setStatus({ userId, status: "connected" }));
```

### Parallel initialization

`SkillProvider` and `AIProvider` may kick off several async tasks on mount (skill discovery, memory init, constitution load). Prefer reading the source for ordering guarantees rather than assuming parallel `Promise.all` everywhere.

### Session Restoration

Providers restore persisted state on mount:

```typescript
useEffect(() => {
  if (persistedSession) {
    service.restoreSession(persistedSession);
  }
}, [persistedSession]);
```

## Context vs Redux

| Use Context For                    | Use Redux For                      |
| ---------------------------------- | ---------------------------------- |
| Service instances (socket, client) | Serializable state (status, data)  |
| Methods (emit, on, off)            | Persisted state (sessions, tokens) |
| Derived values                     | Complex state logic                |

Example:

- `SocketContext` provides `socket` instance and `emit` method
- Redux stores `socketStatus` and `socketId`

## Testing Providers

### Mock Provider for Tests

```typescript
// test-utils.tsx
const mockSocketContext: SocketContextValue = {
  socket: null,
  isConnected: true,
  emit: jest.fn(),
  on: jest.fn(),
  off: jest.fn()
};

export function TestProviders({ children }) {
  return (
    <Provider store={testStore}>
      <SocketContext.Provider value={mockSocketContext}>
        {children}
      </SocketContext.Provider>
    </Provider>
  );
}
```

### Testing Provider Effects

```typescript
test('SocketProvider connects when token is available', () => {
  const store = createTestStore({ auth: { token: 'test-token' } });

  render(
    <Provider store={store}>
      <SocketProvider>
        <TestComponent />
      </SocketProvider>
    </Provider>
  );

  expect(socketService.connect).toHaveBeenCalledWith('test-token');
});
```

---

_Previous: [Components](./06-components.md) | Next: [Hooks & Utils](./08-hooks-utils.md)_
