/**
 * Settings tab API layer for the Intelligence page.
 *
 * Wraps the existing `local_ai_*` core RPCs (re-exported with cleaner names)
 * and exposes a small set of stub helpers for the new
 * `openhuman.memory_tree_set_chat_backend` RPC that ships in
 * Worktree 1 (`feat/memory-cloud-default-backend`). The stubs here let the UI
 * render and respond interactively before the backend RPC is wired in.
 *
 * Logging convention: `[intelligence-settings-api]` prefix for grep-friendly
 * tracing of the new flow per the project debug-logging rule.
 */
import {
  type LocalAiAssetsStatus,
  type LocalAiDiagnostics,
  type LocalAiStatus,
  openhumanLocalAiAssetsStatus,
  openhumanLocalAiDiagnostics,
  openhumanLocalAiDownloadAsset,
  openhumanLocalAiPresets,
  openhumanLocalAiStatus,
  type PresetsResponse,
} from '../../utils/tauriCommands';

/** AI backend the assistant is currently using for chat. */
export type Backend = 'cloud' | 'local';

/** Static descriptor used by ModelAssignment + ModelCatalog. */
export interface ModelDescriptor {
  /** Ollama-style identifier (e.g. `qwen2.5:0.5b`). */
  id: string;
  /** Pretty label shown in the UI (defaults to `id` when omitted). */
  label?: string;
  /** Human-readable disk size, e.g. `400 MB`. */
  size: string;
  /** Bytes — approximate; surfaced for sort / filter. */
  approxBytes: number;
  /** Approx RAM hint, e.g. `≤4 GB RAM`. */
  ramHint: string;
  /** Speed / quality tier — used for the inline annotation under each row. */
  category: 'fast' | 'balanced' | 'high quality' | 'embedder';
  /** One-sentence note about when to pick this model. */
  note: string;
  /** Role(s) this model is suitable for. */
  roles: ReadonlyArray<'extract' | 'summariser' | 'embedder'>;
}

export type ModelRole = 'extract' | 'summariser' | 'embedder';

/**
 * Hard-coded recommended catalog. In a future wave this should come from
 * a `local_ai.recommended_catalog` RPC; for v1 we ship a curated list so
 * the UI is fully populated without a server roundtrip.
 */
export const RECOMMENDED_MODEL_CATALOG: ReadonlyArray<ModelDescriptor> = [
  {
    id: 'qwen2.5:0.5b',
    size: '400 MB',
    approxBytes: 400 * 1024 * 1024,
    ramHint: '≤4 GB RAM',
    category: 'fast',
    note: 'compact, lower quality',
    roles: ['extract'],
  },
  {
    id: 'gemma3:1b-it-qat',
    size: '1.7 GB',
    approxBytes: Math.round(1.7 * 1024 * 1024 * 1024),
    ramHint: '≤8 GB RAM',
    category: 'balanced',
    note: 'default summariser',
    roles: ['extract', 'summariser'],
  },
  {
    id: 'llama3.1:8b',
    size: '4.9 GB',
    approxBytes: Math.round(4.9 * 1024 * 1024 * 1024),
    ramHint: '≥8 GB RAM',
    category: 'high quality',
    note: 'for capable machines',
    roles: ['extract', 'summariser'],
  },
  {
    id: 'bge-m3',
    size: '1.3 GB',
    approxBytes: Math.round(1.3 * 1024 * 1024 * 1024),
    ramHint: '≥4 GB RAM',
    category: 'embedder',
    note: 'required for embeddings',
    roles: ['embedder'],
  },
];

export const DEFAULT_EXTRACT_MODEL = 'qwen2.5:0.5b';
export const DEFAULT_SUMMARISER_MODEL = 'gemma3:1b-it-qat';
export const REQUIRED_EMBEDDER_MODEL = 'bge-m3';

/**
 * In-memory backend choice — survives the React tree but not a full reload.
 * When the new `memory_tree_set_chat_backend` RPC lands, swap this for a
 * persistent core call.
 */
let mockBackend: Backend = 'cloud';

/**
 * Returns the current chat backend.
 *
 * TODO: wire when Worktree 1 lands — call
 * `openhuman.memory_tree_get_chat_backend` instead of returning the mock.
 */
export async function getChatBackend(): Promise<Backend> {
  console.debug('[intelligence-settings-api] getChatBackend (mock)', { mockBackend });
  return mockBackend;
}

/**
 * Switches the chat backend. Returns the effective value the core agreed on
 * — for the mock that is just the input, but the real RPC may downgrade
 * `local` → `cloud` if the user's device cannot satisfy the local minimums.
 *
 * TODO: wire when Worktree 1 lands —
 * `openhuman.memory_tree_set_chat_backend` (mocks any `local_ai_*` work
 * required to spin up the local pipeline).
 */
export async function setChatBackend(next: Backend): Promise<{ effective: Backend }> {
  console.debug('[intelligence-settings-api] setChatBackend (mock)', { next });
  mockBackend = next;
  return { effective: next };
}

/** Re-export the existing assets status fetch with a friendlier name. */
export async function fetchInstalledAssets(): Promise<LocalAiAssetsStatus | null> {
  try {
    const response = await openhumanLocalAiAssetsStatus();
    return response.result;
  } catch (err) {
    console.debug('[intelligence-settings-api] fetchInstalledAssets failed', err);
    return null;
  }
}

/**
 * Fetch local AI status (includes per-capability state + last latency).
 * Used by `CurrentlyLoaded` to render Ollama-side telemetry.
 */
export async function fetchLocalAiStatus(): Promise<LocalAiStatus | null> {
  try {
    const response = await openhumanLocalAiStatus();
    return response.result;
  } catch (err) {
    console.debug('[intelligence-settings-api] fetchLocalAiStatus failed', err);
    return null;
  }
}

/**
 * Reach into the existing diagnostics RPC for the list of installed Ollama
 * models. The diagnostics endpoint already enumerates them and is the
 * cleanest single source of truth — we do not duplicate the model table.
 */
export async function fetchInstalledModels(): Promise<LocalAiDiagnostics['installed_models']> {
  try {
    const response = await openhumanLocalAiDiagnostics();
    return response.installed_models ?? [];
  } catch (err) {
    console.debug('[intelligence-settings-api] fetchInstalledModels failed', err);
    return [];
  }
}

export async function fetchPresets(): Promise<PresetsResponse | null> {
  try {
    return await openhumanLocalAiPresets();
  } catch (err) {
    console.debug('[intelligence-settings-api] fetchPresets failed', err);
    return null;
  }
}

/**
 * Trigger a download for a capability (chat / vision / embedding / stt / tts).
 * Used by ModelCatalog when the user clicks "Download".
 *
 * NOTE: the real RPC is per-capability, not per-model-id, so the catalog
 * picks the closest matching capability. This is acceptable for v1; future
 * iterations can swap in a per-model RPC.
 */
export async function downloadAsset(
  capability: 'chat' | 'vision' | 'embedding' | 'stt' | 'tts'
): Promise<LocalAiAssetsStatus | null> {
  try {
    const response = await openhumanLocalAiDownloadAsset(capability);
    return response.result;
  } catch (err) {
    console.debug('[intelligence-settings-api] downloadAsset failed', { capability, err });
    return null;
  }
}

/** Map a model descriptor to the closest capability bucket the core exposes. */
export function capabilityForModel(model: ModelDescriptor): 'chat' | 'embedding' | null {
  if (model.roles.includes('embedder')) return 'embedding';
  if (model.roles.includes('extract') || model.roles.includes('summariser')) return 'chat';
  return null;
}

/**
 * Cheap pretty-printer for a byte count. Mirrors the `JetBrains Mono`-style
 * compact format we want in the technical-readout sections.
 */
export function formatBytes(bytes: number): string {
  if (!Number.isFinite(bytes) || bytes <= 0) return '—';
  const gb = bytes / (1024 * 1024 * 1024);
  if (gb >= 1) return `${gb.toFixed(1)} GB`;
  const mb = bytes / (1024 * 1024);
  return `${Math.round(mb)} MB`;
}

/**
 * Test-only — reset the in-memory backend to its default. Exported so unit
 * tests can isolate state across cases without leaking through React.
 */
export function __resetMockBackendForTests(): void {
  mockBackend = 'cloud';
}
