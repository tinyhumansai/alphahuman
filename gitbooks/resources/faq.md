---
icon: question
---

# FAQ & Troubleshooting

## Frequently Asked Questions

#### What is AlphaHuman?

AlphaHuman is a personal AI assistant that connects to your communication platforms and productivity tools. It helps you manage high-volume conversations by summarizing, extracting signals, suggesting responses, and creating structured workflows all from a native app that runs on your device.

***

#### What is Neocortex

Neocortex is AlphaHuman's memory engine. It is a human-like AI memory system that can work with over 1 billion tokens of data. It indexes 10 million tokens in under 10 seconds, costs $1 to index 5 million tokens, and runs on a MacBook Air CPU with no GPU required. Unlike vector databases, Neocortex understands time, entities, and relationships. It builds knowledge graphs and manages memory through tiered compression inspired by how the human brain works. Learn more in Neocortex.

***

#### What does "Big Data AI" mean?

Every AI model today is a prompt engine. You type something, it responds, and the context disappears. AlphaHuman is different. It compresses your entire organizational life, messages, documents, tools, transactions, into a structured knowledge graph that persists and evolves. This is what we mean by Big Data AI: an AI that operates on months of your real data, not just the prompt you typed right now.

**How is AlphaHuman different from ChatGPT, Claude, or Gemini?**

Those models are brilliant at reasoning and generation. But they are stateless. They know nothing about your actual life beyond what you paste into the chat window. AlphaHuman is the context layer that makes those models useful. It compresses your organizational data into structured intelligence that any AI can reason over. Think of it this way: ChatGPT is the brain. AlphaHuman is the memory.

***

**How is AlphaHuman different from other AI memory solutions like Mem0, SuperMemory, or MemGPT?**

Most AI memory solutions use vector databases that retrieve whatever is semantically similar, but similarity alone says nothing about importance. They also cannot support consciousness-like systems or process data accurately at scale beyond 10 million tokens. Neocortex is architecturally different: it uses tiered memory, knowledge graphs, temporal weighting, and semantic deduplication. It processes 10 million tokens in under 10 seconds and supports over 1 billion tokens total. It does this with zero LLM dependency.

***

**Is AlphaHuman open source?**

Yes. AlphaHuman is built on the OpenClaw architecture and licensed under GNU GPL3. The full codebase is available on [GitHub](https://github.com/tinyhumansai/alphahuman). Neocortex benchmarks are also open-sourced. Contributions and feedback are welcomed.

***

**Is AlphaHuman AGI?**

No. AlphaHuman is not AGI, and we do not claim it is. It is a meaningful architectural step closer to AGI, with innovations in memory (Neocortex) and consciousness-like processing (the subconscious system) that go beyond what existing agentic systems offer. But it operates within defined boundaries and requires human judgment.

#### Does AlphaHuman read all my messages?

No. AlphaHuman only processes messages **when you ask it to** and only within the scope of your request.

If you ask it to summarize a specific conversation, it reads that conversation. If you do not reference a source, it is not accessed. There is no background monitoring or continuous scanning.

***

#### Is my data safe?

Yes. AlphaHuman is designed around zero retention of message content. Data is processed to produce an output and then discarded. Your conversations are never stored long-term, and they are never used to train AI models.

On desktop platforms, credentials are stored in your operating system's secure keychain. All communication between the app and AlphaHuman's servers is encrypted.

***

#### Does AlphaHuman store my messages?

Messages are processed only to fulfill your request. They are **not permanently stored or reused** beyond producing the requested output. Derived intelligence like summaries and workflow records may persist, but raw message content does not.

***

#### Can AlphaHuman send messages on my behalf?

No. AlphaHuman does not auto-send messages, post in your groups, or act on your behalf inside any connected platform. If a reply is suggested, you choose whether to use it.

***

#### Who is AlphaHuman for?

AlphaHuman is useful for anyone who:

* Manages high-volume communication across multiple platforms and groups
* Needs to stay on top of decisions, action items, and context without reading everything
* Works in distributed teams, communities, or coordination-heavy environments
* Wants structured outputs from conversations, exportable to tools like Notion or Google Sheets

You do not need to be technical to use it.

***

#### What platforms does AlphaHuman support?

AlphaHuman runs natively on **macOS, Windows, Linux, Android, and iOS**, with a **web version** for browser access. Your account and settings sync across all platforms.

***

#### What integrations are available?

AlphaHuman currently integrates with **Telegram** (read and analyze conversations), **Notion** (export structured outputs), and **Google Sheets** (export tabular data and reports). More integrations are planned.

***

#### How much does AlphaHuman cost?

AlphaHuman offers individual and team plans with core analysis included. Deeper features are available in higher tiers. A credit system provides usage-metered flexibility, and credits can be earned through referrals.

See [Pricing](../product/pricing.md) for details.

***

## Troubleshooting

#### Summaries feel incomplete or too broad

If a summary feels incomplete, the most common cause is overly broad scope. When a request spans many conversations, long time ranges, or high-volume groups, AlphaHuman prioritizes signal over detail.

**Solution:** Narrow the request to a specific conversation, time window, or intent. AlphaHuman performs best when it knows what kind of outcome you are looking for.

***

#### Important context seems missing

AlphaHuman only processes the data required to fulfill a request. If relevant context exists outside the scope of that request, it will not be included.

**Solution:** Expand the scope explicitly by referencing additional conversations or extending the time range.

***

#### Outputs feel incorrect or misinterpreted

AlphaHuman interprets conversations probabilistically. Tone, sarcasm, and informal language can be misread, especially in fast-moving or meme-heavy discussions.

**Solution:** Refine the request or re-run analysis with a narrower scope. Outputs should be treated as assistance rather than ground truth.

***

#### Trust or risk signals feel inaccurate

Trust and risk intelligence is indicative, not authoritative. Signals may lag real-world changes or surface false positives.

**Solution:** Use these signals as inputs into your judgment rather than standalone decisions. Over time, as more outcomes accumulate within the same context, signal quality improves.

***

#### A source does not appear in analysis

AlphaHuman can only analyze sources you have connected and that fall within the scope of your request.

**Solution:** Ensure the source is connected in your settings and explicitly referenced or selected in your request. AlphaHuman does not automatically include all connected sources by default.

***

#### Integrations do not update as expected

Integrations only run when explicitly triggered by your action. AlphaHuman does not continuously sync or poll external tools.

**Solution:** If an export fails, check that the integration is still connected and that permissions are valid. Retrying the action after resolving any permission or availability issues usually succeeds.

***

#### Performance feels slow

Response time depends on request complexity, data volume, and current system load.

**Solution:** Large scopes and long histories require more processing. Narrowing scope and intent improves responsiveness. During early rollout phases, performance may vary as capacity is tuned.

***

#### Revoking access and residual data

When access to a source or integration is revoked, AlphaHuman immediately stops processing data from that source.

Previously exported outputs (such as summaries written to Notion or Google Sheets) remain where they were written. AlphaHuman does not retain message content after revocation.

***

#### When to contact support

If an issue persists after refining scope, checking permissions, and retrying the request, support may be required.

Support is intended for system-level issues, not for disputing interpretations or outcomes. AlphaHuman does not adjudicate trust or risk disagreements.
