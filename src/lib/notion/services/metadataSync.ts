/**
 * Send Notion user metadata to the backend via the
 * `integration:metadata-sync` socket event so the server can merge it
 * into the user's Notion OAuth integration metadata.
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

/**
 * Emit `integration:metadata-sync` with Notion user profile so the
 * backend can merge it into the user's Notion OAuth integration.
 * No-op when profile is missing or invalid.
 */
export function syncNotionMetadataToBackend(
  profile: NotionUserProfileLike | null | undefined
): void {
  if (!profile || !profile.id) return;

  const metadata: Record<string, unknown> = {
    id: profile.id,
    name: profile.name ?? null,
    email: profile.email ?? null,
    type: profile.type ?? null,
    avatar_url: profile.avatar_url ?? null,
  };

  const payload = {
    requestId: crypto.randomUUID(),
    provider: PROVIDER_NOTION,
    metadata,
  };

  void emitViaRustSocket(INTEGRATION_METADATA_SYNC_EVENT, payload);
}

