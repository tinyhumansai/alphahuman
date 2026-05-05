import debug from 'debug';
import { useCallback, useEffect, useRef } from 'react';

import { requestUsageRefresh } from '../hooks/usageRefresh';
import { useRefetchSnapshotOnTurnEnd } from '../hooks/useRefetchSnapshotOnTurnEnd';
import { notifyThreadMessagesRefresh } from '../lib/threads/messagesRefreshBus';
import { useThreads } from '../lib/threads/ThreadsContext';
import { useActiveThread } from '../lib/threads/useActiveThread';
import { threadApi } from '../services/api/threadApi';
import {
  type ChatInferenceStartEvent,
  type ChatIterationStartEvent,
  type ChatSegmentEvent,
  type ChatSubagentDoneEvent,
  type ChatToolCallEvent,
  type ChatToolResultEvent,
  type ProactiveMessageEvent,
  subscribeChatEvents,
} from '../services/chatService';
import { store } from '../store';
import {
  clearInferenceStatusForThread,
  clearStreamingAssistantForThread,
  endInferenceTurn,
  markInferenceTurnStreaming,
  recordChatTurnUsage,
  setInferenceStatusForThread,
  setStreamingAssistantForThread,
  setToolTimelineForThread,
  type StreamingAssistantState,
  type ToolTimelineEntry,
  type ToolTimelineEntryStatus,
} from '../store/chatRuntimeSlice';
import { useAppDispatch, useAppSelector } from '../store/hooks';
import { selectSocketStatus } from '../store/socketSelectors';
import { IS_PROD } from '../utils/config';
import { formatTimelineEntry, promptFromArgsBuffer } from '../utils/toolTimelineFormatting';

const logChatRuntime = debug('openhuman:chat-runtime');

function rtLog(message: string, fields?: Record<string, string | number | null | undefined>) {
  if (IS_PROD) return;
  if (fields && Object.keys(fields).length > 0) {
    const parts = Object.entries(fields)
      .filter(([, v]) => v !== undefined && v !== '' && v !== null)
      .map(([k, v]) => `${k}=${v}`);
    logChatRuntime('[chat-runtime] %s %s', message, parts.join(' '));
  } else {
    logChatRuntime('[chat-runtime] %s', message);
  }
}

const ChatRuntimeProvider = ({ children }: { children: React.ReactNode }) => {
  const dispatch = useAppDispatch();
  const { refetch: refetchSnapshot } = useRefetchSnapshotOnTurnEnd();
  const socketStatus = useAppSelector(selectSocketStatus);
  const toolTimelineByThread = useAppSelector(state => state.chatRuntime.toolTimelineByThread);
  const inferenceStatusByThread = useAppSelector(
    state => state.chatRuntime.inferenceStatusByThread
  );
  const streamingAssistantByThread = useAppSelector(
    state => state.chatRuntime.streamingAssistantByThread
  );

  // Threads context: used to resolve the visible thread for proactive messages
  // and to trigger title generation + list refresh after a turn ends.
  const { threads, create: createThread, generateTitleIfNeeded } = useThreads();
  const { activeThreadId, setActiveThreadId } = useActiveThread();

  // Keep refs so event handlers (closures captured at subscribe-time) can
  // read the latest value without stale-closure bugs.
  const threadsRef = useRef(threads);
  const activeThreadIdRef = useRef(activeThreadId);

  useEffect(() => {
    threadsRef.current = threads;
  }, [threads]);

  useEffect(() => {
    activeThreadIdRef.current = activeThreadId;
  }, [activeThreadId]);

  const seenChatEventsRef = useRef<Map<string, number>>(new Map());
  const proactiveThreadCreationPromiseRef = useRef<Promise<string | null> | null>(null);
  const proactiveDispatchQueueRef = useRef<Promise<void>>(Promise.resolve());
  const toolTimelineRef = useRef(toolTimelineByThread);
  const inferenceStatusRef = useRef(inferenceStatusByThread);
  const streamingAssistantRef = useRef(streamingAssistantByThread);

  useEffect(() => {
    toolTimelineRef.current = toolTimelineByThread;
  }, [toolTimelineByThread]);

  useEffect(() => {
    inferenceStatusRef.current = inferenceStatusByThread;
  }, [inferenceStatusByThread]);

  useEffect(() => {
    streamingAssistantRef.current = streamingAssistantByThread;
  }, [streamingAssistantByThread]);

  const markChatEventSeen = (
    key: string,
    meta?: { threadId?: string; requestId?: string }
  ): boolean => {
    const now = Date.now();
    const cache = seenChatEventsRef.current;
    const ttlMs = 10 * 60_000;
    const maxEntries = 500;

    if (cache.has(key)) {
      rtLog('dedupe_drop', {
        key: key.length > 160 ? `${key.slice(0, 160)}…` : key,
        thread: meta?.threadId,
        request: meta?.requestId,
      });
      return false;
    }
    cache.set(key, now);

    for (const [existingKey, timestamp] of cache) {
      if (now - timestamp > ttlMs) {
        cache.delete(existingKey);
      }
    }

    while (cache.size > maxEntries) {
      const oldest = cache.keys().next().value;
      if (!oldest) break;
      cache.delete(oldest);
    }
    return true;
  };

  const proactiveMessageDigest = (input: string): string => {
    // Small non-cryptographic digest to keep dedupe keys bounded.
    let hash = 2166136261;
    for (let i = 0; i < input.length; i += 1) {
      hash ^= input.charCodeAt(i);
      hash = Math.imul(hash, 16777619);
    }
    return (hash >>> 0).toString(36);
  };

  const resolveVisibleThreadForProactive = useCallback(
    async (incomingThreadId: string): Promise<string | null> => {
      if (!incomingThreadId.startsWith('proactive:')) {
        return incomingThreadId;
      }

      // Resolution priority: selected (URL) > active (in-flight inference) > first in list.
      // Read from refs so we don't capture stale state in the closure.
      const currentActiveId = activeThreadIdRef.current;
      const currentThreads = threadsRef.current;

      // Try to get selected thread from URL.
      const urlHash = typeof window !== 'undefined' ? window.location.hash : '';
      const qIdx = urlHash.indexOf('?');
      let selectedFromUrl: string | null = null;
      if (qIdx >= 0) {
        selectedFromUrl = new URLSearchParams(urlHash.slice(qIdx + 1)).get('t');
      }

      const targetFromState = selectedFromUrl ?? currentActiveId ?? currentThreads[0]?.id ?? null;

      if (targetFromState) {
        return targetFromState;
      }

      if (proactiveThreadCreationPromiseRef.current) {
        return proactiveThreadCreationPromiseRef.current;
      }

      const createPromise: Promise<string | null> = (async () => {
        try {
          const newThread = await createThread();
          rtLog('proactive_thread_created', { id: newThread.id });
          return newThread.id;
        } catch (error) {
          rtLog('proactive_thread_create_failed', {
            err: error instanceof Error ? error.message : String(error),
          });
          return null;
        } finally {
          proactiveThreadCreationPromiseRef.current = null;
        }
      })();
      proactiveThreadCreationPromiseRef.current = createPromise;

      try {
        return await createPromise;
      } finally {
        // no-op: cleared in createPromise.finally
      }
    },
    [createThread]
  );

  useEffect(() => {
    if (socketStatus !== 'connected') return;

    const decorateEntry = (entry: ToolTimelineEntry): ToolTimelineEntry => {
      const formatted = formatTimelineEntry(entry);
      return { ...entry, displayName: formatted.title, detail: formatted.detail };
    };

    const findPendingDelegationContext = (
      entries: ToolTimelineEntry[],
      round: number
    ): { sourceToolName?: string; prompt?: string } => {
      for (let i = entries.length - 1; i >= 0; i -= 1) {
        const entry = entries[i];
        if (entry.status !== 'running' || entry.round !== round) continue;
        if (entry.name === 'spawn_subagent' || entry.name.startsWith('delegate_')) {
          return {
            sourceToolName: entry.name,
            prompt: entry.detail ?? promptFromArgsBuffer(entry.argsBuffer),
          };
        }
      }
      return {};
    };

    rtLog('subscribe_chat_events', { socket: socketStatus });
    const cleanup = subscribeChatEvents({
      onInferenceStart: (event: ChatInferenceStartEvent) => {
        rtLog('inference_start', { thread: event.thread_id, request: event.request_id });
        dispatch(markInferenceTurnStreaming({ threadId: event.thread_id }));
        dispatch(
          setInferenceStatusForThread({
            threadId: event.thread_id,
            status: { phase: 'thinking', iteration: 0, maxIterations: 0 },
          })
        );
      },
      onIterationStart: (event: ChatIterationStartEvent) => {
        const prev = inferenceStatusRef.current[event.thread_id];
        rtLog('iteration_start', {
          thread: event.thread_id,
          request: event.request_id,
          iteration: event.round,
        });
        dispatch(
          setInferenceStatusForThread({
            threadId: event.thread_id,
            status: {
              phase: 'thinking',
              iteration: event.round,
              maxIterations: prev?.maxIterations ?? 0,
            },
          })
        );
      },
      onToolCall: (event: ChatToolCallEvent) => {
        const prev = store.getState().chatRuntime.inferenceStatusByThread[event.thread_id];
        dispatch(
          setInferenceStatusForThread({
            threadId: event.thread_id,
            status: {
              ...(prev ?? { iteration: event.round, maxIterations: 0 }),
              phase: 'tool_use',
              activeTool: event.tool_name,
            },
          })
        );

        const eventKey = `tool_call:${event.thread_id}:${event.request_id ?? 'none'}:${event.round}:${event.tool_name}:${event.tool_call_id ?? ''}`;
        if (
          !markChatEventSeen(eventKey, { threadId: event.thread_id, requestId: event.request_id })
        )
          return;

        const existing = store.getState().chatRuntime.toolTimelineByThread[event.thread_id] ?? [];
        const existingIdx = event.tool_call_id
          ? existing.findIndex(entry => entry.id === event.tool_call_id)
          : -1;

        let entries: ToolTimelineEntry[];
        if (existingIdx >= 0) {
          entries = [...existing];
          entries[existingIdx] = decorateEntry({
            ...entries[existingIdx],
            name: event.tool_name,
            round: event.round,
            status: 'running',
          });
        } else {
          entries = [
            ...existing,
            decorateEntry({
              id:
                event.tool_call_id ??
                `${event.thread_id}:${event.round}:${existing.length}:${event.tool_name}`,
              name: event.tool_name,
              round: event.round,
              status: 'running',
            }),
          ];
        }
        dispatch(setToolTimelineForThread({ threadId: event.thread_id, entries }));
      },
      onToolResult: (event: ChatToolResultEvent) => {
        const eventKey = `tool_result:${event.thread_id}:${event.request_id ?? 'none'}:${event.round}:${event.tool_name}:${event.success}:${event.tool_call_id ?? ''}`;
        if (
          !markChatEventSeen(eventKey, { threadId: event.thread_id, requestId: event.request_id })
        )
          return;

        const existing = store.getState().chatRuntime.toolTimelineByThread[event.thread_id] ?? [];
        if (existing.length > 0) {
          const nextEntries = [...existing];
          let changed = false;

          if (event.tool_call_id) {
            const idx = nextEntries.findIndex(entry => entry.id === event.tool_call_id);
            if (idx >= 0) {
              nextEntries[idx] = {
                ...nextEntries[idx],
                status: event.success ? 'success' : 'error',
              };
              changed = true;
            }
          }

          if (!changed) {
            for (let i = nextEntries.length - 1; i >= 0; i -= 1) {
              const entry = nextEntries[i];
              if (
                entry.status === 'running' &&
                entry.name === event.tool_name &&
                entry.round === event.round
              ) {
                nextEntries[i] = { ...entry, status: event.success ? 'success' : 'error' };
                changed = true;
                break;
              }
            }
          }

          if (changed) {
            dispatch(setToolTimelineForThread({ threadId: event.thread_id, entries: nextEntries }));
          }
        }

        const current = store.getState().chatRuntime.inferenceStatusByThread[event.thread_id];
        if (!current) return;
        dispatch(
          setInferenceStatusForThread({
            threadId: event.thread_id,
            status: { ...current, phase: 'thinking', activeTool: undefined },
          })
        );
      },
      onSubagentSpawned: event => {
        const prev = store.getState().chatRuntime.inferenceStatusByThread[event.thread_id];
        dispatch(
          setInferenceStatusForThread({
            threadId: event.thread_id,
            status: {
              ...(prev ?? { iteration: event.round, maxIterations: 0 }),
              phase: 'subagent',
              activeSubagent: event.tool_name,
            },
          })
        );

        const existing = store.getState().chatRuntime.toolTimelineByThread[event.thread_id] ?? [];
        const pendingContext = findPendingDelegationContext(existing, event.round);
        dispatch(
          setToolTimelineForThread({
            threadId: event.thread_id,
            entries: [
              ...existing,
              decorateEntry({
                id: `${event.thread_id}:subagent:${event.skill_id}:${event.tool_name}`,
                name: `subagent:${event.tool_name}`,
                round: event.round,
                status: 'running',
                detail: pendingContext.prompt,
                sourceToolName: pendingContext.sourceToolName,
                subagent: {
                  taskId: event.skill_id,
                  agentId: event.tool_name,
                  mode: event.subagent?.mode,
                  dedicatedThread: event.subagent?.dedicated_thread,
                  toolCalls: [],
                },
              }),
            ],
          })
        );
      },
      onSubagentDone: (event: ChatSubagentDoneEvent) => {
        const subagentRowId = `${event.thread_id}:subagent:${event.skill_id}:${event.tool_name}`;
        const existing = store.getState().chatRuntime.toolTimelineByThread[event.thread_id] ?? [];
        if (existing.length > 0) {
          const entries = existing.map(entry => {
            if (entry.id !== subagentRowId || entry.status !== 'running') return entry;
            return decorateEntry({
              ...entry,
              status: (event.success ? 'success' : 'error') as ToolTimelineEntryStatus,
              subagent: entry.subagent
                ? {
                    ...entry.subagent,
                    iterations: event.subagent?.iterations ?? entry.subagent.iterations,
                    elapsedMs: event.subagent?.elapsed_ms ?? entry.subagent.elapsedMs,
                    outputChars: event.subagent?.output_chars ?? entry.subagent.outputChars,
                  }
                : entry.subagent,
            });
          });
          dispatch(setToolTimelineForThread({ threadId: event.thread_id, entries }));
        }

        const current = store.getState().chatRuntime.inferenceStatusByThread[event.thread_id];
        if (!current) return;
        dispatch(
          setInferenceStatusForThread({
            threadId: event.thread_id,
            status: { ...current, phase: 'thinking', activeSubagent: undefined },
          })
        );
      },
      onSubagentIterationStart: event => {
        const taskId = event.subagent?.task_id ?? event.skill_id;
        const agentId = event.subagent?.agent_id ?? event.tool_name;
        const rowId = `${event.thread_id}:subagent:${taskId}:${agentId}`;
        const existing = store.getState().chatRuntime.toolTimelineByThread[event.thread_id] ?? [];
        const idx = existing.findIndex(entry => entry.id === rowId);
        if (idx < 0) return;
        const entry = existing[idx];
        if (!entry.subagent) return;
        const next = [...existing];
        next[idx] = {
          ...entry,
          subagent: {
            ...entry.subagent,
            childIteration: event.subagent?.child_iteration ?? entry.subagent.childIteration,
            childMaxIterations:
              event.subagent?.child_max_iterations ?? entry.subagent.childMaxIterations,
          },
        };
        dispatch(setToolTimelineForThread({ threadId: event.thread_id, entries: next }));
      },
      onSubagentToolCall: event => {
        const taskId = event.subagent?.task_id ?? event.skill_id;
        const agentId = event.subagent?.agent_id;
        if (!agentId) return;
        const rowId = `${event.thread_id}:subagent:${taskId}:${agentId}`;
        const existing = store.getState().chatRuntime.toolTimelineByThread[event.thread_id] ?? [];
        const idx = existing.findIndex(entry => entry.id === rowId);
        if (idx < 0) return;
        const entry = existing[idx];
        if (!entry.subagent) return;
        // De-dupe on call_id — the same call should not append twice if
        // the socket layer redelivers (e.g. on reconnect during a run).
        if (entry.subagent.toolCalls.some(c => c.callId === event.tool_call_id)) return;
        const next = [...existing];
        next[idx] = {
          ...entry,
          subagent: {
            ...entry.subagent,
            toolCalls: [
              ...entry.subagent.toolCalls,
              {
                callId: event.tool_call_id,
                toolName: event.tool_name,
                status: 'running',
                iteration: event.subagent?.child_iteration,
              },
            ],
          },
        };
        dispatch(setToolTimelineForThread({ threadId: event.thread_id, entries: next }));
      },
      onSubagentToolResult: event => {
        const taskId = event.subagent?.task_id ?? event.skill_id;
        const agentId = event.subagent?.agent_id;
        if (!agentId) return;
        const rowId = `${event.thread_id}:subagent:${taskId}:${agentId}`;
        const existing = store.getState().chatRuntime.toolTimelineByThread[event.thread_id] ?? [];
        const idx = existing.findIndex(entry => entry.id === rowId);
        if (idx < 0) return;
        const entry = existing[idx];
        if (!entry.subagent) return;
        const callIdx = entry.subagent.toolCalls.findIndex(c => c.callId === event.tool_call_id);
        if (callIdx < 0) return;
        const updatedCalls = [...entry.subagent.toolCalls];
        updatedCalls[callIdx] = {
          ...updatedCalls[callIdx],
          status: event.success ? 'success' : 'error',
          elapsedMs: event.subagent?.elapsed_ms ?? updatedCalls[callIdx].elapsedMs,
          outputChars: event.subagent?.output_chars ?? updatedCalls[callIdx].outputChars,
        };
        const next = [...existing];
        next[idx] = { ...entry, subagent: { ...entry.subagent, toolCalls: updatedCalls } };
        dispatch(setToolTimelineForThread({ threadId: event.thread_id, entries: next }));
      },
      // onSegment: streaming text segments arrive during a turn and are
      // accumulated into the streaming-preview state (chatRuntimeSlice). The
      // agent already writes each segment to JSONL on the Rust side, so we
      // do NOT call threadApi.appendMessage here — that would duplicate rows.
      // After chat_done we refresh from JSONL to show the canonical content.
      onSegment: (event: ChatSegmentEvent) => {
        // Segments are tracked by chatRuntimeSlice for streaming preview;
        // we only need to dedupe so no double-counting on reconnect.
        const eventKey = `segment:${event.thread_id}:${event.request_id}:${event.segment_index}`;
        markChatEventSeen(eventKey, { threadId: event.thread_id, requestId: event.request_id });
        // Streaming preview content is driven by onTextDelta below.
      },
      onTextDelta: event => {
        const cr = store.getState().chatRuntime;
        const existing = cr.streamingAssistantByThread[event.thread_id];
        let streaming: StreamingAssistantState;
        if (existing && existing.requestId !== event.request_id) {
          streaming = { requestId: event.request_id, content: event.delta, thinking: '' };
        } else {
          streaming = {
            requestId: event.request_id,
            content: `${existing?.content ?? ''}${event.delta}`,
            thinking: existing?.thinking ?? '',
          };
        }
        dispatch(setStreamingAssistantForThread({ threadId: event.thread_id, streaming }));
      },
      onThinkingDelta: event => {
        const cr = store.getState().chatRuntime;
        const existing = cr.streamingAssistantByThread[event.thread_id];
        let streaming: StreamingAssistantState;
        if (existing && existing.requestId !== event.request_id) {
          streaming = { requestId: event.request_id, content: '', thinking: event.delta };
        } else {
          streaming = {
            requestId: event.request_id,
            content: existing?.content ?? '',
            thinking: `${existing?.thinking ?? ''}${event.delta}`,
          };
        }
        dispatch(setStreamingAssistantForThread({ threadId: event.thread_id, streaming }));
      },
      onToolArgsDelta: event => {
        const cr = store.getState().chatRuntime;
        const existing = cr.toolTimelineByThread[event.thread_id] ?? [];
        let matchIdx = -1;
        if (event.tool_call_id) {
          matchIdx = existing.findIndex(entry => entry.id === event.tool_call_id);
        }
        if (matchIdx < 0 && event.tool_name) {
          matchIdx = existing.findIndex(
            entry =>
              entry.status === 'running' &&
              entry.name === event.tool_name &&
              entry.round === event.round
          );
        }

        let entries: ToolTimelineEntry[];
        if (matchIdx >= 0) {
          entries = [...existing];
          entries[matchIdx] = decorateEntry({
            ...entries[matchIdx],
            argsBuffer: `${entries[matchIdx].argsBuffer ?? ''}${event.delta}`,
            name:
              entries[matchIdx].name.length === 0 && event.tool_name
                ? event.tool_name
                : entries[matchIdx].name,
          });
        } else {
          entries = [
            ...existing,
            decorateEntry({
              id: event.tool_call_id,
              name: event.tool_name ?? '',
              round: event.round,
              status: 'running',
              argsBuffer: event.delta,
            }),
          ];
        }
        dispatch(setToolTimelineForThread({ threadId: event.thread_id, entries }));
      },
      onProactiveMessage: (event: ProactiveMessageEvent) => {
        const messageDigest = proactiveMessageDigest(event.full_response ?? '');
        const eventKey = `proactive:${event.thread_id}:${event.request_id ?? 'none'}:${messageDigest}`;
        if (
          !markChatEventSeen(eventKey, { threadId: event.thread_id, requestId: event.request_id })
        )
          return;

        proactiveDispatchQueueRef.current = proactiveDispatchQueueRef.current.then(async () => {
          try {
            const targetThreadId = await resolveVisibleThreadForProactive(event.thread_id);
            if (!targetThreadId) return;
            rtLog('proactive_message', {
              from: event.thread_id,
              to: targetThreadId,
              request: event.request_id,
            });
            // Append the proactive message to the core (JSONL) directly — the
            // agent side may not have written it if it came via the proactive path.
            const proactiveMsg = {
              id: `msg_${globalThis.crypto?.randomUUID ? globalThis.crypto.randomUUID() : `${Date.now()}-${Math.random().toString(36).slice(2)}`}`,
              content: event.full_response,
              type: 'text' as const,
              extraMetadata: {},
              sender: 'agent' as const,
              createdAt: new Date().toISOString(),
            };
            await threadApi.appendMessage(targetThreadId, proactiveMsg);
            rtLog('messages_refresh_notify', {
              thread: targetThreadId,
              request: event.request_id,
              reason: 'proactive',
            });
            notifyThreadMessagesRefresh(targetThreadId);
          } catch (error) {
            rtLog('proactive_dispatch_failed', {
              from: event.thread_id,
              request: event.request_id,
              error: error instanceof Error ? error.message : String(error),
            });
          }
        });
      },
      onDone: event => {
        const eventKey = `done:${event.thread_id}:${event.request_id ?? 'none'}`;
        if (
          !markChatEventSeen(eventKey, { threadId: event.thread_id, requestId: event.request_id })
        )
          return;

        rtLog('chat_done', {
          thread: event.thread_id,
          request: event.request_id,
          segments: event.segment_total,
          input_tokens: event.total_input_tokens,
          output_tokens: event.total_output_tokens,
        });

        dispatch(
          recordChatTurnUsage({
            inputTokens: event.total_input_tokens,
            outputTokens: event.total_output_tokens,
          })
        );
        dispatch(clearInferenceStatusForThread({ threadId: event.thread_id }));
        dispatch(clearStreamingAssistantForThread({ threadId: event.thread_id }));

        const existing = store.getState().chatRuntime.toolTimelineByThread[event.thread_id] ?? [];
        if (existing.length > 0) {
          const entries = existing.map(entry =>
            entry.status === 'running' ? { ...entry, status: 'success' as const } : entry
          );
          dispatch(setToolTimelineForThread({ threadId: event.thread_id, entries }));
        }

        // The Rust core has already written the full assistant turn to JSONL.
        // Trigger title generation (which also refreshes the thread list) and
        // fire usage/snapshot refresh. No local message append needed.
        void generateTitleIfNeeded(event.thread_id, event.full_response).catch(err => {
          rtLog('chat_done_title_failed', {
            thread: event.thread_id,
            error: err instanceof Error ? err.message : String(err),
          });
        });

        rtLog('refresh_usage_counter', {
          thread: event.thread_id,
          request: event.request_id,
          reason: 'chat_done',
        });
        requestUsageRefresh();
        rtLog('snapshot_refetch_queued', {
          thread: event.thread_id,
          request: event.request_id,
          reason: 'chat_done',
        });
        refetchSnapshot();
        rtLog('messages_refresh_notify', {
          thread: event.thread_id,
          request: event.request_id,
          reason: 'chat_done',
        });
        notifyThreadMessagesRefresh(event.thread_id);
        dispatch(endInferenceTurn({ threadId: event.thread_id }));
        setActiveThreadId(null);
      },
      onError: event => {
        const eventKey = `error:${event.thread_id}:${event.request_id ?? 'none'}:${event.error_type}:${event.message}`;
        if (
          !markChatEventSeen(eventKey, { threadId: event.thread_id, requestId: event.request_id })
        )
          return;

        rtLog('chat_error', {
          thread: event.thread_id,
          request: event.request_id,
          err: event.error_type,
        });

        dispatch(clearInferenceStatusForThread({ threadId: event.thread_id }));
        dispatch(clearStreamingAssistantForThread({ threadId: event.thread_id }));

        const existing = store.getState().chatRuntime.toolTimelineByThread[event.thread_id] ?? [];
        if (existing.length > 0) {
          const entries = existing.map(entry =>
            entry.status === 'running' ? { ...entry, status: 'error' as const } : entry
          );
          dispatch(setToolTimelineForThread({ threadId: event.thread_id, entries }));
        }

        if (event.error_type !== 'cancelled') {
          // For non-cancelled errors, append an error message to core JSONL so
          // the user sees it on rejoin. Then fire usage/snapshot refresh.
          void (async () => {
            try {
              const errorMsg = {
                id: `msg_${globalThis.crypto?.randomUUID ? globalThis.crypto.randomUUID() : `${Date.now()}-${Math.random().toString(36).slice(2)}`}`,
                content: 'Something went wrong — please try again.',
                type: 'text' as const,
                extraMetadata: {},
                sender: 'agent' as const,
                createdAt: new Date().toISOString(),
              };
              await threadApi.appendMessage(event.thread_id, errorMsg);
              rtLog('messages_refresh_notify', {
                thread: event.thread_id,
                request: event.request_id,
                reason: 'chat_error',
              });
              notifyThreadMessagesRefresh(event.thread_id);
            } catch (appendErr) {
              rtLog('chat_error_append_failed', {
                thread: event.thread_id,
                error: appendErr instanceof Error ? appendErr.message : String(appendErr),
              });
            }
          })();

          rtLog('refresh_usage_counter', {
            thread: event.thread_id,
            request: event.request_id,
            reason: 'chat_error',
          });
          requestUsageRefresh();
        }

        dispatch(endInferenceTurn({ threadId: event.thread_id }));
        setActiveThreadId(null);
      },
    });

    return () => {
      rtLog('unsubscribe_chat_events');
      cleanup();
    };
  }, [
    dispatch,
    resolveVisibleThreadForProactive,
    socketStatus,
    refetchSnapshot,
    generateTitleIfNeeded,
    setActiveThreadId,
  ]);

  return <>{children}</>;
};

export default ChatRuntimeProvider;
