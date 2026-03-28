/**
 * Tool Bridge — converts skill tools to AI tool registry format.
 *
 * Namespaces tool names as `<skillId>__<toolName>` to avoid collisions.
 * Each tool's execute function delegates to skillManager.callTool().
 */

import { skillManager } from "./manager";
import type { SkillToolDefinition } from "./types";

export interface BridgedTool {
  name: string;
  description: string;
  parameters: Record<string, unknown>;
  execute: (args: Record<string, unknown>) => Promise<string>;
  skillId: string;
  originalName: string;
}

/**
 * Convert a skill's tools into bridged tools with namespaced names.
 */
export function bridgeSkillTools(
  skillId: string,
  tools: SkillToolDefinition[],
): BridgedTool[] {
  return tools.map((tool) => ({
    name: `${skillId}__${tool.name}`,
    description: tool.description,
    parameters: tool.inputSchema,
    skillId,
    originalName: tool.name,
    execute: async (args: Record<string, unknown>): Promise<string> => {
      const result = await skillManager.callTool(skillId, tool.name, args);
      if (result.isError) {
        throw new Error(
          result.content.map((c) => c.text).join("\n") || "Tool execution failed",
        );
      }
      return result.content.map((c) => c.text).join("\n");
    },
  }));
}

/**
 * Extract the skill ID and original tool name from a namespaced name.
 */
export function parseToolName(namespacedName: string): {
  skillId: string;
  toolName: string;
} | null {
  const idx = namespacedName.indexOf("__");
  if (idx === -1) return null;
  return {
    skillId: namespacedName.substring(0, idx),
    toolName: namespacedName.substring(idx + 2),
  };
}
