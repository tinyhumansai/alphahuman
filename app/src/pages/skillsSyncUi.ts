import type { SkillHostConnectionState } from '../lib/skills/types';

export interface SkillSyncUiState {
  isSyncing: boolean;
  progressPercent: number | null;
  progressMessage: string | null;
  metricsText: string | null;
}

type SkillStateRecord = SkillHostConnectionState & Record<string, unknown>;
export interface SkillSyncStatsLike {
  syncCount?: number;
  lastSyncAtMs?: number | null;
  localDataBytes?: number | null;
  localFileCount?: number | null;
}

function readNumber(value: unknown): number | null {
  if (typeof value === 'number' && Number.isFinite(value)) return value;
  if (typeof value === 'string' && value.trim() !== '') {
    const parsed = Number(value);
    return Number.isFinite(parsed) ? parsed : null;
  }
  return null;
}

function readBoolean(value: unknown): boolean | null {
  return typeof value === 'boolean' ? value : null;
}

function clampPercent(value: number): number {
  if (value < 0) return 0;
  if (value > 100) return 100;
  return value;
}

function formatNumber(n: number): string {
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
  if (n >= 1_000) return `${(n / 1_000).toFixed(1)}K`;
  return String(n);
}

function formatBytes(bytes: number): string {
  if (!Number.isFinite(bytes) || bytes < 0) return '0 B';
  if (bytes < 1024) return `${Math.round(bytes)} B`;
  const units = ['KB', 'MB', 'GB', 'TB'];
  let value = bytes / 1024;
  let unitIndex = 0;
  while (value >= 1024 && unitIndex < units.length - 1) {
    value /= 1024;
    unitIndex += 1;
  }
  return `${value.toFixed(value >= 10 ? 0 : 1)} ${units[unitIndex]}`;
}

function formatRelativeTime(ms: number): string {
  const delta = Date.now() - ms;
  if (delta < 60_000) return 'just now';
  if (delta < 3_600_000) return `${Math.max(1, Math.floor(delta / 60_000))}m ago`;
  if (delta < 86_400_000) return `${Math.max(1, Math.floor(delta / 3_600_000))}h ago`;
  return `${Math.max(1, Math.floor(delta / 86_400_000))}d ago`;
}

function buildMetricsText(state: SkillStateRecord): string | null {
  const values = {
    newEmailsCount: readNumber(state.newEmailsCount),
    totalEmails: readNumber(state.totalEmails),
    totalDocuments: readNumber(state.totalDocuments),
    totalPages: readNumber(state.totalPages),
    pagesWithSummary: readNumber(state.pagesWithSummary),
    summariesPending: readNumber(state.summariesPending),
    totalFiles: readNumber(state.totalFiles),
    itemsDone: readNumber(state.itemsDone),
    itemsTotal: readNumber(state.itemsTotal),
  };

  const parts: string[] = [];
  if (values.newEmailsCount != null)
    parts.push(`${formatNumber(values.newEmailsCount)} new emails`);
  if (values.totalEmails != null) parts.push(`${formatNumber(values.totalEmails)} emails`);
  if (values.totalDocuments != null) parts.push(`${formatNumber(values.totalDocuments)} docs`);
  if (values.totalPages != null) parts.push(`${formatNumber(values.totalPages)} pages`);
  if (values.pagesWithSummary != null)
    parts.push(`${formatNumber(values.pagesWithSummary)} pages summarized`);
  if (values.summariesPending != null)
    parts.push(`${formatNumber(values.summariesPending)} summaries pending`);
  if (values.totalFiles != null) parts.push(`${formatNumber(values.totalFiles)} files`);
  if (values.itemsDone != null && values.itemsTotal != null && values.itemsTotal > 0) {
    parts.push(`${formatNumber(values.itemsDone)}/${formatNumber(values.itemsTotal)} items`);
  }

  if (parts.length === 0) return null;
  return parts.slice(0, 3).join(' · ');
}

function defaultProgressMessage(skillId: string): string {
  if (skillId === 'gmail') return 'Syncing emails...';
  if (skillId === 'google-drive') return 'Syncing documents...';
  if (skillId === 'notion') return 'Syncing Notion documents...';
  return 'Syncing...';
}

export function deriveSkillSyncUiState(
  skillId: string,
  skillState: SkillStateRecord | undefined
): SkillSyncUiState {
  if (!skillState) {
    return { isSyncing: false, progressPercent: null, progressMessage: null, metricsText: null };
  }

  const isSyncing = readBoolean(skillState.syncInProgress) === true;

  const explicitProgress =
    readNumber(skillState.syncProgress) ??
    readNumber(skillState.progressPercent) ??
    readNumber(skillState.progress);

  const itemDone = readNumber(skillState.itemsDone);
  const itemTotal = readNumber(skillState.itemsTotal);
  const ratioProgress =
    explicitProgress == null && itemDone != null && itemTotal != null && itemTotal > 0
      ? (itemDone / itemTotal) * 100
      : null;

  const progressPercent =
    explicitProgress != null
      ? clampPercent(explicitProgress)
      : ratioProgress != null
        ? clampPercent(ratioProgress)
        : null;

  const progressMessageRaw =
    typeof skillState.syncProgressMessage === 'string' ? skillState.syncProgressMessage.trim() : '';

  return {
    isSyncing,
    progressPercent: isSyncing ? progressPercent : null,
    progressMessage: isSyncing ? progressMessageRaw || defaultProgressMessage(skillId) : null,
    metricsText: isSyncing ? buildMetricsText(skillState) : null,
  };
}

export function deriveSkillSyncSummaryText(
  skillState: SkillStateRecord | undefined,
  syncStats: SkillSyncStatsLike | undefined
): string | null {
  const parts: string[] = [];

  const syncCount = readNumber(syncStats?.syncCount);
  if (syncCount != null && syncCount > 0) {
    parts.push(`${formatNumber(syncCount)} sync${syncCount === 1 ? '' : 's'}`);
  }

  const localDataBytes = readNumber(syncStats?.localDataBytes);
  if (localDataBytes != null && localDataBytes > 0) {
    parts.push(`${formatBytes(localDataBytes)} local`);
  }

  const localFileCount = readNumber(syncStats?.localFileCount);
  if (localFileCount != null && localFileCount > 0) {
    parts.push(`${formatNumber(localFileCount)} files`);
  }

  const lastSyncAtMs = readNumber(syncStats?.lastSyncAtMs);
  if (lastSyncAtMs != null && lastSyncAtMs > 0) {
    parts.push(`last ${formatRelativeTime(lastSyncAtMs)}`);
  } else {
    const lastSyncFromSkill =
      readNumber(skillState?.lastSyncTime) ??
      readNumber(skillState?.last_sync) ??
      readNumber(skillState?.last_sync_time);
    if (lastSyncFromSkill != null && lastSyncFromSkill > 0) {
      parts.push(`last ${formatRelativeTime(lastSyncFromSkill)}`);
    }
  }

  if (parts.length === 0) return null;
  return parts.slice(0, 3).join(' · ');
}
