use super::helpers::{
    def_files::{DefaultFiles, default_files},
    mime_types::get_mime_type,
    split_matter::split_front_matter,
};
use dashmap::DashMap;
use napi::bindgen_prelude::*;
use napi_derive::napi;
use serde::{Deserialize, Serialize};
use std::env;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[napi(namespace = "cache")]
#[derive(Serialize, Deserialize, Clone)]
pub enum SiteFile {
    /// A Markdown document with parsed front matter and body content.
    Markdown {
        /// Destination path for the rendered document.
        out_file_path: PathBuf,
        /// Complete, unmodified source text.
        raw_content: String,
        /// Front matter including delimiters, if present.
        raw_frontmatter: Option<String>,
        /// Document text after the front matter.
        raw_body_content: String,
        /// URL path for this document.
        slug: String,
        /// MIME type inferred from the file extension.
        mime_type: String,
        /// File extension without a leading dot.
        ext: String,
    },
    /// A Liquid template document.
    Liquid {
        /// Destination path for the rendered template.
        out_file_path: PathBuf,
        /// Complete source text.
        raw_content: String,
        /// URL path for this template.
        slug: String,
        /// MIME type inferred from the file extension.
        mime_type: String,
        /// File extension without a leading dot.
        ext: String,
    },
    /// A JavaScript asset.
    Js {
        /// Destination path for the asset.
        out_file_path: PathBuf,
        /// Complete source text.
        raw_content: String,
        /// URL path for this asset.
        slug: String,
        /// MIME type inferred from the file extension.
        mime_type: String,
        /// File extension without a leading dot.
        ext: String,
    },
    /// A stylesheet asset.
    Css {
        /// Destination path for the asset.
        out_file_path: PathBuf,
        /// Complete source text.
        raw_content: String,
        /// URL path for this asset.
        slug: String,
        /// MIME type inferred from the file extension.
        mime_type: String,
        /// File extension without a leading dot.
        ext: String,
    },
    /// An image asset, including its Base64 content and data URI.
    Image {
        /// Destination path for the asset.
        out_file_path: PathBuf,
        /// Base64-encoded image bytes.
        raw_base64: String,
        /// Data URI containing the encoded image.
        base64_uri: String,
        /// URL path for this asset.
        slug: String,
        /// MIME type inferred from the file extension.
        mime_type: String,
        /// File extension without a leading dot.
        ext: String,
    },
    /// A non-image binary asset.
    Binary {
        /// Destination path for the asset.
        out_file_path: PathBuf,
        /// Base64-encoded file bytes.
        raw_base64: String,
        /// URL path for this asset.
        slug: String,
        /// MIME type inferred from the file extension.
        mime_type: String,
        /// File extension without a leading dot.
        ext: String,
    },
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SiteCacheEntry {
    pub hash: [u8; 32],
    pub site_file: SiteFile,
}
#[napi(object, namespace = "cache")]
pub struct SiteObject {
    pub input_file_path: String,
    pub site_file: SiteFile,
    pub from_cache: bool,
}

#[napi(namespace = "cache")]
pub struct SiteCache {
    entries: Arc<DashMap<String, SiteCacheEntry>>,
}
#[napi(namespace = "cache")]
impl SiteCache {
    #[napi(constructor, namespace = "cache")]
    pub fn new() -> Self {
        Self {
            entries: Arc::new(DashMap::new()),
        }
    }
    /// Load persisted cache from disk on startup (optional)
    #[napi(namespace = "cache")]
    pub async fn load_from_disk(cache_file: String) -> Result<Self> {
        match tokio::fs::read(&cache_file).await {
            Ok(bytes) => {
                let map: DashMap<String, SiteCacheEntry> = postcard::from_bytes(&bytes)
                    .map_err(|e| Error::from_reason(format!("Cache decode error: {e}")))?;
                Ok(Self {
                    entries: Arc::new(map),
                })
            }
            Err(_) => Ok(Self::new()), // Cold start if cache file doesn't exist
        }
    }
    /// Persist cache to disk after a build completes
    #[napi(namespace = "cache")]
    pub async fn save_to_disk(&self, cache_file: String) -> Result<()> {
        let bytes = postcard::to_allocvec(&*self.entries)
            .map_err(|e| Error::from_reason(format!("Cache encode error: {e}")))?;
        tokio::fs::write(cache_file, bytes)
            .await
            .map_err(|e| Error::from_reason(format!("Failed to write cache: {e}")))
    }
    /// Parse a file only if its blake3 hash changed
    #[napi(namespace = "cache")]
    pub async fn get_or_parse(
        &self,
        file_path: String,
        // TODO ->
        out_dir: String,
        assets_dir: Option<String>,
    ) -> napi::Result<SiteObject> {
        let raw_bytes = tokio::fs::read(&file_path)
            .await
            .map_err(|e| Error::from_reason(format!("Read error ({file_path}): {e}")))?;

        let current_hash = *blake3::hash(&raw_bytes).as_bytes();

        if let Some(cached) = self.entries.get(&file_path) {
            if cached.hash == current_hash {
                return Ok(SiteObject {
                    input_file_path: file_path,
                    site_file: cached.site_file.clone(),
                    from_cache: true,
                });
            }
        }

        let exts = default_files();

        let site_file =
            page_file_collector(file_path.clone(), exts, raw_bytes, out_dir, assets_dir)
                .await
                .map_err(|err| napi::Error::from_reason(err.reason))?;
        self.entries.insert(
            file_path.clone(),
            SiteCacheEntry {
                hash: current_hash,
                site_file: site_file.clone(),
            },
        );

        Ok(SiteObject {
            input_file_path: file_path,
            site_file,
            from_cache: false,
        })
    }
    /// Evict deleted files from cache (call from your JS/Rust file watcher on unlink)
    #[napi(namespace = "cache")]
    pub fn invalidate(&self, file_path: String) -> bool {
        self.entries.remove(&file_path).is_some()
    }

    /// Prune entries that no longer exist in the current build's file list
    #[napi(namespace = "cache")]
    pub fn retain_paths(&self, active_paths: Vec<String>) {
        let active_set: std::collections::HashSet<String> = active_paths.into_iter().collect();
        self.entries.retain(|k, _| active_set.contains(k));
    }
}

// --------------------------------------------------------------------
#[napi(namespace = "collect")]
pub async fn page_file_collector(
    file_path: String,
    exts: DefaultFiles,
    raw_bytes: Vec<u8>,
    out_dir: String,
    assets_dir: Option<String>,
) -> napi::Result<SiteFile> {
    tokio::task::spawn_blocking(move || {
        page_file_collector_sync(file_path, exts, raw_bytes, out_dir, assets_dir)
    })
    .await
    .map_err(|e| napi::Error::from_reason(format!("Worker thread panicked: {e}")))?
}

fn page_file_collector_sync(
    file_path: String,
    exts: DefaultFiles,
    raw_bytes: Vec<u8>,
    out_dir: String,
    assets_dir: Option<String>,
) -> napi::Result<SiteFile> {
    let cwd: PathBuf = env::current_dir()?;
    let entry_file_path = Path::new(&file_path);
    let out_dir_path = Path::new(&out_dir);
    let ext = entry_file_path
        .extension()
        .and_then(|e| Some(e.to_string_lossy().to_string()))
        .unwrap();

    let mime_type = get_mime_type(ext.clone(), exts.to_owned())
        .map_err(|error| napi::Error::from_reason(error.reason))?;

    let mut relative_path = match entry_file_path.strip_prefix(&cwd) {
        Ok(rp) => rp,
        Err(_) => entry_file_path,
    };
    let content = String::from_utf8_lossy(&raw_bytes);
    let collected_file = if exts.clone().markdown_exts.contains(&ext) {
        let dir_name = relative_path.parent().unwrap();
        let base_name = relative_path.file_name().and_then(|e| e.to_str()).unwrap();
        let parts: Vec<String> = base_name.split('.').map(str::to_string).collect();
        let file_name = parts[0].to_string();
        let base_dir = dir_name.to_string_lossy().to_string();
        let out_real_path = out_dir_path.join(relative_path);
        let out_dir_name = out_real_path
            .parent()
            .unwrap()
            .to_string_lossy()
            .to_string();
        let slug = if base_dir == "." && file_name == "index" {
            "/".to_string()
        } else if base_dir == "." && file_name != "index" {
            format!("/{}", file_name)
        } else if base_dir != "." && file_name == "index" {
            format!("/{}", base_dir)
        } else {
            format!("/{}/{}", base_dir, file_name)
        };
        let out_file_path = if out_dir_name == "." && file_name == "index" {
            format!("/{}", "index.html")
        } else if out_dir_name == "." && file_name != "index" {
            format!("/{}/{}", file_name, "index.html")
        } else if out_dir_name != "." && file_name == "index" {
            format!("/{}/{}", out_dir_name, "index.html")
        } else {
            format!("/{}/{}/{}", out_dir_name, file_name, "index.html")
        };
        let split = split_front_matter(content.to_string());
        SiteFile::Markdown {
            out_file_path: PathBuf::from(out_file_path),
            raw_content: content.to_string(),
            raw_frontmatter: split.front_matter,
            raw_body_content: split.body,
            slug,
            mime_type,
            ext,
        }
    } else if exts.liquid_exts.contains(&ext) {
        let dir_name = relative_path.parent().unwrap();
        let base_name = relative_path.file_name().and_then(|e| e.to_str()).unwrap();
        let parts: Vec<String> = base_name.split('.').map(str::to_string).collect();
        let file_name = parts[0].to_string();
        let base_dir = dir_name.to_string_lossy().to_string();
        let out_real_path = out_dir_path.join(relative_path);
        let out_dir_name = out_real_path
            .parent()
            .unwrap()
            .to_string_lossy()
            .to_string();
        let slug = if base_dir == "." && file_name == "index" {
            "/".to_string()
        } else if base_dir == "." && file_name != "index" {
            format!("/{}", file_name)
        } else if base_dir != "." && file_name == "index" {
            format!("/{}", base_dir)
        } else {
            format!("/{}/{}", base_dir, file_name)
        };
        let out_file_path = if out_dir_name == "." && file_name == "index" {
            format!("/{}", "index.html")
        } else if out_dir_name == "." && file_name != "index" {
            format!("/{}/{}", file_name, "index.html")
        } else if out_dir_name != "." && file_name == "index" {
            format!("/{}/{}", out_dir_name, "index.html")
        } else {
            format!("/{}/{}/{}", out_dir_name, file_name, "index.html")
        };

        SiteFile::Liquid {
            out_file_path: PathBuf::from(out_file_path),
            raw_content: content.to_string(),
            slug,
            mime_type,
            ext,
        }
    } else if exts.js_exts.contains(&ext) {
        if !assets_dir.is_none() && file_path.contains(&assets_dir.clone().unwrap()) {
            relative_path = match entry_file_path.strip_prefix(&assets_dir.clone().unwrap()) {
                Ok(rp) => rp,
                Err(_) => entry_file_path,
            };
        }
        let dir_name = relative_path.parent().unwrap();
        let base_name = relative_path.file_name().and_then(|e| e.to_str()).unwrap();
        let parts: Vec<String> = base_name.split('.').map(str::to_string).collect();
        let file_name = parts[0].to_string();
        let base_dir = dir_name.to_string_lossy().to_string();
        let out_real_path = out_dir_path.join(relative_path);
        let out_dir_name = out_real_path
            .parent()
            .unwrap()
            .to_string_lossy()
            .to_string();
        let slug = if base_dir == "." || base_dir == "" {
            format!("/{}.{}", file_name, ext)
        } else {
            format!("/{}/{}.{}", base_dir, file_name, ext)
        };
        let out_file_path = format!("/{}/{}.{}", out_dir_name, file_name, ext);
        SiteFile::Js {
            out_file_path: PathBuf::from(out_file_path),
            raw_content: content.to_string(),
            slug,
            mime_type,
            ext,
        }
    } else if exts.style_exts.contains(&ext) {
        if !assets_dir.is_none() && file_path.contains(&assets_dir.clone().unwrap()) {
            relative_path = match entry_file_path.strip_prefix(&assets_dir.clone().unwrap()) {
                Ok(rp) => rp,
                Err(_) => entry_file_path,
            };
        }
        let dir_name = relative_path.parent().unwrap();
        let base_name = relative_path.file_name().and_then(|e| e.to_str()).unwrap();
        let parts: Vec<String> = base_name.split('.').map(str::to_string).collect();
        let file_name = parts[0].to_string();
        let base_dir = dir_name.to_string_lossy().to_string();
        let out_real_path = out_dir_path.join(relative_path);
        let out_dir_name = out_real_path
            .parent()
            .unwrap()
            .to_string_lossy()
            .to_string();
        let slug = if base_dir == "." || base_dir == "" {
            format!("/{}.{}", file_name, ext)
        } else {
            format!("/{}/{}.{}", base_dir, file_name, ext)
        };
        let out_file_path = format!("/{}/{}.{}", out_dir_name, file_name, ext);
        SiteFile::Css {
            out_file_path: PathBuf::from(out_file_path),
            raw_content: content.to_string(),
            slug,
            mime_type,
            ext,
        }
    } else if exts.image_exts.contains(&ext) {
        if !assets_dir.is_none() && file_path.contains(&assets_dir.clone().unwrap()) {
            relative_path = match entry_file_path.strip_prefix(&assets_dir.clone().unwrap()) {
                Ok(rp) => rp,
                Err(_) => entry_file_path,
            };
        }
        let dir_name = relative_path.parent().unwrap();
        let base_name = relative_path.file_name().and_then(|e| e.to_str()).unwrap();
        let parts: Vec<String> = base_name.split('.').map(str::to_string).collect();
        let file_name = parts[0].to_string();
        let base_dir = dir_name.to_string_lossy().to_string();
        let out_real_path = out_dir_path.join(relative_path);
        let out_dir_name = out_real_path
            .parent()
            .unwrap()
            .to_string_lossy()
            .to_string();
        let slug = if base_dir == "." || base_dir == "" {
            format!("/{}.{}", file_name, ext)
        } else {
            format!("/{}/{}.{}", base_dir, file_name, ext)
        };
        let raw_base64 = binary_to_base64(&raw_bytes);
        let base64_uri = img_base64(raw_base64.clone(), ext.clone());
        let out_file_path = format!("/{}/{}.{}", out_dir_name, file_name, ext.clone());
        SiteFile::Image {
            out_file_path: PathBuf::from(out_file_path),
            raw_base64,
            base64_uri,
            slug,
            mime_type,
            ext,
        }
    } else if exts.binary_exts.contains(&ext) {
        if !assets_dir.is_none() && file_path.contains(&assets_dir.clone().unwrap()) {
            relative_path = match entry_file_path.strip_prefix(&assets_dir.clone().unwrap()) {
                Ok(rp) => rp,
                Err(_) => entry_file_path,
            };
        }
        let dir_name = relative_path.parent().unwrap();
        let base_name = relative_path.file_name().and_then(|e| e.to_str()).unwrap();
        let parts: Vec<String> = base_name.split('.').map(str::to_string).collect();
        let file_name = parts[0].to_string();
        let base_dir = dir_name.to_string_lossy().to_string();
        let out_real_path = out_dir_path.join(relative_path);
        let out_dir_name = out_real_path
            .parent()
            .unwrap()
            .to_string_lossy()
            .to_string();
        let slug = if base_dir == "." || base_dir == "" {
            format!("/{}.{}", file_name, ext)
        } else {
            format!("/{}/{}.{}", base_dir, file_name, ext)
        };
        let raw_base64 = binary_to_base64(&raw_bytes);
        let out_file_path = format!("/{}/{}.{}", out_dir_name, file_name, ext.clone());
        SiteFile::Binary {
            out_file_path: PathBuf::from(out_file_path),
            raw_base64,
            slug,
            mime_type,
            ext,
        }
    } else {
        return Err(napi::Error::from_reason(format!(
            "Unsupported file extension: {}",
            ext
        )));
    };
    Ok(collected_file)
}

/// Base64 encode bytes (standard alphabet with padding) without a
/// dependency — used for `AssetsObject.raw` of binary assets.
fn binary_to_base64(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = *chunk.get(1).unwrap_or(&0) as u32;
        let b2 = *chunk.get(2).unwrap_or(&0) as u32;
        let triple = (b0 << 16) | (b1 << 8) | b2;
        out.push(TABLE[((triple >> 18) & 63) as usize] as char);
        out.push(TABLE[((triple >> 12) & 63) as usize] as char);
        out.push(if chunk.len() > 1 {
            TABLE[((triple >> 6) & 63) as usize] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            TABLE[(triple & 63) as usize] as char
        } else {
            '='
        });
    }
    out
}

fn img_base64(encoded_string: String, ext: String) -> String {
    format!("{}/{};{},{}", "data:image", { ext }, "base64", {
        encoded_string
    })
}
