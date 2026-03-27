/**
 * Send Notion metadata to backend integration APIs.
 *
 * Socket-based metadata sync is currently disabled.
 */

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
  void notionState;
}
