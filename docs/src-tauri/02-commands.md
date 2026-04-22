# Tauri IPC commands (`app/src-tauri`)

All commands are registered in **`app/src-tauri/src/lib.rs`** inside `tauri::generate_handler![...]` (desktop build). Names below are the **Rust** command names (camelCase in JS via serde where applicable).

## Demo / diagnostics

| Command | Purpose                                    |
| ------- | ------------------------------------------ |
| `greet` | Demo string (safe to remove in production) |

## AI configuration (bundled prompts)

| Command                | Purpose                                                                                      |
| ---------------------- | -------------------------------------------------------------------------------------------- |
| `ai_get_config`        | Build `AIPreview` from resolved `SOUL.md` / `TOOLS.md` under bundled or dev `src/openhuman/agent/prompts` |
| `ai_refresh_config`    | Same read path as `ai_get_config` (refresh hook)                                             |
| `write_ai_config_file` | Write a single `.md` under repo `src/openhuman/agent/prompts` (dev / safe filename checks)                |

## Core JSON-RPC relay

| Command          | Purpose                                                                                                        |
| ---------------- | -------------------------------------------------------------------------------------------------------------- |
| `core_rpc_relay` | Body: `{ method, params?, serviceManaged? }` → forwards to local **`openhuman`** HTTP JSON-RPC (`core_rpc.rs`) |

Use **`app/src/services/coreRpcClient.ts`** (`callCoreRpc`) from the frontend.

## Window management

From **`commands/window.rs`** (names may vary slightly; see `lib.rs`):

| Command             | Purpose           |
| ------------------- | ----------------- |
| `show_window`       | Show main window  |
| `hide_window`       | Hide main window  |
| `toggle_window`     | Toggle visibility |
| `is_window_visible` | Query visibility  |
| `minimize_window`   | Minimize          |
| `maximize_window`   | Maximize          |
| `close_window`      | Close             |
| `set_window_title`  | Set title string  |

## OpenHuman daemon / service helpers

From **`commands/openhuman.rs`** (see source for exact payloads):

| Command                            | Purpose                                        |
| ---------------------------------- | ---------------------------------------------- |
| `openhuman_get_daemon_host_config` | Read daemon host preferences (e.g. tray)       |
| `openhuman_set_daemon_host_config` | Persist daemon host preferences                |
| `openhuman_service_install`        | Install background service (platform-specific) |
| `openhuman_service_start`          | Start service                                  |
| `openhuman_service_stop`           | Stop service                                   |
| `openhuman_service_status`         | Query status                                   |
| `openhuman_service_uninstall`      | Uninstall service                              |

## Webview accounts

From **`webview_accounts/mod.rs`**:

| Command                     | Purpose                                                                          |
| --------------------------- | -------------------------------------------------------------------------------- |
| `webview_account_open`      | Open (or reuse) a child webview for an account; params: `OpenArgs`               |
| `webview_account_close`     | Close the webview for an account (keeps data dir); params: `AccountIdArgs`       |
| `webview_account_purge`     | Close the webview and wipe its on-disk data dir (logout); params: `AccountIdArgs` |
| `webview_account_bounds`    | Resize / reposition the child webview; params: `BoundsArgs`                      |
| `webview_account_hide`      | Hide the child webview; params: `AccountIdArgs`                                  |
| `webview_account_show`      | Show the child webview; params: `AccountIdArgs`                                  |
| `webview_recipe_event`      | Receive a scrape event from an injected recipe; params: `RecipeEventArgs`        |

### Call transcription commands

| Command                       | Params                                                                 | Returns                                                | Purpose |
| ----------------------------- | ---------------------------------------------------------------------- | ------------------------------------------------------ | ------- |
| `call_transcription_start`    | `account_id: String`, `provider: String`, `channel_name: Option<String>` | `Ok(())`                                               | Begin collecting CEF audio for the named account. Stores a `CallSession` that buffers ring-buffer samples until `call_transcription_stop` is called. No-op when the `cef` feature is disabled. |
| `call_transcription_stop`     | `account_id: String`, `reason: Option<String>`                         | `Ok(())` immediately                                   | Stop audio capture and spawn a detached background task that assembles the WAV, POSTs it to `openhuman.voice_transcribe_bytes` on the core JSON-RPC sidecar, and emits a `webview:event` with `kind: "call_transcript"` when Whisper finishes. Returns immediately so `closeWebviewAccount` is not blocked by the 120 s Whisper timeout. |
| `call_transcription_status`   | `account_id: String`                                                   | `{ account_id: String, active: bool }`                 | Query whether an active call session exists for the account. Used by the React service layer before `closeWebviewAccount` to decide whether to flush an in-progress recording. |

#### `call_transcript` event payload (`webview:event`)

Emitted on the `webview:event` channel with `kind: "call_transcript"`:

```jsonc
{
  "account_id": "<account-id>",
  "provider": "slack" | "discord" | "whatsapp",
  "kind": "call_transcript",
  "payload": {
    "provider": "slack",
    "channelName": "<channel or contact name, or null>",
    "transcript": "<Whisper transcript text>",
    "durationSecs": 120,
    "reason": "ended",
    "startedAt": 1700000000000,   // Unix ms — captured at call start
    "endedAt":   1700000120000    // Unix ms — captured before transcription
  },
  "ts": 1700000120000
}
```

## Removed / not present

The following **do not** exist in the current `generate_handler!` list: `exchange_token`, `get_auth_state`, `socket_connect`, `start_telegram_login`. Authentication and sockets are handled in the **React** app and **core** process, not via these IPC names.

## Example: core RPC

```typescript
import { invoke } from "@tauri-apps/api/core";

const result = await invoke("core_rpc_relay", {
  request: {
    method: "your.rpc.method",
    params: { foo: "bar" },
    serviceManaged: false,
  },
});
```

---

_See `app/src-tauri/src/lib.rs` for the authoritative list._
