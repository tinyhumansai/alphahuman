---
icon: question
---

# FAQ & Troubleshooting

## Frequently Asked Questions

#### What is OpenHuman?

OpenHuman is a personal AI assistant that runs local AI models on your device and connects to your communication platforms and productivity tools. It helps you build continuous awareness across your digital life through Screen Intelligence, Auto-complete, conversation analysis, signal extraction, and structured workflows.

***

#### What is Neocortex

Neocortex is OpenHuman's memory engine. It is a human-like AI memory system that can work with over 1 billion tokens of data. It indexes 10 million tokens in under 10 seconds, costs $1 to index 5 million tokens, and runs on a MacBook Air CPU with no GPU required. Unlike vector databases, Neocortex understands time, entities, and relationships. It builds knowledge graphs and manages memory through tiered compression inspired by how the human brain works. Learn more in Neocortex.

***

#### What does "Big Data AI" mean?

Every AI model today is a prompt engine. You type something, it responds, and the context disappears. OpenHuman is different. It compresses your entire organizational life, messages, documents, tools, transactions, into a structured knowledge graph that persists and evolves. This is what we mean by Big Data AI: an AI that operates on months of your real data, not just the prompt you typed right now.

#### **How is OpenHuman different from ChatGPT, Claude, or Gemini?**

Those models are brilliant at reasoning and generation. But they are stateless. They know nothing about your actual life beyond what you paste into the chat window. OpenHuman is the context layer that makes those models useful. It compresses your organizational data into structured intelligence that any AI can reason over. Think of it this way: ChatGPT is the brain. OpenHuman is the memory.

***

#### **How is OpenHuman different from other AI memory solutions like Mem0, SuperMemory, or MemGPT?**

Most AI memory solutions use vector databases that retrieve whatever is semantically similar, but similarity alone says nothing about importance. They also cannot support consciousness-like systems or process data accurately at scale beyond 10 million tokens. Neocortex is architecturally different: it uses tiered memory, knowledge graphs, temporal weighting, and semantic deduplication. It processes 10 million tokens in under 10 seconds and supports over 1 billion tokens total. It does this with zero LLM dependency.

***

#### **Is OpenHuman open source?**

Yes. OpenHuman is built on the OpenClaw architecture and licensed under GNU GPL3. The full codebase is available on [GitHub](https://github.com/tinyhumansai/openhuman). Neocortex benchmarks are also open-sourced. Contributions and feedback are welcomed.

***

#### **Is OpenHuman AGI?**

No. OpenHuman is not AGI, and we do not claim it is. It is a meaningful architectural step closer to AGI, with innovations in memory (Neocortex) and consciousness-like processing (the subconscious system) that go beyond what existing agentic systems offer. But it operates within defined boundaries and requires human judgment.

***

#### **Does Screen Intelligence see everything on my screen?**

Only for the apps you allow. Screen Intelligence captures screenshots approximately every 5 seconds and processes them using the on-device vision model. You control which applications are included through per-app permissions. You can exclude any application, such as banking, medical, or personal apps. Raw screenshots are processed locally and discarded. Only compressed text summaries are stored. No raw screen images are ever sent to any server.

***

#### **Can I turn off Screen Intelligence?**

Yes. Screen Intelligence is entirely optional. You can disable it at any time from settings. OpenHuman continues to function with your other connected sources and manual queries.

***

#### **Does Auto-complete send my typing to a server?**

No. Auto-complete runs entirely on the local model using cached memory context from Neocortex. No keystroke data, text input, or completion suggestions leave your device.

#### Does OpenHuman read all my messages?

No. OpenHuman only processes messages **when you ask it to** and only within the scope of your request.

If you ask it to summarize a specific conversation, it reads that conversation. If you do not reference a source, it is not accessed. There is no background monitoring or continuous scanning.

***

#### Is my data safe?

Yes. OpenHuman is designed around zero retention of message content. Data is processed to produce an output and then discarded. Your conversations are never stored long-term, and they are never used to train AI models.

On desktop platforms, credentials are stored in your operating system's secure keychain. All communication between the app and OpenHuman's servers is encrypted.

***

#### Does OpenHuman store my messages?

Messages are processed only to fulfill your request. They are **not permanently stored or reused** beyond producing the requested output. Derived intelligence like summaries and workflow records may persist, but raw message content does not.

***

#### Can OpenHuman send messages on my behalf?

OpenHuman can send messages, manage chats, and perform 80+ actions through your connected Telegram account. All actions use your own encrypted credentials and run locally. You control which capabilities are active through the Skills settings

***

#### Who is OpenHuman for?

OpenHuman is useful for anyone who:

* Manages high-volume communication across multiple platforms and groups
* Needs to stay on top of decisions, action items, and context without reading everything
* Works in distributed teams, communities, or coordination-heavy environments
* Wants structured outputs from conversations, exportable to tools like Notion or Google Sheets

You do not need to be technical to use it.

***

#### What platforms does OpenHuman support?

OpenHuman runs natively on **macOS, Windows, Linux, Android, and iOS**, with a **web version** for browser access. Your account and settings sync across all platforms.

***

#### What integrations are available?

OpenHuman currently integrates with **Telegram** (read and analyze conversations), **Notion** (export structured outputs), and **Google Sheets** (export tabular data and reports). More integrations are planned.

***

#### How much does OpenHuman cost?

OpenHuman offers individual and team plans with core analysis included. Deeper features are available in higher tiers. A credit system provides usage-metered flexibility, and credits can be earned through referrals.

See [Pricing](../product/pricing.md) for details.

***

## Troubleshooting

#### Summaries feel incomplete or too broad

If a summary feels incomplete, the most common cause is overly broad scope. When a request spans many conversations, long time ranges, or high-volume groups, OpenHuman prioritizes signal over detail.

**Solution:** Narrow the request to a specific conversation, time window, or intent. OpenHuman performs best when it knows what kind of outcome you are looking for.

***

#### Important context seems missing

OpenHuman only processes the data required to fulfill a request. If relevant context exists outside the scope of that request, it will not be included.

**Solution:** Expand the scope explicitly by referencing additional conversations or extending the time range.

***

#### Outputs feel incorrect or misinterpreted

OpenHuman interprets conversations probabilistically. Tone, sarcasm, and informal language can be misread, especially in fast-moving or meme-heavy discussions.

**Solution:** Refine the request or re-run analysis with a narrower scope. Outputs should be treated as assistance rather than ground truth.

***

#### **Screen Intelligence summaries seem inaccurate**

The on-device vision model processes screenshots locally. Accuracy depends on screen clarity, text size, and application complexity. Highly dynamic interfaces, very small text, or applications with unusual layouts may produce less precise summaries.

**Solution:** Check your per-app permissions to ensure the relevant app is included. For applications with small text, increasing display zoom may improve capture quality. If summaries for a specific app are consistently poor, you can exclude it and rely on other context sources instead.

***

#### **Auto-complete suggestions feel irrelevant**

Auto-complete draws on your Neocortex memory context. If you have recently connected sources or just started using Screen Intelligence, the memory may not yet have enough context to generate relevant suggestions.

**Solution:** Give it time. As more data is indexed, suggestion quality improves. You can also check that your most relevant connected sources are active and that Screen Intelligence is enabled for the applications where you do most of your work.

#### Trust or risk signals feel inaccurate

Trust and risk intelligence is indicative, not authoritative. Signals may lag real-world changes or surface false positives.

**Solution:** Use these signals as inputs into your judgment rather than standalone decisions. Over time, as more outcomes accumulate within the same context, signal quality improves.

***

#### A source does not appear in analysis

OpenHuman can only analyze sources you have connected and that fall within the scope of your request.

**Solution:** Ensure the source is connected in your settings and explicitly referenced or selected in your request. OpenHuman does not automatically include all connected sources by default.

***

#### Integrations do not update as expected

Integrations only run when explicitly triggered by your action. OpenHuman does not continuously sync or poll external tools.

**Solution:** If an export fails, check that the integration is still connected and that permissions are valid. Retrying the action after resolving any permission or availability issues usually succeeds.

***

#### Performance feels slow

Response time depends on request complexity, data volume, and current system load.

**Solution:** Large scopes and long histories require more processing. Narrowing scope and intent improves responsiveness. During early rollout phases, performance may vary as capacity is tuned.

***

#### Revoking access and residual data

When access to a source or integration is revoked, OpenHuman immediately stops processing data from that source.

Previously exported outputs (such as summaries written to Notion or Google Sheets) remain where they were written. OpenHuman does not retain message content after revocation.

***

#### When to contact support

If an issue persists after refining scope, checking permissions, and retrying the request, support may be required.

Support is intended for system-level issues, not for disputing interpretations or outcomes. OpenHuman does not adjudicate trust or risk disagreements.

***

#### **What are the system requirements?**

OpenHuman requires macOS. The Lightweight model tier works on machines with 4 GB+ RAM. The recommended Performance tier requires 16 GB+ RAM and Apple Silicon. The app detects your hardware and recommends the best tier automatically.

***

#### **How often does OpenHuman sync my email and Notion?**

Gmail syncs every 15 minutes and Notion every 20 minutes by default. You can adjust these intervals in Settings > Automation & Channels > Cron Jobs.

***

#### **What is the panic stop shortcut?**

Press Cmd+Shift+. to immediately stop all Screen Intelligence capture. Useful if you navigate to sensitive content unexpectedly.

***

#### **What apps are excluded from Screen Intelligence by default?**

1password, keychain, and wallet are in the default denylist. You can add more apps in Settings > Automation & Channels > Screen Intelligence under the Denylist field.
