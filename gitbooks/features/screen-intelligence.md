---
icon: desktop-arrow-down
---

# Screen Intelligence

Screen Intelligence captures your screen, summarizes what you see, and feeds the context into OpenHuman's memory. It runs locally on your device. No screenshots or screen data are sent to any server.

#### How It Works

When a Screen Intelligence session is active, OpenHuman captures your screen at a configurable frame rate (default: 1 frame per second). Each capture is processed locally by the vision model, which generates a text summary of what is on screen. These summaries are ingested into your memory and become part of the context OpenHuman can reference when answering questions.

The result: you can ask OpenHuman "What was I working on this morning?" or "What was on that dashboard I looked at earlier?" and get answers based on your actual screen activity.

#### Setup

Screen Intelligence requires three macOS permissions:

**Screen Recording:** Allows OpenHuman to capture screen contents. Grant this in System Settings > Privacy & Security > Screen Recording.

**Accessibility:** Allows OpenHuman to read window titles and application context. Grant this in System Settings > Privacy & Security > Accessibility.

**Input Monitoring:** Allows OpenHuman to detect keyboard and mouse activity for context. Grant this in System Settings > Privacy & Security > Input Monitoring.

The settings page shows the current status of each permission (GRANTED or DENIED) with buttons to request each one. After granting permissions in System Settings, click "Restart & Refresh Permissions" in OpenHuman so the app picks up the changes.

#### Configuration

**Enabled:** Master toggle to turn Screen Intelligence on or off.

**Mode:** Controls which apps are captured. Default is "All Except Blacklist," which captures everything except apps on your denylist. You can switch to an allowlist mode to capture only specific apps.

**Baseline FPS:** Frames per second for screen capture. Default is 1 (one capture per second). Lower values reduce resource usage, higher values capture more detail.

**Keep Screenshots:** When enabled, saves captured screenshots to your workspace instead of deleting them after processing. Useful for debugging or review, but uses more storage.

**Allowlist:** One rule per line. When in allowlist mode, only apps matching these rules are captured.

**Denylist:** One rule per line. Apps matching these rules are never captured, regardless of mode. Default entries include: 1password, keychain, wallet. Add any sensitive applications here to ensure they are never captured.

Click "Save Screen Intelligence Settings" after making changes.

#### Session Controls

**Start Session:** Begins a capture session. Screen Intelligence starts monitoring your screen according to your policy settings.

**Stop Session:** Ends the current capture session.

**Analyze Now:** Triggers immediate vision processing on the current screen without waiting for the next scheduled capture.

**Panic Stop:** Press Cmd+Shift+. to immediately stop all capture. Useful if you navigate to sensitive content unexpectedly.

The session panel shows: Status (Running/Stopped), Remaining time, Frames captured (ephemeral, not stored permanently unless Keep Screenshots is on), Vision status (idle/processing), Vision queue depth, and Last vision timestamp.

#### Vision Summaries

After processing, vision summaries appear in this section. Each summary is a text description of what was on screen at the time of capture. These summaries are what get ingested into your memory, not the raw screenshots.
