//! Local vector store backed by SQLite.
//!
//! Provides a self-contained vector database for storing, searching, and
//! managing text embeddings. Uses SQLite for persistence and brute-force
//! cosine similarity for retrieval (fast enough for on-device workloads up
//! to ~100K vectors).
//!
//! # Usage
//!
//! ```ignore
//! let embedder = Arc::new(OllamaEmbedding::default());
//! let store = VectorStore::open(db_path, embedder)?;
//!
//! store.insert("doc-1", "notes", "The quick brown fox", json!({})).await?;
//! let results = store.search("notes", "fast animal", 5).await?;
//! ```

use std::path::Path;
use std::sync::Arc;

use parking_lot::Mutex;
use rusqlite::Connection;

use super::EmbeddingProvider;

/// SQL to create the vector store schema.
const INIT_SQL: &str = "
    PRAGMA journal_mode = WAL;
    PRAGMA synchronous = NORMAL;

    CREATE TABLE IF NOT EXISTS vectors (
        id         TEXT    NOT NULL,
        namespace  TEXT    NOT NULL,
        text       TEXT    NOT NULL,
        embedding  BLOB    NOT NULL,
        metadata   TEXT    NOT NULL DEFAULT '{}',
        created_at REAL    NOT NULL,
        updated_at REAL    NOT NULL,
        PRIMARY KEY (namespace, id)
    );
    CREATE INDEX IF NOT EXISTS idx_vectors_ns ON vectors(namespace);
";

/// A single search result from the vector store.
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// The stored document ID.
    pub id: String,
    /// The namespace.
    pub namespace: String,
    /// The original text.
    pub text: String,
    /// Cosine similarity score (0.0 – 1.0).
    pub score: f64,
    /// Arbitrary JSON metadata attached at insert time.
    pub metadata: serde_json::Value,
}

/// SQLite-backed local vector store.
///
/// Thread-safe: the inner connection is behind a `parking_lot::Mutex` and
/// the struct is `Send + Sync`. Embedding calls are async and run through
/// the configured [`EmbeddingProvider`].
pub struct VectorStore {
    conn: Arc<Mutex<Connection>>,
    embedder: Arc<dyn EmbeddingProvider>,
}

impl VectorStore {
    /// Opens (or creates) a vector store at the given SQLite database path.
    pub fn open(db_path: &Path, embedder: Arc<dyn EmbeddingProvider>) -> anyhow::Result<Self> {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(db_path)?;
        conn.execute_batch(INIT_SQL)?;

        tracing::debug!(
            target: "embeddings.store",
            "[vector-store] opened at {}, embedder={}",
            db_path.display(),
            embedder.name()
        );

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            embedder,
        })
    }

    /// Opens an in-memory vector store (useful for tests).
    pub fn open_in_memory(embedder: Arc<dyn EmbeddingProvider>) -> anyhow::Result<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch(INIT_SQL)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            embedder,
        })
    }

    /// Returns a reference to the embedding provider.
    pub fn embedder(&self) -> &dyn EmbeddingProvider {
        self.embedder.as_ref()
    }

    // ── Write operations ─────────────────────────────────────

    /// Inserts or updates a text entry. The text is embedded automatically.
    ///
    /// If an entry with the same `(namespace, id)` already exists it is replaced.
    pub async fn insert(
        &self,
        id: &str,
        namespace: &str,
        text: &str,
        metadata: serde_json::Value,
    ) -> anyhow::Result<()> {
        let embedding = self.embedder.embed_one(text).await?;
        self.insert_with_vector(id, namespace, text, &embedding, metadata)
    }

    /// Inserts with a pre-computed embedding vector (skips the embed call).
    pub fn insert_with_vector(
        &self,
        id: &str,
        namespace: &str,
        text: &str,
        embedding: &[f32],
        metadata: serde_json::Value,
    ) -> anyhow::Result<()> {
        let blob = vec_to_bytes(embedding);
        let meta_str = serde_json::to_string(&metadata)?;
        let now = now_ts();

        let conn = self.conn.lock();
        conn.execute(
            "INSERT OR REPLACE INTO vectors (id, namespace, text, embedding, metadata, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![id, namespace, text, blob, meta_str, now, now],
        )?;

        tracing::trace!(
            target: "embeddings.store",
            "[vector-store] inserted id={id} ns={namespace} dims={}",
            embedding.len()
        );

        Ok(())
    }

    /// Bulk-insert multiple entries. Each text is embedded automatically.
    pub async fn insert_batch(
        &self,
        namespace: &str,
        entries: &[(&str, &str, serde_json::Value)], // (id, text, metadata)
    ) -> anyhow::Result<()> {
        if entries.is_empty() {
            return Ok(());
        }

        let texts: Vec<&str> = entries.iter().map(|(_, text, _)| *text).collect();
        let embeddings = self.embedder.embed(&texts).await?;

        if embeddings.len() != entries.len() {
            anyhow::bail!(
                "embedding count mismatch: got {} embeddings for {} entries",
                embeddings.len(),
                entries.len()
            );
        }

        let now = now_ts();
        let conn = self.conn.lock();
        let tx = conn.unchecked_transaction()?;

        for ((id, text, metadata), embedding) in entries.iter().zip(embeddings.iter()) {
            let blob = vec_to_bytes(embedding);
            let meta_str = serde_json::to_string(metadata)?;
            tx.execute(
                "INSERT OR REPLACE INTO vectors (id, namespace, text, embedding, metadata, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                rusqlite::params![id, namespace, text, blob, meta_str, now, now],
            )?;
        }

        tx.commit()?;

        tracing::debug!(
            target: "embeddings.store",
            "[vector-store] batch inserted {} entries in ns={namespace}",
            entries.len()
        );

        Ok(())
    }

    // ── Search ───────────────────────────────────────────────

    /// Searches for the `limit` most similar entries to `query` within a namespace.
    ///
    /// The query is embedded via the configured provider and compared against
    /// all stored vectors using cosine similarity.
    pub async fn search(
        &self,
        namespace: &str,
        query: &str,
        limit: usize,
    ) -> anyhow::Result<Vec<SearchResult>> {
        let query_vec = self.embedder.embed_one(query).await?;
        self.search_by_vector(namespace, &query_vec, limit)
    }

    /// Searches using a pre-computed query vector.
    pub fn search_by_vector(
        &self,
        namespace: &str,
        query_vec: &[f32],
        limit: usize,
    ) -> anyhow::Result<Vec<SearchResult>> {
        if limit == 0 {
            return Ok(Vec::new());
        }

        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, namespace, text, embedding, metadata FROM vectors WHERE namespace = ?1",
        )?;

        let mut scored: Vec<SearchResult> = stmt
            .query_map(rusqlite::params![namespace], |row| {
                let id: String = row.get(0)?;
                let ns: String = row.get(1)?;
                let text: String = row.get(2)?;
                let blob: Vec<u8> = row.get(3)?;
                let meta_str: String = row.get(4)?;
                Ok((id, ns, text, blob, meta_str))
            })?
            .filter_map(|r| r.ok())
            .map(|(id, ns, text, blob, meta_str)| {
                let stored_vec = bytes_to_vec(&blob);
                let score = cosine_similarity(query_vec, &stored_vec);
                let metadata =
                    serde_json::from_str(&meta_str).unwrap_or(serde_json::Value::Null);
                SearchResult {
                    id,
                    namespace: ns,
                    text,
                    score,
                    metadata,
                }
            })
            .collect();

        // Sort descending by score.
        scored.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(limit);

        Ok(scored)
    }

    // ── Delete / management ──────────────────────────────────

    /// Deletes a single entry by ID within a namespace.
    ///
    /// Returns `true` if a row was actually deleted.
    pub fn delete(&self, namespace: &str, id: &str) -> anyhow::Result<bool> {
        let conn = self.conn.lock();
        let affected =
            conn.execute(
                "DELETE FROM vectors WHERE namespace = ?1 AND id = ?2",
                rusqlite::params![namespace, id],
            )?;
        Ok(affected > 0)
    }

    /// Deletes all entries in a namespace.
    ///
    /// Returns the number of deleted rows.
    pub fn clear_namespace(&self, namespace: &str) -> anyhow::Result<usize> {
        let conn = self.conn.lock();
        let affected = conn.execute(
            "DELETE FROM vectors WHERE namespace = ?1",
            rusqlite::params![namespace],
        )?;

        tracing::debug!(
            target: "embeddings.store",
            "[vector-store] cleared namespace={namespace}, deleted={affected}"
        );

        Ok(affected)
    }

    /// Returns the number of entries in a namespace (or all if `None`).
    pub fn count(&self, namespace: Option<&str>) -> anyhow::Result<usize> {
        let conn = self.conn.lock();
        let count: usize = match namespace {
            Some(ns) => conn.query_row(
                "SELECT COUNT(*) FROM vectors WHERE namespace = ?1",
                rusqlite::params![ns],
                |row| row.get(0),
            )?,
            None => conn.query_row("SELECT COUNT(*) FROM vectors", [], |row| row.get(0))?,
        };
        Ok(count)
    }

    /// Lists all distinct namespaces.
    pub fn list_namespaces(&self) -> anyhow::Result<Vec<String>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare("SELECT DISTINCT namespace FROM vectors ORDER BY namespace")?;
        let namespaces: Vec<String> = stmt
            .query_map([], |row| row.get(0))?
            .filter_map(|r| r.ok())
            .collect();
        Ok(namespaces)
    }
}

// ── Vector math utilities ────────────────────────────────────

/// Serializes a float vector to little-endian bytes for SQLite BLOB storage.
pub fn vec_to_bytes(v: &[f32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(v.len() * 4);
    for &f in v {
        bytes.extend_from_slice(&f.to_le_bytes());
    }
    bytes
}

/// Deserializes little-endian bytes back to a float vector.
pub fn bytes_to_vec(bytes: &[u8]) -> Vec<f32> {
    bytes
        .chunks_exact(4)
        .map(|chunk| {
            let arr: [u8; 4] = chunk.try_into().unwrap_or([0; 4]);
            f32::from_le_bytes(arr)
        })
        .collect()
}

/// Computes cosine similarity between two vectors. Returns 0.0 for
/// mismatched lengths, empty vectors, or zero-magnitude vectors.
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f64 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let mut dot = 0.0_f64;
    let mut norm_a = 0.0_f64;
    let mut norm_b = 0.0_f64;
    for (x, y) in a.iter().zip(b.iter()) {
        let x = f64::from(*x);
        let y = f64::from(*y);
        dot += x * y;
        norm_a += x * x;
        norm_b += y * y;
    }
    let denom = norm_a.sqrt() * norm_b.sqrt();
    if denom <= f64::EPSILON {
        return 0.0;
    }
    (dot / denom).clamp(0.0, 1.0)
}

fn now_ts() -> f64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0)
}

// ── Tests ────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// A test embedding provider that returns deterministic vectors.
    /// Each text is mapped to a simple vector based on its hash for
    /// reproducibility. Dimensions are configurable.
    struct FakeEmbedding {
        dims: usize,
    }

    #[async_trait::async_trait]
    impl EmbeddingProvider for FakeEmbedding {
        fn name(&self) -> &str {
            "fake"
        }
        fn dimensions(&self) -> usize {
            self.dims
        }
        async fn embed(&self, texts: &[&str]) -> anyhow::Result<Vec<Vec<f32>>> {
            Ok(texts.iter().map(|t| text_to_vec(t, self.dims)).collect())
        }
    }

    /// Deterministic text → vector for tests. Uses a simple hash-based
    /// approach so similar texts get somewhat similar vectors.
    fn text_to_vec(text: &str, dims: usize) -> Vec<f32> {
        let mut vec = vec![0.0_f32; dims];
        for (i, byte) in text.bytes().enumerate() {
            vec[i % dims] += byte as f32 / 255.0;
        }
        // L2 normalize
        let norm: f32 = vec.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for x in &mut vec {
                *x /= norm;
            }
        }
        vec
    }

    fn fake_store(dims: usize) -> VectorStore {
        VectorStore::open_in_memory(Arc::new(FakeEmbedding { dims })).unwrap()
    }

    // ── vec_to_bytes / bytes_to_vec ─────────────────────────

    #[test]
    fn roundtrip_vec_bytes() {
        let original = vec![1.0_f32, -2.5, 3.14, 0.0, f32::MAX, f32::MIN];
        let bytes = vec_to_bytes(&original);
        assert_eq!(bytes.len(), original.len() * 4);
        let restored = bytes_to_vec(&bytes);
        assert_eq!(original, restored);
    }

    #[test]
    fn empty_vec_roundtrip() {
        let bytes = vec_to_bytes(&[]);
        assert!(bytes.is_empty());
        let restored = bytes_to_vec(&bytes);
        assert!(restored.is_empty());
    }

    #[test]
    fn bytes_to_vec_truncates_partial_bytes() {
        // 5 bytes → only 1 f32 (4 bytes), last byte is ignored.
        let bytes = vec![0u8; 5];
        let result = bytes_to_vec(&bytes);
        assert_eq!(result.len(), 1);
    }

    // ── cosine_similarity ───────────────────────────────────

    #[test]
    fn cosine_identical_vectors() {
        let v = vec![1.0_f32, 2.0, 3.0];
        let score = cosine_similarity(&v, &v);
        assert!((score - 1.0).abs() < 1e-6, "identical vectors: {score}");
    }

    #[test]
    fn cosine_orthogonal_vectors() {
        let a = vec![1.0_f32, 0.0];
        let b = vec![0.0_f32, 1.0];
        let score = cosine_similarity(&a, &b);
        assert!(score.abs() < 1e-6, "orthogonal vectors: {score}");
    }

    #[test]
    fn cosine_opposite_vectors() {
        let a = vec![1.0_f32, 0.0];
        let b = vec![-1.0_f32, 0.0];
        let score = cosine_similarity(&a, &b);
        // Clamped to 0.0 since our impl clamps negative similarities.
        assert!(score.abs() < 1e-6, "opposite vectors: {score}");
    }

    #[test]
    fn cosine_mismatched_lengths() {
        let a = vec![1.0_f32, 2.0];
        let b = vec![1.0_f32, 2.0, 3.0];
        assert_eq!(cosine_similarity(&a, &b), 0.0);
    }

    #[test]
    fn cosine_empty_vectors() {
        assert_eq!(cosine_similarity(&[], &[]), 0.0);
    }

    #[test]
    fn cosine_zero_vector() {
        let a = vec![0.0_f32, 0.0];
        let b = vec![1.0_f32, 0.0];
        assert_eq!(cosine_similarity(&a, &b), 0.0);
    }

    #[test]
    fn cosine_similar_vectors_high_score() {
        let a = vec![1.0_f32, 2.0, 3.0];
        let b = vec![1.1_f32, 2.1, 3.1];
        let score = cosine_similarity(&a, &b);
        assert!(score > 0.99, "similar vectors should score high: {score}");
    }

    // ── VectorStore: open / create ──────────────────────────

    #[test]
    fn open_in_memory() {
        let store = fake_store(3);
        assert_eq!(store.count(None).unwrap(), 0);
    }

    #[test]
    fn open_on_disk() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("sub/dir/vectors.db");
        let embedder: Arc<dyn EmbeddingProvider> = Arc::new(FakeEmbedding { dims: 3 });
        let store = VectorStore::open(&db_path, embedder).unwrap();
        assert_eq!(store.count(None).unwrap(), 0);
        assert!(db_path.exists());
    }

    #[test]
    fn embedder_accessor() {
        let store = fake_store(3);
        assert_eq!(store.embedder().name(), "fake");
    }

    // ── insert + count ──────────────────────────────────────

    #[tokio::test]
    async fn insert_and_count() {
        let store = fake_store(4);
        store.insert("a", "ns1", "hello", json!({})).await.unwrap();
        store.insert("b", "ns1", "world", json!({})).await.unwrap();
        store
            .insert("c", "ns2", "other", json!({}))
            .await
            .unwrap();

        assert_eq!(store.count(Some("ns1")).unwrap(), 2);
        assert_eq!(store.count(Some("ns2")).unwrap(), 1);
        assert_eq!(store.count(None).unwrap(), 3);
    }

    #[tokio::test]
    async fn insert_upsert_replaces() {
        let store = fake_store(4);
        store
            .insert("a", "ns", "original", json!({"v": 1}))
            .await
            .unwrap();
        store
            .insert("a", "ns", "updated", json!({"v": 2}))
            .await
            .unwrap();

        assert_eq!(store.count(Some("ns")).unwrap(), 1);

        let results = store.search_by_vector("ns", &text_to_vec("updated", 4), 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].text, "updated");
        assert_eq!(results[0].metadata["v"], 2);
    }

    // ── insert_with_vector ──────────────────────────────────

    #[test]
    fn insert_with_vector_sync() {
        let store = fake_store(3);
        store
            .insert_with_vector("id1", "ns", "text", &[1.0, 0.0, 0.0], json!({"k": "v"}))
            .unwrap();
        assert_eq!(store.count(Some("ns")).unwrap(), 1);
    }

    // ── insert_batch ────────────────────────────────────────

    #[tokio::test]
    async fn insert_batch_multiple() {
        let store = fake_store(4);
        let entries: Vec<(&str, &str, serde_json::Value)> = vec![
            ("a", "alpha", json!({})),
            ("b", "beta", json!({})),
            ("c", "gamma", json!({})),
        ];
        store.insert_batch("ns", &entries).await.unwrap();
        assert_eq!(store.count(Some("ns")).unwrap(), 3);
    }

    #[tokio::test]
    async fn insert_batch_empty() {
        let store = fake_store(4);
        store.insert_batch("ns", &[]).await.unwrap();
        assert_eq!(store.count(None).unwrap(), 0);
    }

    // ── search ──────────────────────────────────────────────

    #[tokio::test]
    async fn search_returns_ranked_results() {
        let store = fake_store(8);
        store
            .insert("a", "ns", "the quick brown fox", json!({}))
            .await
            .unwrap();
        store
            .insert("b", "ns", "a lazy dog sleeps", json!({}))
            .await
            .unwrap();
        store
            .insert("c", "ns", "the quick brown fox jumps", json!({}))
            .await
            .unwrap();

        let results = store.search("ns", "the quick brown fox", 2).await.unwrap();
        assert_eq!(results.len(), 2);
        // First result should be the exact or closest match.
        assert!(results[0].score >= results[1].score);
        // Scores should be between 0 and 1.
        for r in &results {
            assert!(r.score >= 0.0 && r.score <= 1.0, "score: {}", r.score);
        }
    }

    #[tokio::test]
    async fn search_respects_limit() {
        let store = fake_store(4);
        for i in 0..10 {
            store
                .insert(&format!("id-{i}"), "ns", &format!("text {i}"), json!({}))
                .await
                .unwrap();
        }

        let results = store.search("ns", "text", 3).await.unwrap();
        assert_eq!(results.len(), 3);
    }

    #[tokio::test]
    async fn search_empty_namespace() {
        let store = fake_store(4);
        let results = store.search("empty", "query", 10).await.unwrap();
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn search_namespace_isolation() {
        let store = fake_store(4);
        store
            .insert("a", "ns1", "hello", json!({}))
            .await
            .unwrap();
        store
            .insert("b", "ns2", "hello", json!({}))
            .await
            .unwrap();

        let r1 = store.search("ns1", "hello", 10).await.unwrap();
        let r2 = store.search("ns2", "hello", 10).await.unwrap();
        assert_eq!(r1.len(), 1);
        assert_eq!(r2.len(), 1);
        assert_eq!(r1[0].id, "a");
        assert_eq!(r2[0].id, "b");
    }

    // ── search_by_vector ────────────────────────────────────

    #[test]
    fn search_by_vector_limit_zero() {
        let store = fake_store(3);
        store
            .insert_with_vector("a", "ns", "t", &[1.0, 0.0, 0.0], json!({}))
            .unwrap();
        let results = store.search_by_vector("ns", &[1.0, 0.0, 0.0], 0).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn search_by_vector_scores_correct() {
        let store = fake_store(3);
        // Insert two orthogonal vectors.
        store
            .insert_with_vector("x-axis", "ns", "x", &[1.0, 0.0, 0.0], json!({}))
            .unwrap();
        store
            .insert_with_vector("y-axis", "ns", "y", &[0.0, 1.0, 0.0], json!({}))
            .unwrap();

        // Query along x-axis.
        let results = store.search_by_vector("ns", &[1.0, 0.0, 0.0], 2).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].id, "x-axis");
        assert!((results[0].score - 1.0).abs() < 1e-6);
        assert!(results[1].score < 1e-6); // orthogonal → 0
    }

    #[test]
    fn search_by_vector_preserves_metadata() {
        let store = fake_store(2);
        store
            .insert_with_vector("a", "ns", "t", &[1.0, 0.0], json!({"key": "value"}))
            .unwrap();
        let results = store.search_by_vector("ns", &[1.0, 0.0], 1).unwrap();
        assert_eq!(results[0].metadata["key"], "value");
    }

    // ── delete ──────────────────────────────────────────────

    #[tokio::test]
    async fn delete_existing() {
        let store = fake_store(4);
        store.insert("a", "ns", "text", json!({})).await.unwrap();
        assert!(store.delete("ns", "a").unwrap());
        assert_eq!(store.count(Some("ns")).unwrap(), 0);
    }

    #[test]
    fn delete_nonexistent() {
        let store = fake_store(3);
        assert!(!store.delete("ns", "no-such-id").unwrap());
    }

    #[tokio::test]
    async fn delete_wrong_namespace() {
        let store = fake_store(4);
        store.insert("a", "ns1", "text", json!({})).await.unwrap();
        assert!(!store.delete("ns2", "a").unwrap());
        assert_eq!(store.count(Some("ns1")).unwrap(), 1);
    }

    // ── clear_namespace ─────────────────────────────────────

    #[tokio::test]
    async fn clear_namespace_removes_all() {
        let store = fake_store(4);
        store.insert("a", "ns", "one", json!({})).await.unwrap();
        store.insert("b", "ns", "two", json!({})).await.unwrap();
        store
            .insert("c", "other", "three", json!({}))
            .await
            .unwrap();

        let deleted = store.clear_namespace("ns").unwrap();
        assert_eq!(deleted, 2);
        assert_eq!(store.count(Some("ns")).unwrap(), 0);
        assert_eq!(store.count(Some("other")).unwrap(), 1);
    }

    #[test]
    fn clear_empty_namespace() {
        let store = fake_store(3);
        let deleted = store.clear_namespace("empty").unwrap();
        assert_eq!(deleted, 0);
    }

    // ── list_namespaces ─────────────────────────────────────

    #[tokio::test]
    async fn list_namespaces_empty() {
        let store = fake_store(3);
        let ns = store.list_namespaces().unwrap();
        assert!(ns.is_empty());
    }

    #[tokio::test]
    async fn list_namespaces_populated() {
        let store = fake_store(4);
        store.insert("a", "beta", "t", json!({})).await.unwrap();
        store.insert("b", "alpha", "t", json!({})).await.unwrap();
        store.insert("c", "beta", "t", json!({})).await.unwrap();

        let ns = store.list_namespaces().unwrap();
        assert_eq!(ns, vec!["alpha", "beta"]);
    }

    // ── count ───────────────────────────────────────────────

    #[test]
    fn count_empty() {
        let store = fake_store(3);
        assert_eq!(store.count(None).unwrap(), 0);
        assert_eq!(store.count(Some("ns")).unwrap(), 0);
    }
}
