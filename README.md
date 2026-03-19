<h1 align="center">OpenHuman</h1>

<p align="center">
  <strong>The age of super intelligence is here. OpenHuman is your artificial conscious human.</strong>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/status-early%20beta-orange" alt="Early Beta" />
  <img src="https://img.shields.io/badge/platform-macOS%20%7C%20Windows%20%7C%20Linux%20%7C%20Android%20%7C%20iOS-blue" alt="Platforms" />
  <a href="https://github.com/tinyhumansai/openhuman/releases/latest"><img src="https://img.shields.io/github/v/release/tinyhumansai/openhuman?label=latest" alt="Latest Release" /></a>
</p>

<p align="center">
  <a href="#what-is-openhuman">About</a> ·
  <a href="#openhuman-vs-openclaw">vs OpenClaw</a> ·
  <a href="#download">Download</a> ·
  <a href="#getting-started">Getting Started</a> ·
  <a href="docs/ARCHITECTURE.md">Architecture</a> ·
  <a href="CHANGELOG.md">Changelog</a>
</p>

![The Tet](./docs/the-tet.png)

<p align="center" style="font-style: italic">
  "No Soul. No Humanity. The Tet. What a brilliant machine" — Morgan Freeman <a href="https://youtu.be/SveLVpqy_Rc?si=y83aZNokPiUjILN0&t=60">as he reminisces about alien superintelligence</a> in the movie Oblivion
</p>

OpenHuman is a personal AI assistant that helps you manage high-volume communication without reading everything yourself. It connects to your messaging platforms and productivity tools, understands conversations in context, and produces clear, actionable outputs you can use immediately.

OpenHuman is **not** a chatbot, browser extension, or cloud-only service. It is a **native application** that runs on your device, connects to your tools, and works only when you ask it to. Think of it as a second brain that sits across your communication and productivity stack.

## OpenHuman vs OpenClaw

OpenHuman is designed to be simpler to deploy, cheaper to run, and more intelligent in how it uses models and memory.

|                  | OpenClaw                                                | OpenHuman                                                                                                   |
| ---------------- | ------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------- |
| **Runtime**      | Node.js (TypeScript)                                    | Tauri (Rust + React), native binary                                                                         |
| **Inference**    | Single-tier or manual routing                           | **Custom two-tier**: task-routed (summarize/vibe/memory → cheap; complex/tools → premium)                   |
| **Memory**       | Often external (Pinecone, Lucid, etc.) or markdown-only | **Custom hybrid**: SQLite FTS5 + vector similarity, optional encryption, no external vector DB              |
| **Tunneling**    | Third-party (ngrok, Cloudflare, Tailscale) or none      | **Custom tunneling** — secure app-to-backend path without vendor lock-in                                    |
| **Cost**         | Typically one premium model for everything              | **Lower** — Tier 1 for most ops; Tier 2 only when needed                                                    |
| **Intelligence** | General-purpose agent loop                              | **Smarter** — vibe detection, interest-based escalation, constitution-driven behavior, session-aware memory |
| **Deployment**   | Server/Node process, high memory footprint              | Native desktop/mobile app, Rust socket manager, smaller footprint                                           |

> OpenClaw is a strong open-source agent framework. We chose to build a custom stack so we could own inference routing, memory, and tunneling end-to-end and optimize for cost and clarity.

---

## Download

> **Early Beta** — OpenHuman is under active development. Expect rough edges.

| Platform    | Variant                     | Download                                                                                                     |
| ----------- | --------------------------- | ------------------------------------------------------------------------------------------------------------ |
| **macOS**   | Apple Silicon (M1/M2/M3/M4) | [`.dmg` (aarch64)](https://github.com/tinyhumansai/openhuman/releases/latest/download/OpenHuman_aarch64.dmg) |
| **macOS**   | Intel                       | [`.dmg` (x64)](https://github.com/tinyhumansai/openhuman/releases/latest/download/OpenHuman_x64.dmg)         |
| **Windows** | x64                         | [`.msi`](https://github.com/tinyhumansai/openhuman/releases/latest/download/OpenHuman_x64_en-US.msi)         |
| **Linux**   | Debian / Ubuntu             | [`.deb` (amd64)](https://github.com/tinyhumansai/openhuman/releases/latest/download/OpenHuman_amd64.deb)     |
| **Linux**   | Fedora / RHEL               | [`.rpm` (x86_64)](https://github.com/tinyhumansai/openhuman/releases/latest/download/OpenHuman_x86_64.rpm)   |
| **Linux**   | Universal                   | [`.AppImage`](https://github.com/tinyhumansai/openhuman/releases/latest/download/OpenHuman_amd64.AppImage)   |
| **Android** | —                           | Coming soon                                                                                                  |
| **iOS**     | —                           | Coming soon                                                                                                  |

Browse all releases: [github.com/tinyhumansai/openhuman/releases](https://github.com/tinyhumansai/openhuman/releases)

## Getting Started

1. **Download** the installer for your platform from the [releases page](https://github.com/tinyhumansai/openhuman/releases/latest)
2. **Install** the app (drag to Applications on macOS, or use your package manager on Linux)
3. **Connect a source** — follow the in-app onboarding to link Telegram, Notion, Gmail, or other services
4. **Run your first request** — ask the AI to summarize what you missed, extract action items, or surface key decisions

---

## Links

- [Architecture Overview](docs/ARCHITECTURE.md) — How OpenHuman is built
- [Changelog](CHANGELOG.md) — Release history
- [Website](https://openhuman.xyz) — Learn more

---

# Star us on Github

_Like contributing towards AGI 🧠? Give this repo a star and spread the love ❤️_

<p align="center">
  <a href="https://www.star-history.com/#tinyhumansai/openhuman&type=date&legend=top-left">
    <picture>
     <source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/svg?repos=tinyhumansai/openhuman&type=date&theme=dark&legend=top-left" />
     <source media="(prefers-color-scheme: light)" srcset="https://api.star-history.com/svg?repos=tinyhumansai/openhuman&type=date&legend=top-left" />
     <img alt="Star History Chart" src="https://api.star-history.com/svg?repos=tinyhumansai/openhuman&type=date&legend=top-left" />
    </picture>
  </a>
</p>

# Contributors Hall of Fame

Show some love and end up in the hall of fame

<a href="https://github.com/tinyhumansai/openhuman/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=tinyhumansai/openhuman" alt="openhuman contributors" />
</a>
