import {
  DEFAULT_EXTRACT_MODEL,
  DEFAULT_SUMMARISER_MODEL,
  type ModelDescriptor,
  type ModelRole,
  RECOMMENDED_MODEL_CATALOG,
  REQUIRED_EMBEDDER_MODEL,
} from '../../lib/intelligence/settingsApi';

interface ModelAssignmentProps {
  /** Names of models that are already installed on the user's machine. */
  installedModelIds: ReadonlyArray<string>;
  /** Currently chosen extract model. */
  extractModel: string;
  /** Currently chosen summariser model. */
  summariserModel: string;
  onChangeExtract: (id: string) => void;
  onChangeSummariser: (id: string) => void;
}

/**
 * Per-role assignment table — three rows: Extract, Summariser, Embedder.
 *
 * The embedder row is locked to `bge-m3` for v1 (the spec says we never
 * round-trip embeddings through the cloud). Extract/Summariser dropdowns
 * are populated from the recommended catalog filtered by `roles`, and
 * suffixed with the locally-installed set (so the user can see/select
 * anything they've already pulled even if it isn't on the curated list).
 */
export default function ModelAssignment({
  installedModelIds,
  extractModel,
  summariserModel,
  onChangeExtract,
  onChangeSummariser,
}: ModelAssignmentProps) {
  const extractOptions = optionsFor('extract', installedModelIds);
  const summariserOptions = optionsFor('summariser', installedModelIds);
  const embedderDescriptor = RECOMMENDED_MODEL_CATALOG.find(m => m.id === REQUIRED_EMBEDDER_MODEL);
  const embedderInstalled = installedModelIds.includes(REQUIRED_EMBEDDER_MODEL);

  return (
    <div className="border border-stone-200 rounded-2xl overflow-hidden">
      <Row
        label="Extract LLM"
        sublabel={describe(extractOptions.find(opt => opt.id === extractModel))}>
        <select
          value={extractModel}
          onChange={e => onChangeExtract(e.target.value)}
          className="w-full sm:w-64 px-3 py-1.5 text-sm bg-white border border-stone-200 rounded-lg text-stone-900 focus:outline-none focus:border-primary-500/50 transition-colors"
          aria-label="Extract LLM">
          {extractOptions.map(opt => (
            <option key={opt.id} value={opt.id}>
              {opt.label ?? opt.id}
            </option>
          ))}
        </select>
      </Row>

      <Row
        label="Summariser LLM"
        sublabel={describe(summariserOptions.find(opt => opt.id === summariserModel))}>
        <select
          value={summariserModel}
          onChange={e => onChangeSummariser(e.target.value)}
          className="w-full sm:w-64 px-3 py-1.5 text-sm bg-white border border-stone-200 rounded-lg text-stone-900 focus:outline-none focus:border-primary-500/50 transition-colors"
          aria-label="Summariser LLM">
          {summariserOptions.map(opt => (
            <option key={opt.id} value={opt.id}>
              {opt.label ?? opt.id}
            </option>
          ))}
        </select>
      </Row>

      <Row
        label="Embedder"
        sublabel={
          embedderDescriptor
            ? `${embedderDescriptor.size} · required · 1024-dim`
            : 'required · 1024-dim'
        }
        last>
        <div className="flex items-center gap-2 text-sm font-mono text-stone-700">
          <span>{REQUIRED_EMBEDDER_MODEL}</span>
          {embedderInstalled ? (
            <span className="inline-flex items-center gap-1 text-sage-600 text-xs">
              <svg
                className="w-3 h-3"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth={2.5}>
                <path strokeLinecap="round" strokeLinejoin="round" d="M5 13l4 4L19 7" />
              </svg>
              loaded
            </span>
          ) : (
            <span className="text-amber-700 text-xs">not downloaded</span>
          )}
        </div>
      </Row>
    </div>
  );
}

interface RowProps {
  label: string;
  sublabel: string;
  last?: boolean;
  children: React.ReactNode;
}

function Row({ label, sublabel, last, children }: RowProps) {
  return (
    <div
      className={`grid grid-cols-1 sm:grid-cols-[1fr_auto] gap-2 sm:gap-6 px-5 py-4 ${
        last ? '' : 'border-b border-stone-100'
      }`}>
      <div>
        <div className="text-sm font-semibold text-stone-900">{label}</div>
        <div className="font-mono text-[11px] text-stone-500 mt-0.5">{sublabel}</div>
      </div>
      <div className="flex items-center sm:justify-end">{children}</div>
    </div>
  );
}

function describe(model?: ModelDescriptor): string {
  if (!model) return '—';
  return `${model.size} · ${model.ramHint} · ${model.category}`;
}

/**
 * Build the dropdown options for a role. Catalog entries that match the role
 * always come first; locally-installed models that aren't in the catalog
 * (the user pulled them outside the OpenHuman flow) are appended so they're
 * still selectable.
 */
function optionsFor(role: ModelRole, installedModelIds: ReadonlyArray<string>): ModelDescriptor[] {
  const catalog = RECOMMENDED_MODEL_CATALOG.filter(m => m.roles.includes(role));
  const known = new Set(catalog.map(m => m.id));
  const extras = installedModelIds
    .filter(id => !known.has(id))
    .map<ModelDescriptor>(id => ({
      id,
      size: '—',
      approxBytes: 0,
      ramHint: '—',
      category: 'balanced',
      note: 'locally installed',
      roles: ['extract', 'summariser'],
    }));
  return [...catalog, ...extras];
}

// Re-export defaults for callers that want to seed initial state without
// chasing them through the API module.
export { DEFAULT_EXTRACT_MODEL, DEFAULT_SUMMARISER_MODEL, REQUIRED_EMBEDDER_MODEL };
