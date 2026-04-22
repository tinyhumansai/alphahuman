---
description: >-
  The Intelligence tab shows what OpenHuman knows and how its memory is growing.
  Open it from the bottom navigation bar.
icon: brain-circuit
---

# Intelligence

### System Status

The top of the page shows the current system state (System Idle, Analyzing, etc.) and an "Analyze Now" button. Clicking Analyze Now triggers immediate analysis of your connected skills to extract actionable items, insights, and memory updates.

**Memory** (active): Your knowledge graph, statistics, and ingestion history.

**Subconscious** (coming soon): Background reasoning loop that runs continuously between your prompts, synthesizing thoughts and detecting patterns.

**Dreams** (coming soon): Deeper processing that consolidates and restructures memory during idle periods

### Memory

The Memory section displays six key metrics:

**Storage:** Total size of your local memory store, with file count.

**Documents:** Number of documents ingested into memory.

**Namespaces:** Organizational groupings within your knowledge graph. Different data sources and contexts are separated into namespaces.

**Relations:** Entity relationships extracted and stored in the knowledge graph. These connect people, projects, decisions, and concepts across your data.

**First Memory:** When your first memory was created, with the timestamp of the most recent memory.

**Sessions:** Number of active sessions and total tokens processed.

### Memory Graph

A visual representation of entities and their relationships. This populates as you ingest more data from connected sources. The graph shows how people, projects, topics, and decisions connect across your data.

### Intelligent Insights

Extracted facts, preferences, and relationships from ingested documents. Populates after you run an analysis. Insights include inferred preferences, detected patterns, and factual information about entities in your data.

### Ingestion Activity

A heatmap showing ingestion events over time, similar to a GitHub contribution graph. Displays date range, total events, and peak activity. Helps you understand how actively OpenHuman is processing your data.

### Files & Management

Shows total count of files, namespaces, and documents in your memory store. Expandable for file-level inspection and management.

### Analysis

A search bar lets you search for actionable items across all sources with source filtering. The "Analyze Now" button extracts actionable items from your connected skills. Results appear below, organized by source.
