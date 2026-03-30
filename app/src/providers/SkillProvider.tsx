import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { useEffect } from 'react';

import { skillManager } from '../lib/skills/manager';
import type { SkillManifest, SkillStatus } from '../lib/skills/types';
import { store } from '../store';
import { useAppDispatch, useAppSelector } from '../store/hooks';
import { setSkillError, setSkillState, setSkillStatus } from '../store/skillsSlice';
import { isTauri, runtimeDiscoverSkills } from '../utils/tauriCommands';

type RuntimeSkillStateChangedEvent = {
  skillId?: string;
  skill_id?: string;
  state?: Record<string, unknown>;
};

type RuntimeSkillStatusChangedEvent = {
  skillId?: string;
  skill_id?: string;
  status?: string;
  error?: string | null;
};

const VALID_STATUSES: ReadonlySet<SkillStatus> = new Set([
  'installed',
  'starting',
  'running',
  'setup_required',
  'setup_in_progress',
  'ready',
  'error',
  'stopping',
]);

function normalizeStatus(raw: unknown): SkillStatus | null {
  switch (raw) {
    case 'installed':
    case 'starting':
    case 'running':
    case 'setup_required':
    case 'setup_in_progress':
    case 'ready':
    case 'error':
    case 'stopping':
      return raw;
    // Runtime-side names
    case 'pending':
      return 'installed';
    case 'initializing':
      return 'starting';
    case 'stopped':
      return 'installed';
    default:
      return null;
  }
}

const SkillProvider = ({ children }: { children: React.ReactNode }) => {
  const dispatch = useAppDispatch();
  const token = useAppSelector(state => state.auth.token);

  useEffect(() => {
    if (!isTauri()) return;

    let mounted = true;

    const discoverSkills = async () => {
      if (!token) return;

      try {
        const manifests = await runtimeDiscoverSkills();
        if (!mounted) return;

        for (const manifest of manifests) {
          if (manifest.id.includes('_')) {
            continue;
          }

          skillManager.registerSkill(manifest as SkillManifest);
        }

        const startPromises = manifests
          .filter(manifest => !manifest.id.includes('_'))
          .filter(manifest => store.getState().skills.skills[manifest.id]?.setupComplete === true)
          .map(async manifest => {
            try {
              await skillManager.startSkill(manifest as SkillManifest);
            } catch (err) {
              console.warn(`[SkillProvider] Failed to auto-start ${manifest.id}:`, err);
            }
          });

        await Promise.all(startPromises);
      } catch (err) {
        console.warn('[SkillProvider] Failed to discover skills:', err);
      }
    };

    void discoverSkills();

    return () => {
      mounted = false;
    };
  }, [token]);

  useEffect(() => {
    if (!isTauri()) return;

    let disposed = false;
    let stateUnlisten: UnlistenFn | null = null;
    let statusUnlisten: UnlistenFn | null = null;

    const bind = async () => {
      try {
        const handleStateChanged = (event: { payload: RuntimeSkillStateChangedEvent }) => {
          if (disposed) return;
          const skillId = event.payload?.skillId ?? event.payload?.skill_id;
          const state = event.payload?.state;
          if (!skillId || !state || typeof state !== 'object') return;
          dispatch(setSkillState({ skillId, state }));
        };

        stateUnlisten = await listen<RuntimeSkillStateChangedEvent>(
          'skill-state-changed',
          handleStateChanged
        );
        // Some runtime emitters use the namespaced event.
        const stateUnlistenNamespaced = await listen<RuntimeSkillStateChangedEvent>(
          'runtime:skill-state-changed',
          handleStateChanged
        );

        statusUnlisten = await listen<RuntimeSkillStatusChangedEvent>(
          'runtime:skill-status-changed',
          event => {
            if (disposed) return;
            const skillId = event.payload?.skillId ?? event.payload?.skill_id;
            const status = normalizeStatus(event.payload?.status);
            if (!skillId || !status || !VALID_STATUSES.has(status)) return;

            dispatch(setSkillStatus({ skillId, status }));
            if (status === 'error' && event.payload?.error) {
              dispatch(setSkillError({ skillId, error: event.payload.error }));
            }
          }
        );

        const originalStatusUnlisten = statusUnlisten;
        statusUnlisten = () => {
          originalStatusUnlisten?.();
          stateUnlistenNamespaced?.();
        };
      } catch (err) {
        console.warn('[SkillProvider] Failed to attach runtime event listeners:', err);
      }
    };

    void bind();

    return () => {
      disposed = true;
      if (stateUnlisten) stateUnlisten();
      if (statusUnlisten) statusUnlisten();
    };
  }, [dispatch]);

  return <>{children}</>;
};

export default SkillProvider;
