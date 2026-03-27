/**
 * Agent Tool Registry Service
 *
 * Unified tool discovery and execution for runtime-managed skills.
 * Tool names are expected in namespaced format: `skillId__toolName`.
 */
import { invoke } from '@tauri-apps/api/core';

import type { AgentToolExecution, AgentToolSchema, IAgentToolRegistry } from '../types/agent';

// ZeroClaw format types from Rust
interface ZeroClawToolSchema {
  type: string;
  function: { name: string; description: string; parameters: Record<string, unknown> };
}

interface ZeroClawToolResult {
  success: boolean;
  output: string;
  error?: string;
  execution_time?: number;
}

export class AgentToolRegistry implements IAgentToolRegistry {
  private static instance: AgentToolRegistry;
  private toolSchemas: AgentToolSchema[] = [];
  private lastLoadTime = 0;
  private readonly CACHE_TTL = 5 * 60 * 1000; // 5 minutes

  static getInstance(): AgentToolRegistry {
    if (!this.instance) {
      this.instance = new AgentToolRegistry();
    }
    return this.instance;
  }

  /**
   * Load tool schemas from unified systems (Telegram + skill system fallback)
   */
  async loadToolSchemas(forceReload = false): Promise<AgentToolSchema[]> {
    const now = Date.now();

    // Return cached tools if still fresh
    if (!forceReload && this.toolSchemas.length > 0 && now - this.lastLoadTime < this.CACHE_TTL) {
      return this.toolSchemas;
    }

    try {
      console.log('[tool-registry] Loading tool schemas from runtime');

      const allTools: AgentToolSchema[] = [];
      try {
        const skillTools = await invoke<ZeroClawToolSchema[]>('runtime_get_tool_schemas');
        const skillSchemas = skillTools.map(tool => ({
          type: 'function' as const,
          function: {
            name: tool.function.name,
            description: tool.function.description,
            parameters: tool.function.parameters as AgentToolSchema['function']['parameters'],
          },
        }));

        allTools.push(...skillSchemas);
        console.log(`[tool-registry] Loaded ${skillSchemas.length} tools from runtime`);
      } catch (error) {
        console.warn('[tool-registry] Failed to load tools from runtime:', error);
      }

      this.toolSchemas = allTools;
      this.lastLoadTime = now;

      console.log(`[tool-registry] Updated: ${this.toolSchemas.length} total tools`);

      return this.toolSchemas;
    } catch (error) {
      console.error('❌ Failed to load tool schemas:', error);
      throw new Error(`Failed to load tool schemas: ${error}`);
    }
  }

  /**
   * Execute a tool using unified systems (Telegram unified or skill system fallback)
   */
  async executeTool(
    skillId: string,
    toolName: string,
    toolArguments: string
  ): Promise<AgentToolExecution> {
    const startTime = Date.now();
    const executionId = `exec_${startTime}_${Math.random().toString(36).substr(2, 9)}`;

    const execution: AgentToolExecution = {
      id: executionId,
      toolName,
      skillId,
      arguments: toolArguments,
      status: 'running',
      startTime,
    };

    console.log(`[tool-registry] Execute ${toolName} (skillId=${skillId})`);

    try {
      const toolId = `${skillId}__${toolName}`;
      const result = await invoke<ZeroClawToolResult>('runtime_execute_tool', {
        toolId,
        args: toolArguments,
      });

      execution.endTime = Date.now();
      // Use execution time from Rust if available, otherwise calculate locally
      execution.executionTimeMs = result.execution_time || execution.endTime - execution.startTime;

      if (!result.success) {
        execution.status = 'error';
        execution.errorMessage = result.error || 'Unknown error occurred';
        execution.result = execution.errorMessage;

        console.log(`[tool-registry] Tool failed: ${toolName} (${execution.executionTimeMs}ms)`);
      } else {
        execution.status = 'success';
        execution.result = result.output;

        console.log(
          `[tool-registry] Tool completed: ${toolName} (${execution.executionTimeMs}ms)`
        );
      }

      return execution;
    } catch (error) {
      execution.endTime = Date.now();
      execution.executionTimeMs = execution.endTime - execution.startTime;
      execution.status = 'error';
      execution.errorMessage = error instanceof Error ? error.message : String(error);
      execution.result = execution.errorMessage;

      console.error(`[tool-registry] Tool execution error: ${toolName}`, error);

      return execution;
    }
  }

  /**
   * Get a specific tool by name
   */
  getToolByName(toolName: string): AgentToolSchema | undefined {
    return this.toolSchemas.find(tool => tool.function.name === toolName);
  }

  /**
   * Get all available tools
   */
  getAllTools(): AgentToolSchema[] {
    return [...this.toolSchemas];
  }

  /**
   * Get tools organized by skill
   */
  getToolsBySkill(): Record<string, AgentToolSchema[]> {
    const toolsBySkill: Record<string, AgentToolSchema[]> = {};

    for (const tool of this.toolSchemas) {
      // Extract skill ID from tool name (format: skillId__toolName)
      const skillId = this.extractSkillIdFromToolName(tool.function.name) || 'unknown';

      if (!toolsBySkill[skillId]) {
        toolsBySkill[skillId] = [];
      }
      toolsBySkill[skillId].push(tool);
    }

    return toolsBySkill;
  }

  /**
   * Get tool execution statistics
   */
  getToolStats(): { totalTools: number; skillCount: number; categories: Record<string, number> } {
    const categories: Record<string, number> = {};
    const skills = new Set<string>();

    for (const tool of this.toolSchemas) {
      const skillId = this.extractSkillIdFromToolName(tool.function.name) || 'unknown';
      skills.add(skillId);

      // Categorize by skill name
      const category = this.extractCategoryFromSkillId(skillId);
      categories[category] = (categories[category] || 0) + 1;
    }

    return { totalTools: this.toolSchemas.length, skillCount: skills.size, categories };
  }

  /**
   * Clear the tool registry cache
   */
  clearCache(): void {
    this.toolSchemas = [];
    this.lastLoadTime = 0;
    console.log('[tool-registry] Cache cleared');
  }

  // =============================================================================
  // Private Helper Methods
  // =============================================================================

  /**
   * Extract skill ID from tool name (format: skillId__toolName)
   */
  private extractSkillIdFromToolName(toolName: string): string | null {
    const separator = toolName.indexOf('__');
    if (separator === -1) {
      return null;
    }
    return toolName.substring(0, separator);
  }

  /**
   * Extract category name from skill ID for organization
   */
  private extractCategoryFromSkillId(skillId: string): string {
    // Common skill naming patterns
    if (skillId.includes('github') || skillId.includes('git')) return 'GitHub';
    if (skillId.includes('notion')) return 'Notion';
    if (skillId.includes('telegram') || skillId.includes('tg')) return 'Telegram';
    if (skillId.includes('email') || skillId.includes('gmail')) return 'Email';
    if (skillId.includes('calendar')) return 'Calendar';
    if (skillId.includes('slack')) return 'Slack';
    if (skillId.includes('discord')) return 'Discord';
    if (skillId.includes('twitter') || skillId.includes('x')) return 'Social';
    if (skillId.includes('file') || skillId.includes('fs')) return 'File System';
    if (skillId.includes('crypto') || skillId.includes('blockchain')) return 'Crypto';
    if (skillId.includes('ai') || skillId.includes('ml')) return 'AI/ML';

    return 'Other';
  }
}
