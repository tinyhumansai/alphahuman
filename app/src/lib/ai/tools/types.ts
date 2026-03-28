/**
 * Type definitions for the AI tools system.
 * Provides interfaces for tool loading, caching, and configuration management.
 */

/** Tool definition from skills runtime */
export interface ToolDefinition {
  /** Skill that provides this tool */
  skillId: string;
  /** Tool name/identifier */
  name: string;
  /** Tool description for AI understanding */
  description: string;
  /** JSON Schema for input validation */
  inputSchema: { type: 'object'; properties: Record<string, unknown>; required?: string[] };
}

/** Tool category for organization */
export interface ToolCategory {
  /** Category identifier */
  id: string;
  /** Display name */
  name: string;
  /** Category description */
  description: string;
  /** Skills that belong to this category */
  skills: string[];
  /** Number of tools in category */
  toolCount?: number;
}

/** Skill grouping with tools */
export interface SkillGroup {
  /** Skill identifier */
  skillId: string;
  /** Display name */
  name: string;
  /** Category this skill belongs to */
  category: string;
  /** Tools provided by this skill */
  tools: ToolDefinition[];
}

/** Environment configuration for tools */
export interface ToolEnvironment {
  /** Environment identifier */
  id: string;
  /** Display name */
  name: string;
  /** Environment description */
  description: string;
  /** Access level description */
  accessLevel: string;
  /** Rate limiting information */
  rateLimits: string;
  /** Authentication requirements */
  authentication: string;
  /** Logging configuration */
  logging: string;
}

/** Complete tools configuration */
export interface ToolsConfig {
  /** Raw markdown source */
  raw: string;
  /** All discovered tools */
  tools: ToolDefinition[];
  /** Tools grouped by skill */
  skillGroups: Record<string, SkillGroup>;
  /** Tool categories */
  categories: Record<string, ToolCategory>;
  /** Available environments */
  environments: Record<string, ToolEnvironment>;
  /** Tool statistics */
  statistics: ToolStatistics;
  /** Whether this is the default config or user-customized */
  isDefault: boolean;
  /** Last loaded timestamp */
  loadedAt: number;
}

/** Tool usage and availability statistics */
export interface ToolStatistics {
  /** Total number of tools */
  totalTools: number;
  /** Number of active skills */
  activeSkills: number;
  /** Number of categories */
  categoriesCount: number;
  /** Tools by category */
  toolsByCategory: Record<string, number>;
  /** Skills by category */
  skillsByCategory: Record<string, string[]>;
}

/** Tool parsing result from markdown */
export interface ToolParseResult {
  /** Successfully parsed tools */
  tools: ToolDefinition[];
  /** Parsing errors */
  errors: string[];
  /** Warnings during parsing */
  warnings: string[];
}

/** Cache entry for tools configuration */
export interface ToolsCacheEntry {
  /** Cached configuration */
  config: ToolsConfig;
  /** Cache timestamp */
  timestamp: number;
  /** Cache version for invalidation */
  version: string;
}

/** Tool discovery source */
export type ToolSource = 'runtime' | 'github' | 'bundled' | 'mock';

/** Tool discovery result */
export interface ToolDiscoveryResult {
  /** Discovered tools */
  tools: ToolDefinition[];
  /** Source of the tools */
  source: ToolSource;
  /** Discovery timestamp */
  discoveredAt: number;
  /** Any errors during discovery */
  errors?: string[];
}
