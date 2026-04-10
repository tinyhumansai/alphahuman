/**
 * Imperative RPC wrapper for skill state — single source of truth.
 *
 * Replaces direct Redux access for skill state reads and writes.
 * All functions call the Rust core sidecar via JSON-RPC.
 */

import { callCoreRpc } from '../../services/coreRpcClient';

// Re-export types that consumers need
export interface SkillSnapshotRpc {
  skill_id: string;
  name: string;
  status: string;
  tools: Array<{ name: string; description: string; inputSchema?: unknown }>;
  error?: string | null;
  state: Record<string, unknown>;
  setup_complete: boolean;
  connection_status: string;
}

export interface AvailableSkillEntryRpc {
  id: string;
  name: string;
  version: string;
  description: string;
  runtime: string;
  entry: string;
  auto_start: boolean;
  platforms?: string[] | null;
  setup?: {
    required?: boolean;
    label?: string;
    oauth?: { provider: string; scopes: string[]; apiBaseUrl: string };
    auth?: { modes: Array<{
      type: string;
      label?: string;
      description?: string;
      provider?: string;
      scopes?: string[];
      apiBaseUrl?: string;
      fields?: Array<Record<string, unknown>>;
      textDescription?: string;
      textPlaceholder?: string;
    }> };
  } | null;
  ignore_in_production: boolean;
  download_url: string;
  manifest_url: string;
  checksum_sha256?: string | null;
  category: string;
  installed: boolean;
  installed_version?: string | null;
  update_available: boolean;
}

export interface InstalledSkillInfoRpc {
  id: string;
  name: string;
  version: string;
  description: string;
  runtime: string;
}

/**
 * Result returned by `oauth/complete` and `auth/complete` RPCs.
 *
 * The Rust host (see `handle_oauth_complete` / `handle_auth_complete` in
 * `openhuman/src/openhuman/skills/qjs_skill_instance/event_loop/rpc_handlers.rs`)
 * temp-injects credentials, then calls the skill's `start({oauth, auth, validate:true})`
 * lifecycle hook to validate them against the upstream API. Only on a
 * `{status:'complete'}` return are credentials persisted to disk; a
 * `{status:'error'}` return triggers a rollback. The frontend must inspect
 * this shape — silently treating any non-throwing RPC as success would leave
 * the user staring at a "connecting…" spinner that never finishes.
 */
export type SkillStartResult =
  | { status: 'complete'; message?: string }
  | { status: 'error'; errors: Array<{ field: string; message: string }> };

export interface OAuthCompleteParams {
  credentialId: string;
  provider: string;
  grantedScopes?: string[];
  accountLabel?: string;
  clientKeyShare?: string;
}

export interface AuthCompleteParams {
  mode: string;
  credentials: Record<string, unknown>;
}

// --- Read operations ---

export async function getSkillSnapshot(skillId: string): Promise<SkillSnapshotRpc> {
  return callCoreRpc<SkillSnapshotRpc>({
    method: 'openhuman.skills_status',
    params: { skill_id: skillId },
  });
}

export async function getAllSnapshots(): Promise<SkillSnapshotRpc[]> {
  return callCoreRpc<SkillSnapshotRpc[]>({
    method: 'openhuman.skills_get_all_snapshots',
  });
}

export async function listAvailable(): Promise<AvailableSkillEntryRpc[]> {
  return callCoreRpc<AvailableSkillEntryRpc[]>({
    method: 'openhuman.skills_list_available',
  });
}

export async function listInstalled(): Promise<InstalledSkillInfoRpc[]> {
  return callCoreRpc<InstalledSkillInfoRpc[]>({
    method: 'openhuman.skills_list_installed',
  });
}

export async function searchSkills(
  query: string,
  category?: string,
): Promise<AvailableSkillEntryRpc[]> {
  return callCoreRpc<AvailableSkillEntryRpc[]>({
    method: 'openhuman.skills_search',
    params: { query, category },
  });
}

// --- Write operations ---

export async function startSkill(skillId: string): Promise<SkillSnapshotRpc> {
  return callCoreRpc<SkillSnapshotRpc>({
    method: 'openhuman.skills_start',
    params: { skill_id: skillId },
  });
}

export async function stopSkill(skillId: string): Promise<void> {
  await callCoreRpc({ method: 'openhuman.skills_stop', params: { skill_id: skillId } });
}

export async function installSkill(skillId: string): Promise<void> {
  await callCoreRpc({
    method: 'openhuman.skills_install',
    params: { skill_id: skillId },
  });
}

export async function uninstallSkill(skillId: string): Promise<void> {
  await callCoreRpc({
    method: 'openhuman.skills_uninstall',
    params: { skill_id: skillId },
  });
}

export async function setSetupComplete(skillId: string, complete: boolean): Promise<void> {
  await callCoreRpc({
    method: 'openhuman.skills_set_setup_complete',
    params: { skill_id: skillId, complete },
  });
}

/**
 * Send `oauth/complete` to the running skill via the core RPC pass-through.
 * Returns the typed `{status, errors?}` result so callers can react to
 * validation failures (e.g. show field errors, leave setup_complete unset).
 *
 * Throws on transport-level failures (skill not running, RPC unreachable).
 */
export async function notifyOAuthCompleteRpc(
  skillId: string,
  params: OAuthCompleteParams,
): Promise<SkillStartResult> {
  const result = (await callCoreRpc({
    method: 'openhuman.skills_rpc',
    params: {
      skill_id: skillId,
      method: 'oauth/complete',
      params,
    },
  })) as SkillStartResult | null;
  // Treat null / missing-status as legacy success — older skill bundles that
  // predate the validate-then-persist contract just return undefined here,
  // and we don't want to false-fail them.
  if (!result || (result.status !== 'complete' && result.status !== 'error')) {
    return { status: 'complete' };
  }
  return result;
}

/**
 * Send `auth/complete` to the running skill via the core RPC pass-through.
 * Same `{status, errors?}` contract as {@link notifyOAuthCompleteRpc}.
 */
export async function notifyAuthCompleteRpc(
  skillId: string,
  params: AuthCompleteParams,
): Promise<SkillStartResult> {
  const result = (await callCoreRpc({
    method: 'openhuman.skills_rpc',
    params: {
      skill_id: skillId,
      method: 'auth/complete',
      params,
    },
  })) as SkillStartResult | null;
  if (!result || (result.status !== 'complete' && result.status !== 'error')) {
    return { status: 'complete' };
  }
  return result;
}

export async function revokeOAuth(skillId: string, integrationId: string): Promise<void> {
  await callCoreRpc({
    method: 'openhuman.skills_rpc',
    params: {
      skill_id: skillId,
      method: 'oauth/revoked',
      params: { integrationId },
    },
  });
}

/**
 * Host-side fallback: delete oauth_credential.json from the skill's data dir.
 * Used when the runtime is already stopped so oauth/revoked RPC can't reach it.
 */
export async function removePersistedOAuthCredential(skillId: string): Promise<void> {
  await callCoreRpc({
    method: 'openhuman.skills_data_write',
    params: { skill_id: skillId, filename: 'oauth_credential.json', content: '' },
  });
}

/** Revoke advanced auth credential via skill RPC. */
export async function revokeAuth(skillId: string, mode?: string): Promise<void> {
  await callCoreRpc({
    method: 'openhuman.skills_rpc',
    params: {
      skill_id: skillId,
      method: 'auth/revoked',
      params: { mode: mode ?? 'unknown' },
    },
  });
}

/**
 * Host-side fallback: delete auth_credential.json from the skill's data dir.
 * Used when the runtime is already stopped so auth/revoked RPC can't reach it.
 */
export async function removePersistedAuthCredential(skillId: string): Promise<void> {
  await callCoreRpc({
    method: 'openhuman.skills_data_write',
    params: { skill_id: skillId, filename: 'auth_credential.json', content: '' },
  });
}

/** Host-side fallback: clear client_key.json from the skill's data dir. */
export async function removePersistedClientKey(skillId: string): Promise<void> {
  await callCoreRpc({
    method: 'openhuman.skills_data_write',
    params: { skill_id: skillId, filename: 'client_key.json', content: '' },
  });
}

export async function disableSkill(skillId: string): Promise<void> {
  await callCoreRpc({
    method: 'openhuman.skills_disable',
    params: { skill_id: skillId },
  });
}

export async function fetchRegistryFresh(): Promise<void> {
  await callCoreRpc({
    method: 'openhuman.skills_registry_fetch',
    params: { force: true },
  });
}
