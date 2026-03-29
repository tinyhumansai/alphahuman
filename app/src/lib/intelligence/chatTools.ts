/**
 * Chat tools initialization for Intelligence system.
 *
 * Provides functionality to gather all active skills' tools and send them
 * to the backend when a chat session is initialized, enabling AI to understand
 * available capabilities for task execution.
 */
import { isTauri as coreIsTauri } from '@tauri-apps/api/core';
import debug from 'debug';

import type { ConnectedTool } from '../../services/intelligenceApi';
import { socketService } from '../../services/socketService';
import { store } from '../../store';
import { transformMCPToConnectedTools } from '../../utils/intelligenceTransforms';
import { emitViaRustSocket } from '../../utils/tauriSocket';
import type { MCPTool } from '../mcp';
import { deriveConnectionStatus } from '../skills/hooks';

// Chat tools logger using debug package
const chatToolsLog = debug('chat:tools');
const chatToolsWarn = debug('chat:tools:warn');

// Enable logging in development
if (import.meta.env.DEV || import.meta.env.MODE === 'development') {
  debug.enable('chat:tools*');
}

/**
 * Check if running in Tauri environment
 */
function isTauri(): boolean {
  try {
    return coreIsTauri();
  } catch {
    return false;
  }
}

/**
 * Interface for chat initialization payload
 */
export interface ChatInitPayload {
  tools: MCPTool[];
  connectedTools: ConnectedTool[];
  sessionId?: string;
  threadId?: string;
  timestamp: number;
}

/**
 * Gather all tools from active skills and send them to the backend
 * when initializing a chat session.
 *
 * This function:
 * 1. Retrieves all skills from Redux store
 * 2. Filters for active/ready skills with completed setup
 * 3. Extracts and formats tool definitions in both MCP and Connected formats
 * 4. Emits 'chat:init' event via appropriate socket method
 *
 * @param sessionId - Optional chat session identifier
 * @param threadId - Optional thread identifier for chat session
 * @param useConnectedFormat - Whether to send tools in Connected format (default: true)
 */
export function initializeChatWithTools(
  sessionId?: string,
  threadId?: string,
  useConnectedFormat: boolean = true
): void {
  try {
    const state = store.getState();
    const skills = state.skills.skills;
    const skillStates = state.skills.skillStates;

    chatToolsLog('Initializing chat with tools', {
      sessionId,
      threadId,
      useConnectedFormat,
      totalSkills: Object.keys(skills).length,
    });

    const activeTools: MCPTool[] = [];

    // Process each skill to extract active tools
    for (const [skillId, skill] of Object.entries(skills)) {
      // Derive connection status using existing logic
      const connectionStatus = deriveConnectionStatus(
        skill.status,
        skill.setupComplete,
        skillStates[skillId]
      );

      // Only include tools from skills that are ready and properly connected
      const isSkillActive =
        (skill.status === 'ready' || skill.status === 'running') &&
        skill.setupComplete &&
        (connectionStatus === 'connected' || connectionStatus === 'connecting'); // Include connecting state for gradual initialization

      if (isSkillActive && skill.tools?.length) {
        chatToolsLog('Processing tools for active skill', {
          skillId,
          skillName: skill.manifest.name,
          toolCount: skill.tools.length,
          status: skill.status,
          connectionStatus,
        });

        // Transform skill tools to MCP format with skill prefix
        for (const tool of skill.tools) {
          activeTools.push({
            name: `${skillId}__${tool.name}`,
            description: `${skill.manifest.name}: ${tool.description}`,
            inputSchema: tool.inputSchema,
          });
        }
      } else {
        chatToolsLog('Skipping inactive skill', {
          skillId,
          skillName: skill.manifest.name,
          status: skill.status,
          setupComplete: skill.setupComplete,
          connectionStatus,
          toolCount: skill.tools?.length || 0,
        });
      }
    }

    // Create tools breakdown by skill for detailed logging
    const toolsBySkill: Record<string, { skillName: string; tools: string[] }> = {};
    activeTools.forEach(tool => {
      const [skillId] = tool.name.split('__');
      const skill = skills[skillId];
      if (skill) {
        if (!toolsBySkill[skillId]) {
          toolsBySkill[skillId] = { skillName: skill.manifest.name, tools: [] };
        }
        toolsBySkill[skillId].tools.push(tool.name.split('__')[1]);
      }
    });

    // Transform to connected tools format if requested
    const connectedTools = useConnectedFormat ? transformMCPToConnectedTools(activeTools) : [];

    // Prepare chat initialization payload
    const payload: ChatInitPayload = {
      tools: activeTools,
      connectedTools,
      sessionId,
      threadId,
      timestamp: Date.now(),
    };

    // Detailed logging of available tools
    chatToolsLog('🛠️  AVAILABLE TOOLS FOR CHAT INITIALIZATION', {
      sessionId,
      threadId,
      totalTools: activeTools.length,
      totalConnectedTools: connectedTools.length,
      totalSkills: Object.keys(toolsBySkill).length,
      useConnectedFormat,
    });

    // Log tools breakdown by skill
    Object.entries(toolsBySkill).forEach(([skillId, { skillName, tools }]) => {
      chatToolsLog(`📦 ${skillName} (${skillId}):`, { toolCount: tools.length, tools });
    });

    // Log complete tools list with descriptions
    if (activeTools.length > 0) {
      chatToolsLog(
        '📋 Complete Tools List:',
        activeTools.map(tool => ({
          name: tool.name,
          description: tool.description.slice(0, 80) + (tool.description.length > 80 ? '...' : ''),
        }))
      );
    } else {
      chatToolsLog('⚠️  No active tools available - chat will have no tool capabilities');
    }

    // Emit via appropriate socket method based on environment
    if (isTauri()) {
      chatToolsLog('Emitting chat:init via Rust socket');
      emitViaRustSocket('chat:init', payload);
    } else {
      chatToolsLog('Emitting chat:init via web socket');
      if (socketService.isConnected()) {
        socketService.emit('chat:init', payload);
      } else {
        chatToolsWarn('Socket not connected - chat initialization may be delayed');
        // Could potentially queue the initialization for when socket connects
      }
    }

    chatToolsLog('Chat tools initialization completed', {
      sessionId,
      threadId,
      toolCount: activeTools.length,
      connectedToolCount: connectedTools.length,
      environment: isTauri() ? 'tauri' : 'web',
    });
  } catch (error) {
    chatToolsWarn('Failed to initialize chat with tools', {
      sessionId,
      error: error instanceof Error ? error.message : String(error),
    });

    // Don't throw - allow chat to continue without tools if needed
    // The backend should handle empty tools gracefully
  }
}

/**
 * Initialize Intelligence chat session with both MCP and Connected tools formats
 * @param sessionId - Chat session identifier
 * @param threadId - Thread identifier for chat session
 * @param connectedTools - Pre-transformed connected tools (optional)
 */
export function initializeIntelligenceChatSession(
  sessionId: string,
  threadId: string,
  connectedTools?: ConnectedTool[]
): void {
  try {
    const mcpTools = getCurrentActiveTools();
    const finalConnectedTools = connectedTools || transformMCPToConnectedTools(mcpTools);

    const payload = {
      tools: finalConnectedTools, // Intelligence system expects Connected format
      sessionId,
      threadId,
      timestamp: Date.now(),
    };

    chatToolsLog('Intelligence: Initializing chat session', {
      sessionId,
      threadId,
      toolCount: finalConnectedTools.length,
    });

    // Emit via appropriate socket method based on environment
    if (isTauri()) {
      chatToolsLog('Intelligence: Emitting chat:init via Rust socket');
      emitViaRustSocket('chat:init', payload);
    } else {
      chatToolsLog('Intelligence: Emitting chat:init via web socket');
      if (socketService.isConnected()) {
        socketService.emit('chat:init', payload);
      } else {
        chatToolsWarn('Intelligence: Socket not connected - chat initialization may be delayed');
      }
    }
  } catch (error) {
    chatToolsWarn('Intelligence: Failed to initialize chat session', {
      sessionId,
      threadId,
      error: error instanceof Error ? error.message : String(error),
    });
  }
}

/**
 * Get current active tools without emitting (for debugging/display purposes)
 */
export function getCurrentActiveTools(): MCPTool[] {
  try {
    const state = store.getState();
    const skills = state.skills.skills;
    const skillStates = state.skills.skillStates;
    const activeTools: MCPTool[] = [];

    for (const [skillId, skill] of Object.entries(skills)) {
      const connectionStatus = deriveConnectionStatus(
        skill.status,
        skill.setupComplete,
        skillStates[skillId]
      );

      const isSkillActive =
        (skill.status === 'ready' || skill.status === 'running') &&
        skill.setupComplete &&
        (connectionStatus === 'connected' || connectionStatus === 'connecting');

      if (isSkillActive && skill.tools?.length) {
        for (const tool of skill.tools) {
          activeTools.push({
            name: `${skillId}__${tool.name}`,
            description: `${skill.manifest.name}: ${tool.description}`,
            inputSchema: tool.inputSchema,
          });
        }
      }
    }

    return activeTools;
  } catch (error) {
    chatToolsWarn('Failed to get current active tools', {
      error: error instanceof Error ? error.message : String(error),
    });
    return [];
  }
}

/**
 * Get current active tools in Connected format for Intelligence system
 */
export function getCurrentConnectedTools(): ConnectedTool[] {
  try {
    const mcpTools = getCurrentActiveTools();
    return transformMCPToConnectedTools(mcpTools);
  } catch (error) {
    chatToolsWarn('Failed to get current connected tools', {
      error: error instanceof Error ? error.message : String(error),
    });
    return [];
  }
}
