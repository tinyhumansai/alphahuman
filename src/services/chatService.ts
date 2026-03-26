/**
 * Chat Service — sends messages via Rust backend (Tauri) or falls back to
 * frontend-driven orchestration (web mode).
 */
import { isTauri as coreIsTauri, invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

// ─── Event payload types (must match Rust structs exactly — snake_case) ───────

export interface ChatToolCallEvent {
  thread_id: string;
  tool_name: string;
  skill_id: string;
  args: Record<string, unknown>;
  round: number;
}

export interface ChatToolResultEvent {
  thread_id: string;
  tool_name: string;
  skill_id: string;
  output: string;
  success: boolean;
  round: number;
}

export interface ChatDoneEvent {
  thread_id: string;
  full_response: string;
  rounds_used: number;
  total_input_tokens: number;
  total_output_tokens: number;
}

export interface ChatErrorEvent {
  thread_id: string;
  message: string;
  error_type: 'network' | 'timeout' | 'tool_error' | 'inference' | 'cancelled';
  round: number | null;
}

// ─── Listener setup ───────────────────────────────────────────────────────────

export interface ChatEventListeners {
  onToolCall?: (event: ChatToolCallEvent) => void;
  onToolResult?: (event: ChatToolResultEvent) => void;
  onDone?: (event: ChatDoneEvent) => void;
  onError?: (event: ChatErrorEvent) => void;
}

/**
 * Subscribe to chat events from the Rust backend.
 * Returns a cleanup function that removes all listeners.
 * Only works in Tauri mode.
 */
export async function subscribeChatEvents(listeners: ChatEventListeners): Promise<() => void> {
  const unlisteners: UnlistenFn[] = [];

  if (listeners.onToolCall) {
    const cb = listeners.onToolCall;
    unlisteners.push(await listen<ChatToolCallEvent>('chat:tool_call', e => cb(e.payload)));
  }
  if (listeners.onToolResult) {
    const cb = listeners.onToolResult;
    unlisteners.push(await listen<ChatToolResultEvent>('chat:tool_result', e => cb(e.payload)));
  }
  if (listeners.onDone) {
    const cb = listeners.onDone;
    unlisteners.push(await listen<ChatDoneEvent>('chat:done', e => cb(e.payload)));
  }
  if (listeners.onError) {
    const cb = listeners.onError;
    unlisteners.push(await listen<ChatErrorEvent>('chat:error', e => cb(e.payload)));
  }

  return () => {
    for (const unlisten of unlisteners) {
      unlisten();
    }
  };
}

// ─── Send message ─────────────────────────────────────────────────────────────

export interface ChatSendParams {
  threadId: string;
  message: string;
  model: string;
  authToken: string;
  backendUrl: string;
  messages: Array<{ role: string; content: string; tool_calls?: unknown[]; tool_call_id?: string }>;
  notionContext?: string | null;
}

/**
 * Send a message via the Rust chat_send command.
 * Returns immediately — results arrive via events.
 * Tauri v2 converts camelCase param names to snake_case for the Rust command.
 */
export async function chatSend(params: ChatSendParams): Promise<void> {
  await invoke('chat_send', {
    threadId: params.threadId,
    message: params.message,
    model: params.model,
    authToken: params.authToken,
    backendUrl: params.backendUrl,
    messages: params.messages,
    notionContext: params.notionContext ?? null,
  });
}

/**
 * Cancel an in-flight chat request.
 */
export async function chatCancel(threadId: string): Promise<boolean> {
  return await invoke<boolean>('chat_cancel', { threadId });
}

/**
 * Check if we should use the Rust backend for chat.
 * Returns true when running in Tauri on desktop.
 */
export function useRustChat(): boolean {
  return coreIsTauri();
}
