use dashmap::DashMap;
use napi::bindgen_prelude::*;
use napi_derive::napi;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize, Deserialize, Clone)]
pub struct CachedEntry {
    pub hash: [u8; 32],
    pub layout: Option<String>,
    pub front_matter_json: String,
    pub body_html: String,
}

#[napi(object)]
pub struct PageResult {
    pub path: String,
    pub layout: Option<String>,
    pub front_matter: serde_json::Value,
    pub body: String,
    pub from_cache: bool,
}

#[napi]
pub struct SiteCache {
    entries: Arc<DashMap<String, CachedEntry>>,
}

#[napi]
impl SiteCache {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            entries: Arc::new(DashMap::new()),
        }
    }

    /// Load persisted cache from disk on startup (optional)
    #[napi]
    pub async fn load_from_disk(cache_file: String) -> Result<Self> {
        match tokio::fs::read(&cache_file).await {
            Ok(bytes) => {
                let map: DashMap<String, CachedEntry> = postcard::from_bytes(&bytes)
                    .map_err(|e| Error::from_reason(format!("Cache decode error: {e}")))?;
                Ok(Self {
                    entries: Arc::new(map),
                })
            }
            Err(_) => Ok(Self::new()), // Cold start if cache file doesn't exist
        }
    }

    /// Persist cache to disk after a build completes
    #[napi]
    pub async fn save_to_disk(&self, cache_file: String) -> Result<()> {
        let bytes = postcard::to_allocvec(&*self.entries)
            .map_err(|e| Error::from_reason(format!("Cache encode error: {e}")))?;
        tokio::fs::write(cache_file, bytes)
            .await
            .map_err(|e| Error::from_reason(format!("Failed to write cache: {e}")))
    }

    /// Parse a file only if its blake3 hash changed
    #[napi]
    pub async fn get_or_parse(&self, file_path: String) -> Result<PageResult> {
        let raw_bytes = tokio::fs::read(&file_path)
            .await
            .map_err(|e| Error::from_reason(format!("Read error ({file_path}): {e}")))?;

        let current_hash = *blake3::hash(&raw_bytes).as_bytes();

        // 1. Fast cache hit path
        if let Some(cached) = self.entries.get(&file_path) {
            if cached.hash == current_hash {
                let front_matter = serde_json::from_str(&cached.front_matter_json)
                    .unwrap_or(serde_json::Value::Null);
                return Ok(PageResult {
                    path: file_path,
                    layout: cached.layout.clone(),
                    front_matter,
                    body: cached.body_html.clone(),
                    from_cache: true,
                });
            }
        }

        // 2. Cache miss: run your Markdown + frontmatter parser here
        let content = String::from_utf8_lossy(&raw_bytes);
        let (layout, fm_val, body_html) = parse_markdown_file(&content);
        let front_matter_json = serde_json::to_string(&fm_val).unwrap_or_default();

        self.entries.insert(
            file_path.clone(),
            CachedEntry {
                hash: current_hash,
                layout: layout.clone(),
                front_matter_json,
                body_html: body_html.clone(),
            },
        );

        Ok(PageResult {
            path: file_path,
            layout,
            front_matter: fm_val,
            body: body_html,
            from_cache: false,
        })
    }

    /// Evict deleted files from cache (call from your JS/Rust file watcher on unlink)
    #[napi]
    pub fn invalidate(&self, file_path: String) -> bool {
        self.entries.remove(&file_path).is_some()
    }

    /// Prune entries that no longer exist in the current build's file list
    #[napi]
    pub fn retain_paths(&self, active_paths: Vec<String>) {
        let active_set: std::collections::HashSet<String> = active_paths.into_iter().collect();
        self.entries.retain(|k, _| active_set.contains(k));
    }
}

fn parse_markdown_file(raw: &str) -> (Option<String>, serde_json::Value, String) {
    // Replace with your frontmatter & markdown parser
    (None, serde_json::json!({}), raw.to_string())
}
