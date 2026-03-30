/**
 * Sync tool/skill state to the backend via `tool:sync` socket event.
 *
 * Called whenever skill connection state changes or the socket reconnects,
 * so the backend always has an up-to-date picture of connected tools.
 */

import { socketService } from '../../services/socketService';
import { store } from '../../store';
import { deriveConnectionStatus } from './hooks';
import type { SkillConnectionStatus } from './types';

interface ToolSyncEntry {
  skillId: string;
  name: string;
  status: SkillConnectionStatus;
  tools: string[];
}

/**
 * Read all skills from Redux, derive their connection status,
 * and emit a `tool:sync` event with the full list.
 */
export function syncToolsToBackend(): void {
  const state = store.getState();
  const skills = state.skills.skills;
  const skillStates = state.skills.skillStates;

  const tools: ToolSyncEntry[] = [];

  for (const [skillId, skill] of Object.entries(skills)) {
    const connectionStatus = deriveConnectionStatus(
      skill.status,
      skill.setupComplete,
      skillStates[skillId],
    );

    tools.push({
      skillId,
      name: skill.manifest.name,
      status: connectionStatus,
      tools: (skill.tools ?? []).map(t => t.name),
    });
  }

  const payload = { tools };

  socketService.emit('tool:sync', payload);
}
