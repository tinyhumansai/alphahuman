/**
 * Send Notion metadata to the backend via the
 * `integration:metadata-sync` socket event so the server can merge it
 * into the user's Notion OAuth integration metadata.
 *
 * Mirrors the Gmail metadata sync pattern: we send the primary profile plus
 * additional Notion data (pages, summaries) when available.
 */
import { emitViaRustSocket } from '../../../utils/tauriSocket';

const INTEGRATION_METADATA_SYNC_EVENT = 'integration:metadata-sync';
const PROVIDER_NOTION = 'notion';

export interface NotionUserProfileLike {
  id: string;
  name?: string | null;
  email?: string | null;
  type?: string | null;
  avatar_url?: string | null;
}

export interface NotionPageSummaryLike {
  id: string;
  title: string;
  url: string | null;
  last_edited_time: string;
  content_text: string | null;
}

export interface NotionSummaryLike {
  id: number;
  pageId: string;
  url: string | null;
  summary: string;
  category: string | null;
  sentiment: string;
  topics: string[];
  sourceCreatedAt: string;
  sourceUpdatedAt: string;
}

/**
 * Shape of Notion data we care about for backend integration metadata sync.
 * This is populated from the Notion Redux slice in the runner.
 */
export interface NotionStateForSync {
  profile?: NotionUserProfileLike | null;
  pages?: NotionPageSummaryLike[] | null;
  summaries?: NotionSummaryLike[] | null;
}

/**
 * Emit `integration:metadata-sync` with Notion profile plus additional
 * Notion data (pages and summaries) so the backend can merge everything
 * into the user's Notion OAuth integration metadata.
 *
 * No-op when profile is missing or invalid.
 */
export function syncNotionMetadataToBackend(
  notionState: NotionStateForSync | null | undefined
): void {
  const profile = notionState?.profile;
  if (!profile || !profile.id) return;

  const metadata: Record<string, unknown> = {
    id: profile.id,
    name: profile.name ?? null,
    email: profile.email ?? null,
    type: profile.type ?? null,
    avatar_url: profile.avatar_url ?? null,
  };

  if (Array.isArray(notionState.pages) && notionState.pages.length > 0) {
    metadata.pages = notionState.pages;
    metadata.pages_total = notionState.pages.length;
  }

  if (Array.isArray(notionState.summaries) && notionState.summaries.length > 0) {
    metadata.summaries = notionState.summaries;
    metadata.summaries_total = notionState.summaries.length;
  }

  const payload = { requestId: crypto.randomUUID(), provider: PROVIDER_NOTION, metadata };

  void emitViaRustSocket(INTEGRATION_METADATA_SYNC_EVENT, payload);
}
