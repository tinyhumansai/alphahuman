---
icon: play
---

# Getting Started

This section walks you through setting up OpenHuman and running your first request.

OpenHuman is open source under the GNU GPL3 license. The codebase is available at [github.com/tinyhumansai/openhuman](https://github.com/tinyhumansai/openhuman). You can self-host, contribute, or simply use the hosted version.

***

#### System Requirements

OpenHuman is a desktop application for macOS. Download it from [openhuman.ai](https://openhuman.ai).

OpenHuman runs a local AI model on your machine. The experience scales with your hardware.

| Model Tier                | Download Size | Minimum RAM | GPU Required              | Model                  |
| ------------------------- | ------------- | ----------- | ------------------------- | ---------------------- |
| Lightweight               | \~1 GB        | 4 GB        | No                        | gemma3:1b-it-q4\_0     |
| Balanced                  | \~3 GB        | 8 GB        | No                        | gemma3:4b-it-qat       |
| Performance (Recommended) | \~8 GB        | 16 GB       | Yes (Apple Silicon Metal) | gemma3:12b-it-q4\_K\_M |

The app detects your system specs automatically and recommends the best tier. You can change the model tier at any time from Settings > AI & Skills > Local AI Model.

#### macOS Permissions

Some features require macOS permissions. You will be prompted during setup, but you can also configure them later in Settings > Automation & Channels.

**Screen Intelligence** requires: Screen Recording, Accessibility, and Input Monitoring permissions.

**Inline Autocomplete** requires: Accessibility and Input Monitoring permissions.

After granting permissions in System Settings, return to OpenHuman and click "Restart & Refresh Permissions" so the app picks up the grants.

## Download & Install

Download the OpenHuman desktop app from the official website at [tryopenhuman.com](https://tryopenhuman.com). The desktop app runs local AI models on your machine for private screen analysis, inline autocomplete, and chat.

OpenHuman runs natively on

* **macOS** (Intel and Apple Silicon)
* **Windows** (x64 and ARM64)
* **Linux** (x64 and ARM64: AppImage and .deb)
* **Android**
* &#x20;**iOS**
* **Web** (any modern browser).

The desktop app is the primary experience. It has a small footprint, starts fast, runs local AI models, and uses your operating system's secure credential storage. Mobile and web versions provide companion access to your account and intelligence layer when you are away from your desktop.

***

## Create Your Account

When you first open OpenHuman, you will be asked to sign in. Multiple sign-in options are available, including social login providers.

After signing in, you may be placed on a waitlist depending on rollout status. The waitlist is used to manage access during launch and early scaling.

{% hint style="info" %}
**No permanent lock-in.** Creating an account does not grant OpenHuman ongoing access to anything. All processing still requires explicit actions from you later.
{% endhint %}

***

### Set Up Screen Intelligence

Screen Intelligence is one of OpenHuman's core capabilities. It captures your screen activity locally and builds context from what you see across your applications throughout the day.

**How it works:** The desktop app takes screenshots approximately every 5 seconds and processes them using the on-device vision model (Gemma 3). Each capture is analyzed into a structured summary: which app you were using, what content was on screen, and what you were doing. The raw screenshots are processed and discarded. Only the compressed summaries persist in your local Neocortex memory.

**Per-app permissions:** You control which applications Screen Intelligence monitors. When you first enable it, you can review and adjust which apps are included. Exclude any application you want to keep private, such as banking, medical, or personal apps. You can change these permissions at any time from settings.

**What gets stored:** Structured summaries only. "User was reviewing a spreadsheet in Google Sheets showing Q3 revenue figures" rather than a pixel-for-pixel copy of your screen. Raw screenshots never leave your device and are not stored after processing.

***

### Enable Inline Autocomplete

Inline autocomplete uses your Neocortex memory combined with the local model to suggest relevant text completions on any input surface across your system.

Because it draws on your accumulated context, suggestions reflect your actual terminology, projects, contacts, and patterns. This is different from generic inline autocomplete: it knows what you have been working on, who you have been talking to, and what is currently relevant.

Inline autocomplete runs entirely on your local model. No keystroke data or text input is sent to any server. You can enable or disable it from settings at any time.

***

## Connect Your First Source

OpenHuman works by connecting to your existing tools and platforms. Each connection expands your knowledge graph, giving Neocortex more data to compress and reason over. You choose what to connect, and you can revoke access at any time.

**Telegram:** Connect your Telegram account to analyze conversations, extract signals, and generate summaries across your chats and groups. OpenHuman supports the full range of Telegram capabilities: chat management, message search, contact management, group and channel administration, reactions, polls, inline buttons, and more.

**Notion:** Connect Notion to export structured outputs like summaries, action items, decisions, and workflow records into your workspace.

**Google Sheets:** Connect Google Sheets to log reports, track actions, or export tabular data.

**Slack:** Connect Slack to extend your knowledge graph across workplace conversations.

All integrations are optional. You can start with Screen Intelligence alone, without connecting any external source, and add integrations later. The more sources you connect, the more powerful the intelligence becomes. Each connection is independently revocable.

> **Planned integrations:** Discord, iMessage, WhatsApp, Gmail, Google Calendar, and Web3 wallets are on the roadmap.

***

## Run Your First Request

Once Screen Intelligence is running or a source is connected, you can start asking OpenHuman questions.

Try prompts like:

**Screen-aware queries:**

* "What was I working on this morning?"
* "Summarize what I saw in that meeting"
* "What was on that dashboard I was looking at earlier?"

**Messaging queries:**

* "Summarize what I missed today"
* "What are the key decisions from this week?"
* "Extract action items from my recent conversations"
* "What topics are trending across my groups?"

**Cross-source queries:**

* "Connect what my team discussed in Slack with what I was reviewing on screen"
* "What did Sarah say about the project I was working on yesterday?"

OpenHuman processes only the data needed to answer your request, produces an output, and presents it for your review. If the output feels too broad or too shallow, narrow the scope. Specify a particular conversation, time range, or intent.

***

## Explore Skills & Integrations

After your first request, explore what else OpenHuman can do:

* **Skills** extend the assistant's capabilities, fetching external data, running scheduled tasks, processing information, and writing outputs to connected tools
* **Integrations** let you push structured results to Notion, Google Sheets, and other tools
* **Workflows** turn conversation decisions into trackable actions you can follow through on

Learn more in [Skills & Integrations](../product/skills-and-integrations.md).

***

#### Join the Community

OpenHuman is in early alpha. Feedback and contributions make a real difference at this stage.

Join the Discord community to connect with other users, share feedback, report issues, and contribute to the project. Early users get free usage as part of the alpha program.

**GitHub:** [github.com/tinyhumansai/openhuman](https://github.com/tinyhumansai/openhuman)
