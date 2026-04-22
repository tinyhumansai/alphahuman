---
icon: code-branch
---

# Architecture

OpenHuman is built on the OpenClaw architecture and open-sourced under the GNU GPL3 license. This page explains how the major components connect.

#### The four pillars

OpenHuman's architecture rests on four pillars that work together:

<figure><img src="../.gitbook/assets/V15 — Three Pillars@2x.png" alt=""><figcaption></figcaption></figure>

**Neocortex** is the memory engine. It ingests data from connected sources, builds knowledge graphs, manages tiered memory, and provides the recall capabilities that power both conscious queries and subconscious processing. Detailed in Neocortex.

**Local processing** runs AI models directly on your device. Gemma 3 handles chat, vision analysis (for Screen Intelligence), speech-to-text, and text-to-speech. This layer processes raw screen captures, handles Auto-complete suggestions, and manages local encryption. No raw data leaves this layer.

<figure><img src="/broken/files/9CaipntgroxxUYuIrQXB" alt=""><figcaption></figcaption></figure>

**Multi-agent orchestration** distributes work across specialized agents rather than relying on a single monolithic model. An orchestrator agent manages routing, personality, and context distribution. Specialist agents handle specific domains: communication analysis, document synthesis, task management, trading. Agents execute in parallel, not sequentially, enabling real-time responsiveness.

**Privacy-preserving inference** ensures that raw data never leaves the user's device. Data is encrypted on-device with AES-256-GCM. Encryption keys never leave the device. Only compressed metadata and summaries are processed server-side. Detailed in [Privacy & Security](../product/privacy-and-security.md).

#### How data flows

<figure><img src="/broken/files/7fx1YutriAn2pnRrusN7" alt=""><figcaption></figcaption></figure>

1. **Local capture.** Screen Intelligence captures your screen activity. The on-device vision model processes raw screenshots into structured summaries. Raw screenshots are discarded after processing.
2. **Ingestion.** Data arrives from connected sources: Telegram, Slack, Gmail, Notion, blockchain wallets, and others. Each source has its own connector that handles authentication and data retrieval. Screen activity summaries from the local layer are also ingested.
3. **Compression.** Neocortex processes raw data on-device. Semantic deduplication removes noise. Entity resolution links references across sources. Temporal weighting prioritizes recency. The output is a compressed knowledge graph, not raw text.
4. **Storage.** The knowledge graph is stored in Neocortex's tiered memory system. Raw data is discarded after compression. Only structured metadata and summaries persist.
5. **Conscious processing.** When you make a request, the orchestrator routes it to the appropriate specialist agent(s). Those agents query Neocortex for relevant context, process your request, and return a result.
6. **Subconscious processing.** Independent of your requests, the subconscious system triggers periodic memory recalls from Neocortex. These feed into a self-learning loop that surfaces proactive insights, patterns, and recommendations.
7. **Output.** Results are presented to you directly or exported to connected tools like Notion and Google Sheets. Only structured, compressed intelligence leaves the device. Raw data never does.

#### Local Models

The desktop app runs the following models on your device hardware:

**Gemma 3 (chat):** Handles conversational interactions, query processing, and response generation locally. For requests that require deeper reasoning or larger context, processing is routed to the server-side intelligence layer.

**Gemma 3 (vision):** Processes screen captures from Screen Intelligence. Analyzes visual content to produce structured summaries of what is on screen: application names, visible text, interface elements, and user activity context.

**Speech-to-text:** Converts voice input to text locally. No audio data is sent to any server.

**Text-to-speech:** Generates spoken responses locally for voice interaction.

All local models run on your device's CPU. No GPU is required, though GPU acceleration is used when available. The models are optimized for low memory footprint and fast inference.

#### Model-agnostic design

OpenHuman is not locked to any single AI model. The compression engine and memory layer sit on top of the AI infrastructure, not inside it. Today the system works with specific models. Tomorrow it could feed context to any model: GPT, Claude, Gemini, Llama, Mistral, or whatever comes next.

This is a deliberate architectural choice. AI models are commoditizing. Performance is converging. The real differentiator is the context you feed the model, and OpenHuman owns the context layer.

#### Open source

OpenHuman is publicly available on GitHub under the GNU GPL3 license.

**GitHub:** [github.com/tinyhumansai/openhuman](https://github.com/tinyhumansai/openhuman)

**Neocortex benchmarks:** [github.com/tinyhumansai/neocortex/tree/main/benchmarks](https://github.com/tinyhumansai/neocortex/tree/main/benchmarks)

Contributions, feedback, and issues are welcomed. The project is in early alpha.
