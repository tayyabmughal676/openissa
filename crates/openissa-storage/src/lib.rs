//! OpenISSA Storage: Embedded SQLite + WAL caching and BM25 full-text indexing.

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use openissa_core::error::{OpenIssaError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHit {
    pub url: String,
    pub title: String,
    pub snippet: String,
    pub score: f64,
}

pub struct StorageEngine {
    conn: Mutex<Connection>,
}

impl StorageEngine {
    /// Open or create the local SQLite database in WAL mode with FTS5 BM25 index.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn =
            Connection::open(path).map_err(|e| OpenIssaError::StorageError(e.to_string()))?;

        // Enable WAL mode for high concurrency & initialize tables
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA foreign_keys = ON;
             
             CREATE TABLE IF NOT EXISTS http_cache (
                 url TEXT PRIMARY KEY,
                 status INTEGER NOT NULL,
                 markdown TEXT NOT NULL,
                 fetched_at INTEGER NOT NULL,
                 expires_at INTEGER NOT NULL
             );
             
             CREATE TABLE IF NOT EXISTS sessions (
                 id TEXT PRIMARY KEY,
                 goal TEXT NOT NULL,
                 status TEXT NOT NULL,
                 created_at INTEGER NOT NULL
             );

             CREATE VIRTUAL TABLE IF NOT EXISTS doc_index USING fts5(
                 url UNINDEXED,
                 title,
                 body,
                 tokenize = 'porter unicode61'
             );",
        )
        .map_err(|e| OpenIssaError::StorageError(e.to_string()))?;

        // Sync any unindexed http_cache items into doc_index
        let _ = conn.execute(
            "INSERT OR IGNORE INTO doc_index (url, title, body)
             SELECT url, url, markdown FROM http_cache
             WHERE url NOT IN (SELECT url FROM doc_index)",
            [],
        );

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Open an in-memory database for testing.
    pub fn open_in_memory() -> Result<Self> {
        let conn =
            Connection::open_in_memory().map_err(|e| OpenIssaError::StorageError(e.to_string()))?;

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS http_cache (
                 url TEXT PRIMARY KEY,
                 status INTEGER NOT NULL,
                 markdown TEXT NOT NULL,
                 fetched_at INTEGER NOT NULL,
                 expires_at INTEGER NOT NULL
             );

             CREATE VIRTUAL TABLE IF NOT EXISTS doc_index USING fts5(
                 url UNINDEXED,
                 title,
                 body,
                 tokenize = 'porter unicode61'
             );",
        )
        .map_err(|e| OpenIssaError::StorageError(e.to_string()))?;

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Retrieve a cached Markdown document for a URL if not expired.
    pub fn get_cached_url(&self, url: &str) -> Result<Option<String>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| OpenIssaError::StorageError(e.to_string()))?;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| OpenIssaError::StorageError(e.to_string()))?
            .as_secs() as i64;

        let mut stmt = conn
            .prepare("SELECT markdown FROM http_cache WHERE url = ?1 AND expires_at > ?2")
            .map_err(|e| OpenIssaError::StorageError(e.to_string()))?;

        let mut rows = stmt
            .query(params![url, now])
            .map_err(|e| OpenIssaError::StorageError(e.to_string()))?;

        if let Some(row) = rows
            .next()
            .map_err(|e| OpenIssaError::StorageError(e.to_string()))?
        {
            let markdown: String = row
                .get(0)
                .map_err(|e| OpenIssaError::StorageError(e.to_string()))?;
            Ok(Some(markdown))
        } else {
            Ok(None)
        }
    }

    /// Save a fetched URL and its Markdown content with a TTL, automatically indexing into FTS5.
    pub fn set_cached_url(
        &self,
        url: &str,
        status: u16,
        markdown: &str,
        ttl_secs: u64,
    ) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| OpenIssaError::StorageError(e.to_string()))?;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| OpenIssaError::StorageError(e.to_string()))?
            .as_secs() as i64;
        let expires_at = now + (ttl_secs as i64);

        conn.execute(
            "INSERT OR REPLACE INTO http_cache (url, status, markdown, fetched_at, expires_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![url, status, markdown, now, expires_at],
        )
        .map_err(|e| OpenIssaError::StorageError(e.to_string()))?;

        // Extract title from markdown or fallback to domain
        let title = markdown
            .lines()
            .find(|l| l.starts_with("# ") || !l.trim().is_empty())
            .map(|l| l.trim_start_matches('#').trim().to_string())
            .unwrap_or_else(|| url.to_string());

        // Update BM25 search index
        let _ = conn.execute("DELETE FROM doc_index WHERE url = ?1", params![url]);
        let _ = conn.execute(
            "INSERT INTO doc_index (url, title, body) VALUES (?1, ?2, ?3)",
            params![url, title, markdown],
        );

        Ok(())
    }

    /// Index a document explicitly into the BM25 full-text index.
    pub fn index_document(&self, url: &str, title: &str, body: &str) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| OpenIssaError::StorageError(e.to_string()))?;

        let _ = conn.execute("DELETE FROM doc_index WHERE url = ?1", params![url]);
        conn.execute(
            "INSERT INTO doc_index (url, title, body) VALUES (?1, ?2, ?3)",
            params![url, title, body],
        )
        .map_err(|e| OpenIssaError::StorageError(e.to_string()))?;

        Ok(())
    }

    /// Record a research session in the sessions table.
    pub fn save_session(&self, id: &str, goal: &str, status: &str) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| OpenIssaError::StorageError(e.to_string()))?;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| OpenIssaError::StorageError(e.to_string()))?
            .as_secs() as i64;

        conn.execute(
            "INSERT OR REPLACE INTO sessions (id, goal, status, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![id, goal, status, now],
        )
        .map_err(|e| OpenIssaError::StorageError(e.to_string()))?;

        Ok(())
    }

    /// Search local BM25 full-text index across all indexed documents (Tier 2 Retrieval Ladder).
    pub fn search_index(&self, query_str: &str, limit: usize) -> Result<Vec<SearchHit>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| OpenIssaError::StorageError(e.to_string()))?;

        let clean_query = query_str.replace(['"', '\'', '*'], "").trim().to_string();

        if clean_query.is_empty() {
            return Ok(Vec::new());
        }

        let mut stmt = conn
            .prepare(
                "SELECT url, title, snippet(doc_index, 2, '<b>', '</b>', '...', 20), bm25(doc_index)
                 FROM doc_index
                 WHERE doc_index MATCH ?1
                 ORDER BY rank
                 LIMIT ?2",
            )
            .map_err(|e| OpenIssaError::StorageError(e.to_string()))?;

        let rows = stmt
            .query_map(params![clean_query, limit as i64], |row| {
                Ok(SearchHit {
                    url: row.get(0)?,
                    title: row.get(1)?,
                    snippet: row.get(2)?,
                    score: row.get(3)?,
                })
            })
            .map_err(|e| OpenIssaError::StorageError(e.to_string()))?;

        let hits = rows.flatten().collect();

        Ok(hits)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_caching() {
        let storage = StorageEngine::open_in_memory().unwrap();
        let url = "https://example.com";
        let content = "# Example Page\nContent here";

        assert!(storage.get_cached_url(url).unwrap().is_none());

        storage.set_cached_url(url, 200, content, 3600).unwrap();
        let cached = storage.get_cached_url(url).unwrap();
        assert_eq!(cached, Some(content.to_string()));
    }

    #[test]
    fn test_bm25_full_text_search() {
        let storage = StorageEngine::open_in_memory().unwrap();
        storage
            .index_document(
                "https://docs.rs/tokio",
                "Tokio Async Runtime",
                "Tokio is an asynchronous runtime for the Rust programming language.",
            )
            .unwrap();

        storage
            .index_document(
                "https://docs.rs/axum",
                "Axum Web Framework",
                "Axum is a web application framework that focuses on ergonomics and modularity.",
            )
            .unwrap();

        let hits = storage.search_index("asynchronous runtime", 5).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].url, "https://docs.rs/tokio");
        assert!(hits[0].title.contains("Tokio"));
    }
}
