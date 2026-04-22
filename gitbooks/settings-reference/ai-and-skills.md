---
icon: head-side-circuit
---

# AI & Skills

#### Local AI Model

Choose your model tier based on device capability.

OpenHuman detects your system specs automatically: RAM, CPU cores, and GPU type.

| Tier        | Size   | RAM    | GPU                       | Model                  | Best For                             |
| ----------- | ------ | ------ | ------------------------- | ---------------------- | ------------------------------------ |
| Lightweight | \~1 GB | 4 GB+  | No                        | gemma3:1b-it-q4\_0     | Older machines, minimal resource use |
| Balanced    | \~3 GB | 8 GB+  | No                        | gemma3:4b-it-qat       | Moderate hardware                    |
| Performance | \~8 GB | 16 GB+ | Yes (Apple Silicon Metal) | gemma3:12b-it-q4\_K\_M | Best quality, recommended            |

The active tier is marked "ACTIVE" and the recommended tier is marked "RECOMMENDED." Switching tiers triggers a new model download.

"Show Advanced" expands additional model configuration options.

#### AI Configuration

Configure persona, prompting behavior, and AI runtime settings.

**AI System Overview:** Prompt and markdown orchestration handled in Rust runtime. Shows configuration status (Ready or Fallback Mode) and loading duration.

**Local Model Runtime:** State (ready/loading/error) and target model. "Open Manager" and "Retry Download" buttons.

**SOUL Persona Configuration:** Defines OpenHuman's identity and behavior. Shows identity name (OpenHuman), role (AI assistant), source, and load timestamp. "Refresh SOUL" reloads persona configuration.

**TOOLS Configuration:** Available tools count and active skills count. "Refresh TOOLS" reloads tool definitions.

**Refresh All AI Configuration:** Reloads the entire AI configuration stack.

#### Skills

Configure browser access and installed skill capabilities.

**Browser Access:** Controls whether OpenHuman's browser tool can visit public domains. Private and file URLs are always blocked.

"Restrict to allowlist" limits access to only approved domains. When unchecked, any public domain is accessible.

Shows registered integrations list.
