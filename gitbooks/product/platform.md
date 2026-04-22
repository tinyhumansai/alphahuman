---
icon: layer-plus
---

# Platform & Availability

OpenHuman is a native application that runs on six platforms from a single codebase. It is not a web-only tool, browser extension, or Electron wrapper. It is built for performance, security, and a small footprint on every device you use.

***

## Six Platforms, One Experience

OpenHuman compiles to native binaries for each supported platform:

| Platform    | Architectures        | Distribution   |
| ----------- | -------------------- | -------------- |
| **macOS**   | Intel, Apple Silicon | .dmg installer |
| **Windows** | x64, ARM64           | .msi installer |
| **Linux**   | x64, ARM64           | AppImage, .deb |
| **Android** | ARM                  | .apk package   |
| **iOS**     | ARM64                | App Store      |
| **Web**     | Any browser          | Direct access  |

Your account, connected sources, preferences, and settings sync across all platforms. You can start a request on your desktop and review the output on your phone.

***

## Desktop-First Capabilities

The desktop app is the primary OpenHuman experience. It runs local AI models on your machine and provides capabilities that are only available on desktop:

**Screen Intelligence** captures your screen approximately every 5 seconds, processes captures through the on-device vision model, and produces structured context summaries. This feature requires the desktop app because it needs access to your screen and local compute for real-time processing. Per-app permissions let you control which applications are monitored.

**Inline Autocomplete** uses your Neocortex memory context combined with the local model to generate relevant text completions on any input surface. This runs entirely on-device and is available on desktop platforms.

**Local model inference** for chat, vision, speech-to-text, and text-to-speech runs on your desktop hardware. No GPU is required, though GPU acceleration is used when available.

Mobile and web versions provide full access to your Neocortex memory, connected source intelligence, and subconscious insights, but do not run local models or Screen Intelligence.

***

## Why Native Matters

OpenHuman is built as a native application rather than a web wrapper for three reasons.

**Small footprint.** The app is lightweight. A fraction of the size of typical communication tools. It starts in under a second and uses minimal memory, so it stays out of the way when running alongside other demanding applications.

**Fast startup.** There is no browser engine to initialize. The app launches quickly and is ready to accept requests immediately.

**OS-level security.** On desktop platforms, OpenHuman stores credentials in your operating system's secure keychain (macOS Keychain, Windows Credential Manager, Linux Secret Service). Sensitive data never sits in browser storage or plain text files.

***

## Architecture at a Glance

OpenHuman operates across three layers:

**Application layer.** The native app on your device handles the interface, user input, local state, credential management, local model inference, Screen Intelligence, **Inline Autocomplete**, and skill execution. This layer is responsible for everything you see and interact with, and for all on-device AI processing.

**Intelligence layer.** OpenHuman's analysis, coordination, and intelligence systems run as a secure backend service. When a request requires deeper language processing, it is handled here. This layer is operated and maintained by OpenHuman.

**External services.** Connected tools and platforms:Telegram, Notion, Google Sheets, and others are accessed only when you explicitly request it. OpenHuman acts as a bridge between your sources and the intelligence layer, not as a replacement for any of them.

{% hint style="info" %}
The intelligence layer is not part of the client application. It performs analysis, coordination, and trust scoring separately from the frontend.
{% endhint %}

***

## Real-Time Communication

OpenHuman maintains a persistent connection between the application and the intelligence layer. This means responses arrive in real time as they are generated. You see outputs streaming, not loading.

The connection is designed for resilience. If the network drops, OpenHuman reconnects automatically with progressive backoff. There is no manual reconnection process.

***

## Offline Behavior

OpenHuman's local state persists on your device. Your preferences, settings, and connected source configurations remain available even when you are offline.

On desktop, local model capabilities continue to function offline. Screen Intelligence continues capturing and processing locally. **Inline Autocomplete** continues generating suggestions from cached memory context. Chat with the local model works without a network connection.

Full analysis and intelligence features that require the server-side intelligence layer need a network connection. When connectivity is restored, the app resumes normal operation without requiring you to re-authenticate or reconfigure.
