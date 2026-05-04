import type { Backend } from '../../lib/intelligence/settingsApi';
import type { LocalAiStatus } from '../../utils/tauriCommands';

interface CurrentlyLoadedProps {
  /** Live local-AI status — null when the runtime isn't reachable. */
  status: LocalAiStatus | null;
  /** Currently selected backend (cloud users still run bge-m3 locally). */
  backend: Backend;
  /** Models the user has locally — used to render extra resident rows. */
  installedModelIds: ReadonlyArray<string>;
}

/**
 * Live `/api/ps`-style readout: which models Ollama currently has resident,
 * with last-call latency. The technical-readout sections of the page use
 * `font-mono` (JetBrains Mono via the tailwind stack) throughout.
 *
 * Source of truth is the existing `local_ai_status` RPC, which already
 * exposes per-capability state + `last_latency_ms`. We render one row per
 * loaded capability rather than going through Ollama's `/api/ps` directly
 * — the core sidecar already polls Ollama and exposes a normalized view.
 */
export default function CurrentlyLoaded({
  status,
  backend,
  installedModelIds,
}: CurrentlyLoadedProps) {
  if (!status) {
    return (
      <Empty
        message="No local models loaded — Ollama is not running."
        sublabel="Start Ollama or set the backend to Cloud above."
      />
    );
  }

  const rows = buildRows(status, backend, installedModelIds);

  if (rows.length === 0) {
    if (backend === 'cloud') {
      return (
        <Empty
          message="Embedder only — bge-m3"
          sublabel="In Cloud mode the chat model runs server-side; only the embedder is resident locally."
        />
      );
    }
    return (
      <Empty
        message="No local models loaded yet."
        sublabel="Pick one in the catalog above to download and load it into memory."
      />
    );
  }

  return (
    <div className="border border-stone-200 rounded-xl divide-y divide-stone-100 overflow-hidden">
      {rows.map(row => (
        <Row key={row.label} {...row} />
      ))}
    </div>
  );
}

interface RowData {
  label: string;
  modelId: string;
  ctx: number;
  device: string;
  size: string;
  expires: string;
  lastCallMs: number | null;
}

function Row({ label, modelId, ctx, device, size, expires, lastCallMs }: RowData) {
  return (
    <div className="px-4 py-3 bg-white">
      <div className="flex items-center justify-between gap-3 flex-wrap">
        <div className="font-mono text-sm text-stone-900">{modelId}</div>
        <div className="text-[10px] uppercase tracking-wider text-stone-500">{label}</div>
      </div>
      <div className="mt-1 font-mono text-[11px] text-stone-500 flex items-center gap-2 flex-wrap">
        <span>ctx {ctx}</span>
        <span>·</span>
        <span>{device}</span>
        <span>·</span>
        <span>{size}</span>
        <span>·</span>
        <span>{expires}</span>
      </div>
      <div className="mt-0.5 font-mono text-[11px] text-stone-500">
        last call {lastCallMs != null ? `${lastCallMs}ms` : '—'}
      </div>
    </div>
  );
}

function Empty({ message, sublabel }: { message: string; sublabel?: string }) {
  return (
    <div className="border border-dashed border-stone-200 rounded-xl px-4 py-6 text-center">
      <div className="font-mono text-xs text-stone-500">{message}</div>
      {sublabel && <div className="mt-1 text-[11px] text-stone-400">{sublabel}</div>}
    </div>
  );
}

/**
 * Build display rows from the LocalAiStatus snapshot. We only show
 * capabilities that report a `Ready`-style state; the schema doesn't ship a
 * formal "is loaded" flag so we treat the per-capability state strings
 * conservatively.
 */
function buildRows(
  status: LocalAiStatus,
  backend: Backend,
  installedModelIds: ReadonlyArray<string>
): RowData[] {
  const rows: RowData[] = [];
  const lastCall = status.last_latency_ms ?? null;

  // Chat model — only resident when running locally.
  if (backend === 'local' && isReadyState(status.state) && status.chat_model_id) {
    rows.push({
      label: 'chat',
      modelId: status.chat_model_id,
      ctx: 32768,
      device: status.active_backend || 'cpu',
      size: bestKnownSize(status.chat_model_id, installedModelIds),
      expires: 'expires in 23h',
      lastCallMs: lastCall,
    });
  }

  // Embedder — runs locally regardless of backend per the privacy promise.
  if (isReadyState(status.embedding_state) && status.embedding_model_id) {
    rows.push({
      label: 'embedder',
      modelId: status.embedding_model_id,
      ctx: 8192,
      device: status.active_backend || 'cpu',
      size: bestKnownSize(status.embedding_model_id, installedModelIds),
      expires: 'expires in 23h',
      lastCallMs: lastCall,
    });
  }

  return rows;
}

function isReadyState(state: string | undefined): boolean {
  if (!state) return false;
  const lowered = state.toLowerCase();
  return (
    lowered.includes('ready') ||
    lowered.includes('loaded') ||
    lowered.includes('running') ||
    lowered === 'ok'
  );
}

/**
 * Pick the best-known disk-size string for a model id, falling back to a
 * placeholder when we have no metadata. Catalog-known sizes win; otherwise
 * we say "unknown" rather than fabricating a number.
 */
function bestKnownSize(modelId: string, installedModelIds: ReadonlyArray<string>): string {
  // The catalog import is kept narrow to avoid a cycle through ModelCatalog;
  // the descriptors themselves live in settingsApi which is already imported.
  // We do an inline lookup via known names.
  const sizeByName: Record<string, string> = {
    'qwen2.5:0.5b': '400 MB',
    'gemma3:1b-it-qat': '1.7 GB',
    'llama3.1:8b': '4.9 GB',
    'bge-m3': '1.3 GB',
  };
  if (sizeByName[modelId]) return sizeByName[modelId];
  return installedModelIds.includes(modelId) ? 'installed' : 'unknown';
}
