import { loadAIConfig } from './loader';
import { injectSoulIntoMessage } from './soul/injector';
import { injectToolsIntoMessage } from './tools/injector';
import type { Message } from './providers/interface';
import type { AIConfig } from './types';

export interface UnifiedInjectionOptions {
  mode: 'prepend' | 'context-block' | 'invisible';
  includeMetadata?: boolean;
  soul?: {
    enabled?: boolean;
  };
  tools?: {
    enabled?: boolean;
    maxTools?: number;
    format?: 'list' | 'categories' | 'compact';
  };
}

/**
 * Inject both SOUL and TOOLS contexts into user message.
 * Automatically loads AI configuration if not provided.
 */
export async function injectAll(
  message: Message,
  configOrOptions?: AIConfig | UnifiedInjectionOptions,
  optionsWhenConfigProvided?: UnifiedInjectionOptions
): Promise<Message> {
  // Handle overloaded parameters
  let config: AIConfig;
  let options: UnifiedInjectionOptions;

  if (configOrOptions && 'soul' in configOrOptions && 'tools' in configOrOptions && 'metadata' in configOrOptions) {
    // First param is AIConfig
    config = configOrOptions;
    options = optionsWhenConfigProvided || { mode: 'context-block' };
  } else {
    // First param is options, need to load config
    config = await loadAIConfig();
    options = (configOrOptions as UnifiedInjectionOptions) || { mode: 'context-block' };
  }

  // Default options
  const finalOptions: UnifiedInjectionOptions = {
    includeMetadata: false,
    soul: { enabled: true },
    tools: { enabled: true, maxTools: 20, format: 'compact' },
    ...options,
    mode: options.mode || 'context-block'
  };

  let injectedMessage = message;

  // Inject SOUL first (if enabled)
  if (finalOptions.soul?.enabled) {
    try {
      injectedMessage = injectSoulIntoMessage(injectedMessage, config.soul, {
        mode: finalOptions.mode,
        includeMetadata: finalOptions.includeMetadata
      });
    } catch (error) {
      console.warn('⚠️ SOUL injection failed, continuing with TOOLS only:', error);
    }
  }

  // Then inject TOOLS (if enabled)
  if (finalOptions.tools?.enabled) {
    try {
      injectedMessage = injectToolsIntoMessage(injectedMessage, config.tools, {
        mode: finalOptions.mode,
        includeMetadata: finalOptions.includeMetadata,
        maxTools: finalOptions.tools.maxTools,
        format: finalOptions.tools.format
      });
    } catch (error) {
      console.warn('⚠️ TOOLS injection failed, continuing without tools context:', error);
    }
  }

  return injectedMessage;
}

/**
 * Check if message has AI context injected
 */
export function hasInjectedContext(message: Message): {
  hasSoul: boolean;
  hasTools: boolean;
  hasAny: boolean;
} {
  const text = message.content
    .filter(block => block.type === 'text')
    .map(block => block.text)
    .join(' ');

  const hasSoul = text.includes('[PERSONA_CONTEXT]') || text.includes('<!--SOUL_CONTEXT:');
  const hasTools = text.includes('[TOOLS_CONTEXT]') || text.includes('<!--TOOLS_CONTEXT:');

  return {
    hasSoul,
    hasTools,
    hasAny: hasSoul || hasTools
  };
}

/**
 * Extract combined context information for debugging
 */
export function extractContextInfo(message: Message): {
  soulContextLength: number;
  toolsContextLength: number;
  totalInjectedLength: number;
  originalLength: number;
} {
  const text = message.content
    .filter(block => block.type === 'text')
    .map(block => block.text)
    .join(' ');

  const soulMatch = text.match(/\[PERSONA_CONTEXT\]([\s\S]*?)\[\/PERSONA_CONTEXT\]/);
  const toolsMatch = text.match(/\[TOOLS_CONTEXT\]([\s\S]*?)\[\/TOOLS_CONTEXT\]/);

  const soulContextLength = soulMatch ? soulMatch[1].length : 0;
  const toolsContextLength = toolsMatch ? toolsMatch[1].length : 0;

  // Calculate original message length (without injected context)
  let originalText = text;
  originalText = originalText.replace(/\[PERSONA_CONTEXT\][\s\S]*?\[\/PERSONA_CONTEXT\]\s*User message:\s*/g, '');
  originalText = originalText.replace(/\[TOOLS_CONTEXT\][\s\S]*?\[\/TOOLS_CONTEXT\]\s*User message:\s*/g, '');
  originalText = originalText.replace(/<!--SOUL_CONTEXT:[^>]+-->/g, '');
  originalText = originalText.replace(/<!--TOOLS_CONTEXT:[^>]+-->/g, '');

  return {
    soulContextLength,
    toolsContextLength,
    totalInjectedLength: soulContextLength + toolsContextLength,
    originalLength: originalText.length
  };
}