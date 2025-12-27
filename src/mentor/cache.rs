// Mentor Guidance Cache
//
// Caches LLM-generated guidance to avoid repeated API calls
// for the same or similar errors.

use anyhow::Result;
use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;
use std::sync::Mutex;

use super::guidance::{GuidanceSource, MentorGuidance};
use super::types::ErrorInfo;

/// Cache for mentor guidance responses
pub struct GuidanceCache {
    conn: Mutex<Connection>,
}

impl GuidanceCache {
    /// Create a new cache with the given database path
    pub fn new(db_path: impl AsRef<Path>) -> Result<Self> {
        let conn = Connection::open(db_path)?;

        // Create table if not exists
        conn.execute(
            "CREATE TABLE IF NOT EXISTS guidance_cache (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                cache_key TEXT UNIQUE NOT NULL,
                error_type TEXT NOT NULL,
                guidance_json TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                hit_count INTEGER DEFAULT 1
            )",
            [],
        )?;

        // Create index for faster lookups
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_cache_key ON guidance_cache(cache_key)",
            [],
        )?;

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Create an in-memory cache (for testing)
    pub fn in_memory() -> Result<Self> {
        Self::new(":memory:")
    }

    /// Generate cache key from error info
    fn cache_key(error: &ErrorInfo) -> String {
        // Key based on error type and normalized key message
        let normalized_msg = error
            .key_message
            .to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace())
            .collect::<String>();

        format!("{}:{}", error.error_type.name(), normalized_msg)
    }

    /// Get cached guidance for an error
    pub fn get(&self, error: &ErrorInfo) -> Option<MentorGuidance> {
        let key = Self::cache_key(error);
        let conn = self.conn.lock().ok()?;

        let result: Option<String> = conn
            .query_row(
                "SELECT guidance_json FROM guidance_cache WHERE cache_key = ?",
                params![key],
                |row| row.get(0),
            )
            .optional()
            .ok()?;

        if let Some(json) = result {
            // Update hit count
            let _ = conn.execute(
                "UPDATE guidance_cache SET hit_count = hit_count + 1 WHERE cache_key = ?",
                params![key],
            );

            // Parse and return
            serde_json::from_str::<MentorGuidance>(&json)
                .ok()
                .map(|mut g| {
                    g.source = GuidanceSource::Cached;
                    g
                })
        } else {
            None
        }
    }

    /// Store guidance in cache
    pub fn set(&self, error: &ErrorInfo, guidance: &MentorGuidance) -> Result<()> {
        let key = Self::cache_key(error);
        let json = serde_json::to_string(guidance)?;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("{}", e))?;

        conn.execute(
            "INSERT OR REPLACE INTO guidance_cache (cache_key, error_type, guidance_json, created_at)
             VALUES (?, ?, ?, ?)",
            params![key, error.error_type.name(), json, now],
        )?;

        Ok(())
    }

    /// Clean entries older than the given number of days
    pub fn clean_old_entries(&self, retention_days: u32) -> Result<usize> {
        let cutoff = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
            - (retention_days as i64 * 24 * 60 * 60);

        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("{}", e))?;

        let deleted = conn.execute(
            "DELETE FROM guidance_cache WHERE created_at < ?",
            params![cutoff],
        )?;

        Ok(deleted)
    }

    /// Get cache statistics
    pub fn stats(&self) -> Result<CacheStats> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("{}", e))?;

        let total_entries: i64 = conn.query_row(
            "SELECT COUNT(*) FROM guidance_cache",
            [],
            |row| row.get(0),
        )?;

        let total_hits: i64 = conn.query_row(
            "SELECT COALESCE(SUM(hit_count), 0) FROM guidance_cache",
            [],
            |row| row.get(0),
        )?;

        Ok(CacheStats {
            total_entries: total_entries as usize,
            total_hits: total_hits as usize,
        })
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_entries: usize,
    pub total_hits: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mentor::types::ErrorType;

    fn create_test_error() -> ErrorInfo {
        ErrorInfo::new(
            ErrorType::CommandNotFound,
            127,
            "command not found: kubectl",
            "kubectl get pods",
        )
    }

    fn create_test_guidance() -> MentorGuidance {
        MentorGuidance::from_pattern(
            "kubectl not found",
            "The kubectl command is not installed",
        )
        .with_search(vec!["install kubectl".to_string()])
    }

    #[test]
    fn test_cache_creation() {
        let cache = GuidanceCache::in_memory();
        assert!(cache.is_ok());
    }

    #[test]
    fn test_cache_key_generation() {
        let error = create_test_error();
        let key = GuidanceCache::cache_key(&error);

        assert!(key.contains("Command Not Found"));
        assert!(key.contains("kubectl"));
    }

    #[test]
    fn test_cache_miss() {
        let cache = GuidanceCache::in_memory().unwrap();
        let error = create_test_error();

        let result = cache.get(&error);
        assert!(result.is_none());
    }

    #[test]
    fn test_cache_set_and_get() {
        let cache = GuidanceCache::in_memory().unwrap();
        let error = create_test_error();
        let guidance = create_test_guidance();

        // Set
        cache.set(&error, &guidance).unwrap();

        // Get
        let cached = cache.get(&error);
        assert!(cached.is_some());

        let cached = cached.unwrap();
        assert_eq!(cached.key_message, "kubectl not found");
        assert_eq!(cached.source, GuidanceSource::Cached);
    }

    #[test]
    fn test_cache_stats() {
        let cache = GuidanceCache::in_memory().unwrap();
        let error = create_test_error();
        let guidance = create_test_guidance();

        cache.set(&error, &guidance).unwrap();
        cache.get(&error); // Hit
        cache.get(&error); // Another hit

        let stats = cache.stats().unwrap();
        assert_eq!(stats.total_entries, 1);
        assert_eq!(stats.total_hits, 3); // Initial + 2 hits
    }

    #[test]
    fn test_similar_errors_same_cache() {
        let cache = GuidanceCache::in_memory().unwrap();
        let guidance = create_test_guidance();

        let error1 = ErrorInfo::new(
            ErrorType::CommandNotFound,
            127,
            "command not found: KUBECTL",
            "KUBECTL get pods",
        );

        let error2 = ErrorInfo::new(
            ErrorType::CommandNotFound,
            127,
            "Command Not Found: kubectl",
            "kubectl get pods",
        );

        cache.set(&error1, &guidance).unwrap();

        // error2 should hit the same cache entry (normalized)
        let cached = cache.get(&error2);
        assert!(cached.is_some());
    }
}
