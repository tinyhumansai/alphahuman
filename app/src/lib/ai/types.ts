/**
 * Unified types for the AI configuration system.
 * Combines SOUL persona and TOOLS configurations.
 */
import type { SoulConfig } from './soul/types';
import type { ToolsConfig } from './tools/types';

/** Complete AI configuration combining SOUL and TOOLS */
export interface AIConfig {
  /** SOUL persona configuration */
  soul: SoulConfig;
  /** Tools configuration */
  tools: ToolsConfig;
  /** Overall loading metadata */
  metadata: AIConfigMetadata;
}

/** AI configuration loading metadata */
export interface AIConfigMetadata {
  /** Last loaded timestamp */
  loadedAt: number;
  /** Loading duration in milliseconds */
  loadingDuration: number;
  /** Whether any component used fallback data */
  hasFallbacks: boolean;
  /** Sources used for loading */
  sources: {
    soul: 'memory' | 'localStorage' | 'github' | 'bundled';
    tools: 'memory' | 'localStorage' | 'github' | 'bundled';
  };
  /** Loading errors (non-fatal) */
  errors?: string[];
}

/** AI configuration loading options */
export interface AIConfigLoadOptions {
  /** Force reload from remote sources */
  forceRefresh?: boolean;
  /** Include detailed loading metadata */
  includeMetadata?: boolean;
  /** Timeout for remote loading (ms) */
  timeout?: number;
}

/** AI configuration cache entry */
export interface AIConfigCacheEntry {
  /** Cached configuration */
  config: AIConfig;
  /** Cache timestamp */
  timestamp: number;
  /** Cache version for invalidation */
  version: string;
}

/** AI configuration loading result */
export interface AIConfigLoadResult {
  /** Loaded configuration */
  config: AIConfig;
  /** Loading was successful */
  success: boolean;
  /** Any errors encountered */
  errors: string[];
  /** Loading duration */
  duration: number;
}
