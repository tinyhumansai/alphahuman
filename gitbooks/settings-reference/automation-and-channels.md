---
icon: nfc-signal
---

# Automation & Channels

### Automation & Channels

Navigate to Settings > Automation & Channels. Configure desktop automation, screen capture, messaging, autocomplete, and scheduled jobs.

#### Accessibility Automation

Desktop permissions, assisted controls, and safety-bound sessions.

**Permissions:** Accessibility (GRANTED/DENIED) and Input Monitoring (GRANTED/DENIED). Buttons: Request Accessibility, Open Input Monitoring, Refresh Status.

**Features:** Three toggles: Screen Monitoring (observe screen for context), Device Control (interact with UI elements), Predictive Input (provide input predictions).

**Session:** Status (Running/Stopped), remaining time, frames captured (ephemeral), panic stop shortcut (Cmd+Shift+.). Start Session, Stop Session, and Analyze Now buttons.

**Vision Summaries:** Processed summaries from the most recent session.

#### Screen Intelligence

Window capture policy, vision summaries, and memory ingestion. See the full [Screen Intelligence](../features/screen-intelligence.md) page for detailed setup.

#### Inline Autocomplete

Predictive text style, app filters, and live completion controls. See the full [Inline Autocomplete](../features/inline-autocomplete.md) page for detailed setup.

#### Messaging Channels

Configure default messaging channel and auth modes.

**Default Messaging Channel:** Choose between Telegram, Discord, or Web. Shows active route status.

**Channel Integrations:** Configure auth modes for Telegram and Discord. Click channel name to open its configuration page.

#### Cron Jobs

Scheduled jobs that keep your data sources in sync.

**Core Cron Jobs:** System-level jobs in the OpenHuman core scheduler database.

**Runtime Skill Cron Settings:** Per-skill sync intervals.

| Skill  | Default Sync Interval |
| ------ | --------------------- |
| Gmail  | Every 15 minutes      |
| Notion | Every 20 minutes      |

Adjust intervals using the dropdown next to each skill. More frequent syncing keeps data fresher but uses more inference budget.

Click "Refresh Cron Jobs" to reload the schedule.
