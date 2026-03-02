/**
 * Send Gmail profile (and optionally emails) to the backend via the
 * `integration:metadata-sync` socket event so the server can merge them
 * into the user's Google OAuth integration metadata.
 */
import { emitViaRustSocket } from '../../../utils/tauriSocket';

const INTEGRATION_METADATA_SYNC_EVENT = 'integration:metadata-sync';
const PROVIDER_GOOGLE = 'gmail';

/** Gmail profile shape from skill state (snake_case). */
export interface GmailProfileLike {
  email_address: string;
  messages_total: number;
  threads_total: number;
  history_id: string;
}

/**
 * Emit `integration:metadata-sync` with Gmail profile and emails so the
 * backend can merge them into the user's Google OAuth integration.
 * No-op when profile is missing or not in Tauri.
 */
export function syncGmailMetadataToBackend(gmailState: GmailProfileLike | undefined): void {
  if (!gmailState) return;

  const metadata: Record<string, unknown> = {
    email_address: gmailState.email_address,
    messages_total: gmailState.messages_total,
    threads_total: gmailState.threads_total,
    history_id: gmailState.history_id,
  };

  // if (Array.isArray(gmailState.emails) && gmailState.emails.length > 0) {
  //   metadata.emails = gmailState.emails as GmailEmailSummaryLike[];
  // }

  const payload = { requestId: crypto.randomUUID(), provider: PROVIDER_GOOGLE, metadata };

  void emitViaRustSocket(INTEGRATION_METADATA_SYNC_EVENT, payload);
}
