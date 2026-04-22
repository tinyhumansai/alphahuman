---
icon: shield
---

# Privacy & Security

OpenHuman operates on a principle of **zero-knowledge intelligence**. The system is architecturally designed so that your raw data never needs to leave your device. Neocortex compresses your data locally into structured metadata and summaries. Only this compressed output is processed server-side. Your AI has months of context about your entire organizational life. Your raw data has never touched our servers.

***

## Privacy by Design

OpenHuman operates on a principle of **zero retention** for message content. When you make a request, the relevant data is processed to produce an output, and the source content is discarded afterward.

**No long-term raw data storage.** OpenHuman does not maintain a persistent archive of your conversations or screen captures. Context is compressed into knowledge graph structures. Raw text and raw images are not stored.

**No training on your data.** Your conversations, analysis results, and personal information are never used to train AI models or improve systems. Your data serves you and only you.

**OS-level credential storage.** On desktop platforms, OpenHuman uses your operating system's secure keychain to store credentials and sensitive tokens. Credentials are never stored in plain text, browser storage, or application-level databases.

**On-device where possible.** Screen Intelligence, Inline autocomplete, local chat, voice processing, and encryption all happen on your device. Only tasks requiring deeper language processing beyond local model capabilities are handled server-side, under the same privacy constraints.

### Screen Intelligence Privacy

Screen Intelligence captures your screen approximately every 5 seconds and processes captures using the on-device vision model. This is the most privacy-sensitive feature in OpenHuman, and it is designed accordingly.

**Processing is entirely local.** Raw screenshots are analyzed by the local vision model running on your device's hardware. The model produces structured text summaries. The raw screenshot is then discarded.

**Raw screenshots are never stored.** After the on-device model processes a capture, the raw image is deleted. Only the compressed text summary persists in your local Neocortex memory.

**Raw screenshots are never transmitted.** No raw screen image data is ever sent to any server, cloud service, or third party.

**Per-app permissions.** You control which applications Screen Intelligence monitors. You can exclude any application from capture: banking apps, medical portals, password managers, or any application you consider sensitive.

**You can disable it entirely.** Screen Intelligence is optional. You can turn it off at any time from settings.

***

### Inline autocomplete Privacy

Inline autocomplete runs on the local model using cached memory context from Neocortex. No keystroke data, text input, or completion suggestions are sent to any server. The entire feature operates on-device.

***

## Permissions and Access Control

OpenHuman operates on an **explicit-access model**. It only accesses data when you issue a request, and only the data needed to fulfill that request.

### Request-Scoped Access

Access is determined by your requests, not by background monitoring. If you ask OpenHuman to summarize a specific conversation, only that conversation is processed. If you do not reference a source, it is not accessed.

OpenHuman does not silently expand its access over time. There is no progressive permission creep.

### Source-Specific Permissions

Each connected source has its own permission scope:

* **Telegram:** Full capability access, including reading messages, sending messages, managing contacts, and administering groups. All actions are executed only on your explicit instruction. OpenHuman never sends messages, joins groups, or takes any Telegram action autonomously. You remain in control of every action. See Skills & Integrations for the complete capability list.
* **Notion:** Write access to specific workspaces or pages you approve. OpenHuman does not read unrelated documents.
* **Google Sheets:** Write access to specific spreadsheets you approve. OpenHuman does not read unrelated sheets.
* **Slack:** Read access to conversations you reference in your requests. Used to expand your knowledge graph with workplace context.

Integration permissions are limited to what is needed for the specific action you request.

### User-Initiated Actions Only

Every meaningful operation in OpenHuman is user-initiated. Summaries, analysis, trust evaluation, workflow creation, and exports all require a direct request. There is no continuous background processing or monitoring.

The one exception is Screen Intelligence, which runs on a continuous capture cycle when enabled. However, Screen Intelligence operates entirely on-device and does not access any connected platform. It captures what is visible on your screen.

{% hint style="info" %}
OpenHuman is idle with respect to your connected platforms unless you ask it to do something.
{% endhint %}

***

## Revoking Access

You can revoke OpenHuman's access to any connected source at any time.

* Disconnect a source from your settings
* Remove integration permissions
* Stop using the application entirely

Once access is revoked, OpenHuman immediately stops processing data from that source. There is no delayed or cached processing after revocation. Previously exported outputs (such as summaries written to Notion or Google Sheets) remain where they were written, but no new processing occurs.

This makes OpenHuman safe to test, pause, or stop using without residual exposure.

***

## Security

OpenHuman implements security at every layer of the system.

**AES-256-GCM encryption.** All sensitive data stored locally is encrypted using AES-256-GCM. Encryption keys are derived from your credentials and stored in your operating system's secure keychain. Keys never leave the device. Even if server-side infrastructure were compromised, your raw data would remain inaccessible because it was never there.

**Secure credential storage.** On desktop platforms, credentials are stored in the operating system's secure keychain. On web, short-lived tokens and secure session management are used instead.

**Sandboxed skills.** Each skill runs in its own isolated execution environment with enforced memory and resource limits. Skills cannot access each other's data, the host system's file system, or your credentials.

**Encrypted transit.** All communication between the application and OpenHuman's servers uses encrypted connections. No data travels in plain text.

**Short-lived tokens.** Authentication tokens are time-limited and single-use where applicable, reducing the window of exposure if a token is compromised.

***

## How Neocortex enables privacy

Most AI assistants face a tradeoff: more context means more raw data sent to the cloud. Neocortex eliminates this tradeoff.

Because Neocortex compresses millions of tokens into structured knowledge graphs on-device, the server only ever receives compressed metadata. The knowledge graph contains entities, relationships, and temporal patterns. It does not contain your actual messages, emails, or documents.

Compression itself becomes the privacy architecture. The raw data never needs to exist outside your device in the first place.

<figure><img src="../.gitbook/assets/V17 — Privacy Shield@2x.png" alt=""><figcaption></figcaption></figure>

## Trust & Risk Intelligence

OpenHuman includes an intelligence layer designed to help you reason about credibility, information quality, and potential risks across your connected sources.

### What It Does

**Scam and impersonation signals.** OpenHuman can surface behavioral patterns associated with scams, impersonation, or coordinated abuse. These signals are derived from patterns observed across contexts, not from individual message content.

**Contextual dynamic trust.** Trust is represented through aggregated artifacts, historical accuracy of claims, consistency of contributions, peer interaction patterns rather than static scores or universal ratings. Trust is always contextual: credibility in one domain does not automatically transfer to another.

**Advisory, not enforcement.** OpenHuman does not ban users, remove messages, block actions, or enforce moderation decisions. Trust and risk outputs are advisory signals that inform your judgment. You decide how to act on them.

### Scope

Trust and risk intelligence operates at different levels:

* **Personal:** Visible only to you. Your own analysis, trust assessments, and risk alerts.
* **Community:** Aggregated patterns within a group or organization, supporting shared coordination and moderation. Never exposes individual message content.
* **Network:** Anonymized patterns across the broader OpenHuman user base, improving early detection of shared risks like recurring scam vectors.

Information does not move between scopes without abstraction and anonymization.

***

## Shared Environments

When OpenHuman is used in team or community settings, privacy remains user-centric.

OpenHuman does not grant administrators the ability to read private messages through another user's account. Each user's permissions apply only to their own connected sources.

Community-level intelligence is derived from aggregated and anonymized signals, not from direct access to individual message content. Shared insights help teams coordinate effectively without compromising individual privacy.
