import type { Message } from '../providers/interface';
import type { ToolsConfig } from './types';

export interface ToolsInjectionOptions {
  mode: 'prepend' | 'context-block' | 'invisible';
  includeMetadata?: boolean;
  maxTools?: number;
  format?: 'list' | 'categories' | 'compact';
}

/**
 * Inject TOOLS context into user message content.
 * Creates seamless tools availability injection without modifying original structure.
 */
export function injectToolsIntoMessage(
  message: Message,
  toolsConfig: ToolsConfig,
  options: ToolsInjectionOptions = { mode: 'context-block' }
): Message {
  if (message.role !== 'user') {
    return message; // Only inject into user messages
  }

  const toolsContext = buildToolsContext(toolsConfig, options);

  switch (options.mode) {
    case 'prepend':
      return {
        ...message,
        content: [
          { type: 'text', text: toolsContext },
          ...message.content
        ]
      };

    case 'context-block':
      return {
        ...message,
        content: [
          { type: 'text', text: `[TOOLS_CONTEXT]\n${toolsContext}\n[/TOOLS_CONTEXT]\n\nUser message:` },
          ...message.content
        ]
      };

    case 'invisible':
      // Add as hidden metadata that AI can access
      return {
        ...message,
        content: message.content.map((block, index) => {
          if (index === 0 && block.type === 'text') {
            return {
              ...block,
              text: `<!--TOOLS_CONTEXT:${btoa(toolsContext)}-->${block.text}`
            };
          }
          return block;
        })
      };

    default:
      return message;
  }
}

/**
 * Build compact TOOLS context string optimized for token efficiency
 */
function buildToolsContext(toolsConfig: ToolsConfig, options: ToolsInjectionOptions): string {
  const parts: string[] = [];

  // Compact format - statistics and key info
  parts.push(`${toolsConfig.statistics.totalTools} tools across ${toolsConfig.statistics.activeSkills} skills`);

  // Top categories by tool count
  const topCategories = Object.entries(toolsConfig.statistics.toolsByCategory)
    .sort(([,a], [,b]) => b - a)
    .slice(0, 4)
    .map(([cat, count]) => `${toolsConfig.categories[cat]?.name || cat} (${count})`)
    .join(', ');

  if (topCategories) {
    parts.push(`Categories: ${topCategories}`);
  }

  // Key skills (limit to avoid token bloat)
  const keySkills = Object.keys(toolsConfig.skillGroups).slice(0, 6);
  if (keySkills.length > 0) {
    parts.push(`Key skills: ${keySkills.join(', ')}`);
  }

  // Add specific format handling
  if (options.format === 'list' && options.maxTools && options.maxTools > 0) {
    const topTools = toolsConfig.tools
      .slice(0, Math.min(options.maxTools, 10))
      .map(tool => `${tool.name} (${tool.skillId})`)
      .join(', ');
    if (topTools) {
      parts.push(`Available: ${topTools}`);
    }
  }

  if (options.includeMetadata) {
    parts.push(`Updated: ${new Date(toolsConfig.loadedAt).toISOString()}`);
  }

  return parts.join('\n');
}

/**
 * Remove TOOLS context from message (for display purposes)
 */
export function stripToolsFromMessage(message: Message): Message {
  if (message.role !== 'user') {
    return message;
  }

  return {
    ...message,
    content: message.content.map(block => {
      if (block.type === 'text') {
        // Remove context blocks
        let text = block.text.replace(/\[TOOLS_CONTEXT\][\s\S]*?\[\/TOOLS_CONTEXT\]\s*User message:\s*/g, '');
        // Remove invisible context
        text = text.replace(/<!--TOOLS_CONTEXT:[^>]+-->/g, '');
        return { ...block, text };
      }
      return block;
    })
  };
}