/**
 * Skill Manager — orchestrates multiple skill runtimes.
 *
 * Singleton that manages skill discovery, lifecycle, setup flows,
 * and tool invocation. Dispatches status changes to Redux.
 */

import { SkillRuntime } from "./runtime";
import type {
  SkillManifest,
  SkillStatus,
  SetupStep,
  SetupResult,
  SkillToolDefinition,
  SkillOptionDefinition,
} from "./types";
import { store } from "../../store";
import {
  addSkill,
  setSkillStatus,
  setSkillError,
  setSkillSetupComplete,
  setSkillTools,
  setSkillState,
} from "../../store/skillsSlice";
import { TELEGRAM_API_ID, TELEGRAM_API_HASH } from "../../utils/config";

class SkillManager {
  private runtimes = new Map<string, SkillRuntime>();

  /**
   * Add a discovered skill manifest to Redux.
   */
  registerSkill(manifest: SkillManifest): void {
    // Validate that skill name doesn't contain underscores (used for tool namespacing)
    if (manifest.id.includes("_")) {
      console.error(
        `Skill name "${manifest.id}" contains underscore. Skill names cannot contain underscores as they are used for tool namespacing (skillId__toolName).`
      );
      return;
    }
    store.dispatch(addSkill({ manifest }));
  }

  /**
   * Start a skill — spawn process, load, check setup status.
   * If setup is already complete, loads the skill fully and lists tools.
   */
  async startSkill(manifest: SkillManifest): Promise<void> {
    const skillId = manifest.id;

    // Check if already running
    if (this.runtimes.has(skillId)) {
      const existing = this.runtimes.get(skillId)!;
      if (existing.isRunning) return;
      // Dead runtime — clean up
      this.runtimes.delete(skillId);
    }

    store.dispatch(setSkillStatus({ skillId, status: "starting" }));

    const runtime = new SkillRuntime(manifest);

    // Wire up reverse RPC handler
    runtime.onReverseRpc(async (method, params) => {
      return this.handleReverseRpc(skillId, method, params);
    });

    try {
      // Build env vars to pass to the process
      const env: Record<string, string> = {};
      if (TELEGRAM_API_ID) {
        env.TELEGRAM_API_ID = String(TELEGRAM_API_ID);
      }
      if (TELEGRAM_API_HASH) {
        env.TELEGRAM_API_HASH = TELEGRAM_API_HASH;
      }

      await runtime.start(env);
      this.runtimes.set(skillId, runtime);

      store.dispatch(setSkillStatus({ skillId, status: "running" }));

      // Load the skill
      await runtime.load();

      // Check if setup is needed
      const state = store.getState();
      const skillState = state.skills.skills[skillId];
      const setupRequired = manifest.setup?.required && !skillState?.setupComplete;

      if (setupRequired) {
        store.dispatch(setSkillStatus({ skillId, status: "setup_required" }));
      } else {
        // Skill is ready — list tools
        await this.activateSkill(skillId);
      }
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      store.dispatch(setSkillError({ skillId, error: msg }));
      this.runtimes.delete(skillId);
      throw err;
    }
  }

  /**
   * Activate a skill that has completed setup — list its tools and mark as ready.
   */
  private async activateSkill(skillId: string): Promise<void> {
    const runtime = this.runtimes.get(skillId);
    if (!runtime) return;

    try {
      const tools = await runtime.listTools();
      store.dispatch(setSkillTools({ skillId, tools }));
      store.dispatch(setSkillStatus({ skillId, status: "ready" }));
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      store.dispatch(setSkillError({ skillId, error: msg }));
    }
  }

  /**
   * Start the setup flow for a skill. Returns the first step.
   */
  async startSetup(skillId: string): Promise<SetupStep> {
    const runtime = this.runtimes.get(skillId);
    if (!runtime) {
      throw new Error(`Skill ${skillId} is not running`);
    }

    store.dispatch(
      setSkillStatus({ skillId, status: "setup_in_progress" }),
    );
    return runtime.setupStart();
  }

  /**
   * Submit a setup step. Returns the result (next step, error, or complete).
   */
  async submitSetup(
    skillId: string,
    stepId: string,
    values: Record<string, unknown>,
  ): Promise<SetupResult> {
    const runtime = this.runtimes.get(skillId);
    if (!runtime) {
      throw new Error(`Skill ${skillId} is not running`);
    }

    const result = await runtime.setupSubmit(stepId, values);

    if (result.status === "complete") {
      store.dispatch(setSkillSetupComplete({ skillId, complete: true }));
      // Activate the skill now that setup is done
      await this.activateSkill(skillId);
    }

    return result;
  }

  /**
   * Cancel the setup flow for a skill.
   */
  async cancelSetup(skillId: string): Promise<void> {
    const runtime = this.runtimes.get(skillId);
    if (!runtime) return;

    try {
      await runtime.setupCancel();
    } catch {
      // Ignore errors on cancel
    }
    store.dispatch(setSkillStatus({ skillId, status: "setup_required" }));
  }

  /**
   * Call a tool on a running skill.
   */
  async callTool(
    skillId: string,
    name: string,
    args: Record<string, unknown>,
  ): Promise<{ content: Array<{ type: string; text: string }>; isError: boolean }> {
    const runtime = this.runtimes.get(skillId);
    if (!runtime) {
      throw new Error(`Skill ${skillId} is not running`);
    }
    return runtime.callTool(name, args);
  }

  /**
   * Get the list of tools for a running skill.
   */
  async listTools(skillId: string): Promise<SkillToolDefinition[]> {
    const runtime = this.runtimes.get(skillId);
    if (!runtime) {
      throw new Error(`Skill ${skillId} is not running`);
    }
    return runtime.listTools();
  }

  /**
   * List runtime-configurable options for a running skill.
   */
  async listOptions(skillId: string): Promise<SkillOptionDefinition[]> {
    const runtime = this.runtimes.get(skillId);
    if (!runtime) {
      throw new Error(`Skill ${skillId} is not running`);
    }
    return runtime.listOptions();
  }

  /**
   * Set a single option on a running skill.
   */
  async setOption(skillId: string, name: string, value: unknown): Promise<void> {
    const runtime = this.runtimes.get(skillId);
    if (!runtime) {
      throw new Error(`Skill ${skillId} is not running`);
    }
    await runtime.setOption(name, value);
    // Refresh tools list since tool_filter options can change available tools
    await this.activateSkill(skillId);
  }

  /**
   * Forward session start to all ready skills.
   */
  async sessionStart(sessionId: string): Promise<void> {
    for (const [, runtime] of this.runtimes) {
      if (runtime.isRunning) {
        try {
          await runtime.sessionStart(sessionId);
        } catch {
          // Non-critical
        }
      }
    }
  }

  /**
   * Forward session end to all ready skills.
   */
  async sessionEnd(sessionId: string): Promise<void> {
    for (const [, runtime] of this.runtimes) {
      if (runtime.isRunning) {
        try {
          await runtime.sessionEnd(sessionId);
        } catch {
          // Non-critical
        }
      }
    }
  }

  /**
   * Disconnect a skill — stop it and reset setup state.
   */
  async disconnectSkill(skillId: string): Promise<void> {
    await this.stopSkill(skillId);
    store.dispatch(setSkillSetupComplete({ skillId, complete: false }));
    store.dispatch(setSkillState({ skillId, state: {} }));
  }

  /**
   * Stop a specific skill.
   */
  async stopSkill(skillId: string): Promise<void> {
    const runtime = this.runtimes.get(skillId);
    if (!runtime) return;

    store.dispatch(setSkillStatus({ skillId, status: "stopping" }));
    try {
      await runtime.stop();
    } catch {
      // Ignore stop errors
    }
    this.runtimes.delete(skillId);
    store.dispatch(setSkillStatus({ skillId, status: "installed" }));
  }

  /**
   * Stop all running skills.
   */
  async stopAll(): Promise<void> {
    const ids = Array.from(this.runtimes.keys());
    await Promise.all(ids.map((id) => this.stopSkill(id)));
  }

  /**
   * Check if a skill is currently running.
   */
  isSkillRunning(skillId: string): boolean {
    return this.runtimes.get(skillId)?.isRunning ?? false;
  }

  /**
   * Get the current status of a skill from Redux.
   */
  getSkillStatus(skillId: string): SkillStatus | undefined {
    return store.getState().skills.skills[skillId]?.status;
  }

  // -----------------------------------------------------------------------
  // Reverse RPC handling
  // -----------------------------------------------------------------------

  private async handleReverseRpc(
    skillId: string,
    method: string,
    params: Record<string, unknown>,
  ): Promise<unknown> {
    switch (method) {
      case "state/get":
        return { state: store.getState().skills.skillStates[skillId] ?? {} };

      case "state/set": {
        // For now, store in Redux
        // The skill's state is stored in skillStates[skillId]
        const partial = params.partial as Record<string, unknown>;
        const currentState =
          store.getState().skills.skillStates[skillId] ?? {};
        const newState = { ...currentState, ...partial };
        // We need a setSkillState action for this
        store.dispatch({
          type: "skills/setSkillState",
          payload: { skillId, state: newState },
        });
        return { ok: true };
      }

      case "data/read": {
        // Read from the skill's data directory
        // For now, use fetch or Tauri command
        const filename = params.filename as string;
        try {
          const { invoke } = await import("@tauri-apps/api/core");
          const content = await invoke<string>("skill_read_data", {
            skillId,
            filename,
          });
          return { content };
        } catch {
          return { content: "" };
        }
      }

      case "data/write": {
        const filename = params.filename as string;
        const content = params.content as string;
        try {
          const { invoke } = await import("@tauri-apps/api/core");
          await invoke("skill_write_data", {
            skillId,
            filename,
            content,
          });
        } catch (err) {
          console.error("[skill-manager] data/write error:", err);
        }
        return { ok: true };
      }

      case "intelligence/emitEvent":
        // Future: forward to intelligence system
        console.debug("[skill-manager] Intelligence event:", params);
        return { ok: true };

      case "entities/upsert":
        // Future: forward to entity manager
        console.debug("[skill-manager] Entity upsert:", params);
        return { ok: true };

      case "entities/search":
        // Future: forward to entity manager
        return { results: [] };

      case "entities/upsertRelationship":
        console.debug("[skill-manager] Relationship upsert:", params);
        return { ok: true };

      case "entities/getRelationships":
        return { results: [] };

      default:
        throw new Error(`Unknown reverse RPC method: ${method}`);
    }
  }
}

// Export singleton
export const skillManager = new SkillManager();
