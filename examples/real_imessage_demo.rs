//! REAL DATA demo: ingest a slice of ~/Library/Messages/chat.db into an
//! in-memory life_capture PersonalIndex and run keyword searches.
//!
//! No embeddings (skips OpenAI) — just SQLite FTS5 over real iMessages, to
//! prove the loop works on real personal data.
//!
//! Run with: `cargo run --example real_imessage_demo --release`
//! (Requires Full Disk Access for the running terminal.)

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use openhuman_core::openhuman::life_capture::index::{IndexReader, IndexWriter, PersonalIndex};
use openhuman_core::openhuman::life_capture::types::{Item, Person, Source};
use rusqlite::{Connection, OpenFlags};

const N_MESSAGES: usize = 5000;
const QUERIES: &[&str] = &[
    "burger", "flight", "meeting", "dinner", "love", "uber", "thanks",
];

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let chat_db = std::env::var("CHAT_DB").unwrap_or_else(|_| {
        let home = std::env::var("HOME").expect("HOME unset");
        format!("{home}/Library/Messages/chat.db")
    });

    println!("→ opening chat.db at {chat_db}");
    let conn = Connection::open_with_flags(&chat_db, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .with_context(|| format!("open chat.db at {chat_db}"))?;

    println!("→ pulling last {N_MESSAGES} messages with text…");
    let mut stmt = conn.prepare(
        "SELECT m.ROWID, \
                COALESCE(m.text, '') as text, \
                m.date, \
                m.is_from_me, \
                COALESCE(h.id, '') as handle \
         FROM message m \
         LEFT JOIN handle h ON h.ROWID = m.handle_id \
         WHERE m.text IS NOT NULL AND length(m.text) > 0 \
         ORDER BY m.date DESC \
         LIMIT ?1",
    )?;

    // Apple stores dates in nanoseconds since 2001-01-01 UTC.
    const APPLE_EPOCH_OFFSET: i64 = 978_307_200; // unix seconds for 2001-01-01
    let rows = stmt.query_map([N_MESSAGES as i64], |row| {
        let rowid: i64 = row.get(0)?;
        let text: String = row.get(1)?;
        let raw_date: i64 = row.get(2)?;
        let is_from_me: i64 = row.get(3)?;
        let handle: String = row.get(4)?;
        let unix_secs = APPLE_EPOCH_OFFSET + (raw_date / 1_000_000_000);
        Ok((rowid, text, unix_secs, is_from_me != 0, handle))
    })?;

    let mut items = Vec::with_capacity(N_MESSAGES);
    for r in rows {
        let (rowid, text, unix_secs, is_from_me, handle) = r?;
        let ts = DateTime::<Utc>::from_timestamp(unix_secs, 0)
            .unwrap_or_else(|| DateTime::<Utc>::from_timestamp(0, 0).unwrap());
        let author = if is_from_me {
            None
        } else if handle.is_empty() {
            None
        } else {
            Some(Person {
                display_name: None,
                email: None,
                source_id: Some(handle.clone()),
            })
        };
        items.push(Item {
            id: uuid::Uuid::new_v4(),
            source: Source::IMessage,
            external_id: format!("imsg:rowid:{rowid}"),
            ts,
            author,
            subject: None,
            text,
            metadata: serde_json::json!({"is_from_me": is_from_me, "handle": handle}),
        });
    }
    println!("  pulled {} messages", items.len());

    println!("→ opening in-memory PersonalIndex…");
    let idx = PersonalIndex::open_in_memory().await?;
    let writer = IndexWriter::new(&idx);
    let reader = IndexReader::new(&idx);

    println!("→ ingesting (FTS5 only, no embeddings)…");
    let t0 = std::time::Instant::now();
    // Upsert in chunks to keep transaction sizes sane.
    for chunk in items.chunks_mut(500) {
        writer.upsert(chunk).await?;
    }
    let elapsed = t0.elapsed();
    println!(
        "  ingested {} items in {:.2}s ({:.0} items/s)",
        items.len(),
        elapsed.as_secs_f64(),
        items.len() as f64 / elapsed.as_secs_f64()
    );

    println!("\n=== KEYWORD SEARCH RESULTS ON REAL iMESSAGE DATA ===\n");
    for q in QUERIES {
        let hits = reader.keyword_search(q, 3).await?;
        println!("─── query: \"{}\"  ({} hits) ───", q, hits.len());
        for (i, h) in hits.iter().enumerate() {
            let dt = h.item.ts.format("%Y-%m-%d %H:%M");
            let from = h
                .item
                .author
                .as_ref()
                .and_then(|p| p.source_id.as_deref())
                .unwrap_or("me");
            let snippet = if h.snippet.is_empty() {
                h.item.text.chars().take(120).collect::<String>()
            } else {
                h.snippet.clone()
            };
            println!(
                "  {}. [{}] [{:>20}] (score {:.2}) {}",
                i + 1,
                dt,
                truncate(from, 20),
                h.score,
                snippet.replace('\n', " ")
            );
        }
        println!();
    }

    Ok(())
}

fn truncate(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        s.to_string()
    } else {
        s.chars().take(n.saturating_sub(1)).collect::<String>() + "…"
    }
}
