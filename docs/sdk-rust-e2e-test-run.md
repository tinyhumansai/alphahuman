# Rust SDK E2E Test Run — `example_e2e.rs`

**Run date:** 2026-03-27
**Source:** `neocortex/packages/sdk-rust/tests/example_e2e.rs`
**API base URL:** `https://staging-api.alphahuman.xyz`
**Namespace:** `sdk-rust-e2e`
**Document ID:** `sdk-rust-e2e-doc-single-1774605977640`

---

## What the test does

The file is a standalone end-to-end integration program (not a `#[test]`-annotated unit test). It exercises the `tinyhumansai` Rust SDK against the staging API in six sequential steps:

| Step | Operation | SDK method |
|------|-----------|------------|
| 1 | Insert a memory document | `insert_memory` |
| 2 | Poll ingestion job until complete | `get_ingestion_job` + `wait_for_ingestion_job` |
| 3 | List documents filtered by namespace | `list_documents` |
| 4 | Fetch the specific document | `get_document` |
| 5 | Semantic query over the namespace | `query_memory` |
| 6 | Recall all memory context | `recall_memory` |

The document inserted contains sprint velocity data for four teams (Atlas, Beacon, Comet, Delta).

---

## How to run

The file has its own `async fn main()` (annotated with `#[tokio::main]`), so Cargo's default test harness intercepts it and reports 0 tests. To run it as intended, add `harness = false` to `Cargo.toml` temporarily:

```toml
[[test]]
name = "example_e2e"
harness = false
```

Then:

```bash
cd neocortex/packages/sdk-rust
cargo test --test example_e2e
```

> **Note:** `TINYHUMANS_TOKEN` in the file is intentionally left blank. Populate it with a valid API token before running.

---

## Step-by-step output

### Step 1 — `insertMemory`

**Endpoint:** `POST /memory/insert`

**Request body** (serialized from `InsertMemoryBody`; `priority`, `createdAt`, `updatedAt` omitted because they are `None` and marked `skip_serializing_if`):

```json
{
  "title": "Sprint Dataset - Team Velocity",
  "content": "Sprint snapshot: Team Atlas completed 42 story points with 3 blockers, Team Beacon completed 35 story points with 1 blocker, Team Comet completed 48 story points with 5 blockers, and Team Delta completed 39 story points with 2 blockers. The highest velocity team is Team Comet and the fewest blockers team is Team Beacon.",
  "namespace": "sdk-rust-e2e",
  "sourceType": "doc",
  "metadata": { "source": "example_e2e.rs" },
  "documentId": "sdk-rust-e2e-doc-single-1774605977640"
}
```

**Result:** success
**Job ID:** `a2a1396c-bcf5-4552-afc0-6c822bafd7c6`
**Initial job state:** `pending`

```
InsertMemoryResponse {
    success: true,
    data: InsertMemoryData {
        job_id: Some("a2a1396c-bcf5-4552-afc0-6c822bafd7c6"),
        state: Some("pending"),
        ...
    },
}
```

---

### Step 2 — `getIngestionJob` + `waitForIngestionJob`

**Endpoint:** `GET /memory/ingestion/jobs/{jobId}` (no request body)

**URL:** `GET /memory/ingestion/jobs/a2a1396c-bcf5-4552-afc0-6c822bafd7c6`

`wait_for_ingestion_job` then polls the same endpoint repeatedly (every 1 s, up to 30 s) until the state is not in `{pending, queued, processing, in_progress, started}`.

Initial poll returned state `processing`, so the SDK waited. Job completed successfully.

**Final state:** `completed`
**Completed at:** `2026-03-27T10:06:24.974Z`
**Ingestion latency:** `2.6173 s`

Key stats from the completed job response:

| Metric | Value |
|--------|-------|
| Chunks new | 1 |
| Chunks total | 1 |
| Chunks deduplicated | 0 |
| Entities extracted | 15 |
| Relations extracted | 25 |
| Sections | 1 |
| Source type | `doc` |
| Embedding tokens used | 244 |
| Cost (USD) | $0.00000488 |

Timing breakdown (selected):

| Stage | Seconds |
|-------|---------|
| Chunking | 0.000826 |
| Chunk embedding | 0.2688 |
| Chunk storage | 0.2049 |
| Entity extraction | 0.8131 |
| Entity embedding | 0.0476 |
| Graph structure | 0.2427 |
| Relationship storage | 0.3800 |
| Storage total | 1.2504 |

---

### Step 3 — `listDocuments`

**Endpoint:** `GET /memory/documents?namespace=sdk-rust-e2e&limit=10&offset=0` (no request body)

This run uses the updated `list_documents(ListDocumentsParams { namespace, limit, offset })` signature (new in the local SDK). Passing `namespace` now filters results correctly — previous runs returned an empty array because no namespace filter was applied.

**4 documents returned** (all previous E2E runs in this namespace):

| Document ID | Created at |
|-------------|------------|
| `sdk-rust-e2e-doc-single-1774598994566` | 2026-03-27T08:09:56 |
| `sdk-rust-e2e-doc-single-1774600415507` | 2026-03-27T08:33:37 |
| `sdk-rust-e2e-doc-single-1774604625874` | 2026-03-27T09:43:47 |
| `sdk-rust-e2e-doc-single-1774605977640` | 2026-03-27T10:06:20 ← this run |

All share `namespace: "sdk-rust-e2e"`, `title: "Sprint Dataset - Team Velocity"`, `chunk_count: 1`.

---

### Step 4 — `getDocument`

**Endpoint:** `GET /memory/documents/{documentId}?namespace={namespace}` (no request body)

**URL:** `GET /memory/documents/sdk-rust-e2e-doc-single-1774605977640?namespace=sdk-rust-e2e`

```json
{
  "success": true,
  "data": {
    "document_id": "sdk-rust-e2e-doc-single-1774605977640",
    "namespace": "sdk-rust-e2e",
    "title": "Sprint Dataset - Team Velocity",
    "chunk_count": 1,
    "chunk_ids": [-1427053832764092200],
    "created_at": "2026-03-27T10:06:20.655791+00:00",
    "updated_at": "2026-03-27T10:06:21.964953+00:00",
    "user_id": "69b12a6fd11460481185a040"
  }
}
```

---

### Step 5 — `queryMemory`

**Endpoint:** `POST /memory/query`

**Request body** (serialized from `QueryMemoryParams` with `#[serde(rename_all = "camelCase")]`; `documentIds` and `llmQuery` omitted because they are `None`):

```json
{
  "query": "Which team has the highest velocity and which team has the fewest blockers?",
  "includeReferences": true,
  "namespace": "sdk-rust-e2e",
  "maxChunks": 5.0
}
```

**Result:** 1 chunk returned with score `19.117`

The relevant chunk was retrieved correctly. The LLM context message assembled by the API:

```
## Sources

[1]  Section: Sprint Dataset - Team Velocity
[1]  Sprint snapshot: Team Atlas completed 42 story points with 3 blockers,
     Team Beacon completed 35 story points with 1 blocker, Team Comet
     completed 48 story points with 5 blockers, and Team Delta completed 39
     story points with 2 blockers. The highest velocity team is Team Comet
     and the fewest blockers team is Team Beacon.
```

Top entity mentions extracted from the chunk (by normalized importance):

| Entity | Normalized importance | Count |
|--------|-----------------------|-------|
| THE FEWEST BLOCKERS TEAM | 1.000 | 6 |
| 42 STORY POINTS | 0.842 | 14 |
| TEAM BEACON | 0.486 | 2 |
| TEAM COMET | 0.476 | 2 |
| THE HIGHEST VELOCITY TEAM | 0.440 | 4 |

**Usage:**
- Embedding tokens: 20
- Cost: $0.0000004
- Cached: false

---

### Step 6 — `recallMemoryContext`

**Endpoint:** `POST /memory/recall`

**Request body** (serialized from `RecallMemoryParams` with `#[serde(rename_all = "camelCase")]`):

```json
{
  "namespace": "sdk-rust-e2e",
  "maxChunks": 5.0
}
```

Recall (no query — returns all recent/relevant context) returned the same chunk with a higher score of `31.357` (recall scoring differs from query scoring).

**Counts:** 1 chunk, 0 entities, 0 relations
**Latency:** 2.8183 s
**Usage:** 0 tokens, $0 cost (recall is embedding-free)
**Cached:** false

The LLM context message was identical to step 5.

---

## Changes since previous run

| Area | Previous run | This run |
|------|-------------|----------|
| `step3_list_documents` signature | `list_documents()` — no args | `list_documents(ListDocumentsParams { namespace, limit, offset })` |
| Step 3 result | Empty `documents: []` (no filter) | 4 documents returned (namespace filter working) |
| Job ID | `4b3cc8e2-...` | `a2a1396c-...` |
| Document ID | `sdk-rust-e2e-doc-single-1774600415507` | `sdk-rust-e2e-doc-single-1774605977640` |
| Ingestion latency | 1.7791 s | 2.6173 s |
| Entity extraction time | 0.0117 s | 0.8131 s |

---

## Overall result

```
E2E Rust SDK example completed.
```

All 6 steps passed. The SDK correctly:
- Inserted a document and received a job ID
- Polled and waited for the ingestion job to reach `completed`
- Listed documents filtered by namespace (returning all 4 prior E2E inserts)
- Retrieved the document metadata by ID
- Performed a semantic query and received the correct chunk with entity importance scores
- Recalled memory context with latency and count metadata
