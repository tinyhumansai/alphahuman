import debug from 'debug';

import { callCoreRpc } from '../coreRpcClient';

const log = debug('skillsApi');

/**
 * Scope a skill was discovered in.
 *
 * Mirrors `openhuman::skills::ops::SkillScope` on the Rust side — serialized
 * as a lowercase string (`"user" | "project" | "legacy"`).
 */
export type SkillScope = 'user' | 'project' | 'legacy';

/**
 * Wire-format representation of a discovered skill returned by
 * `openhuman.skills_list`.
 *
 * Paths are intentionally serialized as strings (not URLs) to avoid lossy
 * conversions on non-UTF-8 filesystems.
 */
export interface SkillSummary {
  /** Stable identifier — equal to `name` on the Rust side. */
  id: string;
  /** Display name, from frontmatter or directory. */
  name: string;
  /** Short prose summary from frontmatter / `description`. */
  description: string;
  /** Version string, if declared (empty otherwise). */
  version: string;
  /** Author string, if declared. */
  author: string | null;
  /** Tags declared in frontmatter metadata. */
  tags: string[];
  /** Tool hint from `allowed-tools`. */
  tools: string[];
  /** Prompt files declared in the legacy manifest. */
  prompts: string[];
  /** Path to `SKILL.md` (or `skill.json`) on disk, or null if unknown. */
  location: string | null;
  /** Bundled resource files, relative to the skill root. */
  resources: string[];
  /** Where the skill came from. */
  scope: SkillScope;
  /** True when loaded from the legacy `skills/` layout. */
  legacy: boolean;
  /** Non-fatal parse warnings to surface in the UI. */
  warnings: string[];
}

interface SkillsListResult {
  skills: SkillSummary[];
}

/**
 * Result of `openhuman.skills_read_resource`.
 */
export interface SkillResourceContent {
  /** Echo of the requested skill id. */
  skillId: string;
  /** Echo of the requested relative path. */
  relativePath: string;
  /** UTF-8 file contents (<= 128 KB). */
  content: string;
  /** Size of the file on disk, in bytes. */
  bytes: number;
}

interface RawSkillsReadResourceResult {
  skill_id: string;
  relative_path: string;
  content: string;
  bytes: number;
}

interface Envelope<T> {
  data?: T;
}

function unwrapEnvelope<T>(response: Envelope<T> | T): T {
  if (response && typeof response === 'object' && 'data' in response) {
    const envelope = response as Envelope<T>;
    if (envelope.data !== undefined) {
      return envelope.data as T;
    }
  }
  return response as T;
}

export const skillsApi = {
  /** Enumerate SKILL.md / legacy skills visible in the active workspace. */
  listSkills: async (): Promise<SkillSummary[]> => {
    log('listSkills: request');
    const response = await callCoreRpc<Envelope<SkillsListResult> | SkillsListResult>({
      method: 'openhuman.skills_list',
    });
    const result = unwrapEnvelope(response);
    const skills = result?.skills ?? [];
    log('listSkills: response count=%d', skills.length);
    return skills;
  },

  /**
   * Read a single bundled resource file from a discovered skill. Rejects on
   * traversal, symlink escape, non-UTF-8 payloads, or files larger than
   * 128 KB — the caller surfaces the error string verbatim in the drawer.
   */
  readSkillResource: async ({
    skillId,
    relativePath,
  }: {
    skillId: string;
    relativePath: string;
  }): Promise<SkillResourceContent> => {
    log('readSkillResource: request skillId=%s path=%s', skillId, relativePath);
    const response = await callCoreRpc<
      Envelope<RawSkillsReadResourceResult> | RawSkillsReadResourceResult
    >({
      method: 'openhuman.skills_read_resource',
      params: { skill_id: skillId, relative_path: relativePath },
    });
    const raw = unwrapEnvelope(response);
    const normalized: SkillResourceContent = {
      skillId: raw.skill_id,
      relativePath: raw.relative_path,
      content: raw.content,
      bytes: raw.bytes,
    };
    log('readSkillResource: response bytes=%d', normalized.bytes);
    return normalized;
  },
};
