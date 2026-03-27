/**
 * Send Gmail profile metadata to backend integration APIs.
 *
 * Socket-based metadata sync is currently disabled.
 */

/** Gmail profile shape from skill state (snake_case). */
interface GmailProfileLike {
  email_address: string;
  messages_total: number;
  threads_total: number;
  history_id: string;
}

/** Single email summary from skill state. */
interface GmailEmailSummaryLike {
  id: string;
  threadId: string;
  snippet?: string;
  subject?: string;
  from?: string;
  date?: string;
}

/** Gmail skill state slice we care about for metadata sync. */
export interface GmailStateForSync {
  profile?: GmailProfileLike | null;
  emails?: GmailEmailSummaryLike[] | null;
}

/**
 * Emit `integration:metadata-sync` with Gmail profile and emails so the
 * backend can merge them into the user's Google OAuth integration.
 * No-op when profile is missing or not in Tauri.
 */
export function syncGmailMetadataToBackend(gmailState: GmailStateForSync | undefined): void {
  void gmailState;
}
