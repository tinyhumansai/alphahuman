import { createSlice, type PayloadAction } from '@reduxjs/toolkit';

import type {
  OAuthCredential,
  SkillManifest,
  SkillState,
  SkillStatus,
  SkillToolDefinition,
} from '../lib/skills/types';

export interface SkillSyncStats {
  syncCount: number;
  lastSyncAtMs: number | null;
  lastSyncStartedAtMs: number | null;
  lastSyncDurationMs: number | null;
  localDataBytes: number | null;
  localFileCount: number | null;
  updatedAtMs: number;
}

interface SkillsState {
  /** Skill states keyed by skill ID */
  skills: Record<string, SkillState>;
  /** Arbitrary per-skill state for reverse RPC state/get and state/set */
  skillStates: Record<string, Record<string, unknown>>;
  /** Persistent per-skill sync metrics for UI reporting */
  syncStatsBySkill: Record<string, SkillSyncStats>;
}

const initialState: SkillsState = { skills: {}, skillStates: {}, syncStatsBySkill: {} };

const skillsSlice = createSlice({
  name: 'skills',
  initialState,
  reducers: {
    addSkill(state, action: PayloadAction<{ manifest: SkillManifest }>) {
      const { manifest } = action.payload;
      // Preserve existing setupComplete if re-adding
      const existing = state.skills[manifest.id];
      state.skills[manifest.id] = {
        manifest,
        status: 'installed',
        setupComplete: existing?.setupComplete ?? false,
        tools: existing?.tools ?? [],
      };
    },

    setSkillStatus(state, action: PayloadAction<{ skillId: string; status: SkillStatus }>) {
      const { skillId, status } = action.payload;
      if (state.skills[skillId]) {
        state.skills[skillId].status = status;
        if (status !== 'error') {
          delete state.skills[skillId].error;
        }
      }
    },

    setSkillError(state, action: PayloadAction<{ skillId: string; error: string }>) {
      const { skillId, error } = action.payload;
      if (state.skills[skillId]) {
        state.skills[skillId].status = 'error';
        state.skills[skillId].error = error;
      }
    },

    setSkillSetupComplete(state, action: PayloadAction<{ skillId: string; complete: boolean }>) {
      const { skillId, complete } = action.payload;
      if (state.skills[skillId]) {
        state.skills[skillId].setupComplete = complete;
      }
    },

    setSkillOAuthCredential(
      state,
      action: PayloadAction<{ skillId: string; credential: OAuthCredential | undefined }>
    ) {
      const { skillId, credential } = action.payload;
      if (state.skills[skillId]) {
        state.skills[skillId].oauthCredential = credential;
      }
    },

    setSkillTools(state, action: PayloadAction<{ skillId: string; tools: SkillToolDefinition[] }>) {
      const { skillId, tools } = action.payload;
      if (state.skills[skillId]) {
        state.skills[skillId].tools = tools;
      }
    },

    setSkillState(
      state,
      action: PayloadAction<{ skillId: string; state: Record<string, unknown> }>
    ) {
      const { skillId, state: incomingState } = action.payload;
      const existing = state.skillStates[skillId] as Record<string, unknown> | undefined;
      // Preserve frontend-only keys (e.g. oauthTokens from deep link) that the skill never sends
      const preserved =
        existing && typeof existing.oauthTokens === 'object' && existing.oauthTokens !== null
          ? { oauthTokens: existing.oauthTokens }
          : {};
      state.skillStates[skillId] = { ...incomingState, ...preserved };
    },

    removeSkill(state, action: PayloadAction<string>) {
      delete state.skills[action.payload];
      delete state.skillStates[action.payload];
      delete state.syncStatsBySkill[action.payload];
    },

    upsertSkillSyncStats(
      state,
      action: PayloadAction<{
        skillId: string;
        patch: Partial<Omit<SkillSyncStats, 'syncCount'>> & { syncCountDelta?: number };
      }>
    ) {
      const { skillId, patch } = action.payload;
      const { syncCountDelta = 0, ...restPatch } = patch;
      const existing = state.syncStatsBySkill[skillId] ?? {
        syncCount: 0,
        lastSyncAtMs: null,
        lastSyncStartedAtMs: null,
        lastSyncDurationMs: null,
        localDataBytes: null,
        localFileCount: null,
        updatedAtMs: Date.now(),
      };

      const nextSyncCount = Math.max(0, existing.syncCount + syncCountDelta);

      state.syncStatsBySkill[skillId] = {
        ...existing,
        ...restPatch,
        syncCount: nextSyncCount,
        updatedAtMs: Date.now(),
      };
    },

    resetSkillsState() {
      return initialState;
    },
  },
});

export const {
  addSkill,
  setSkillStatus,
  setSkillError,
  setSkillSetupComplete,
  setSkillOAuthCredential,
  setSkillTools,
  setSkillState,
  removeSkill,
  upsertSkillSyncStats,
  resetSkillsState,
} = skillsSlice.actions;

export default skillsSlice.reducer;
