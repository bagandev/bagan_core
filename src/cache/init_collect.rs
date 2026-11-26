use super::def_files::DefaultFiles;
use super::mime_types::get_mime_type;
use super::split_matter;
use napi_derive::napi;
use std::path::PathBuf;
use std::{env, fs};

/// A file prepared for collection, with content and output metadata.
///
/// The variant identifies how the file should be handled: text files retain
/// their source, images include a data URI, and other binary files include
/// Base64-encoded content.
#[napi(namespace = "collect")]
pub enum InitFile {
    /// A Markdown document with parsed front matter and body content.
    Markdown {
        /// Path to the source file.
        input_file_path: PathBuf,
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
        /// Path to the source file.
        input_file_path: PathBuf,
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
        /// Path to the source file.
        input_file_path: PathBuf,
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
        /// Path to the source file.
        input_file_path: PathBuf,
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
        /// Path to the source file.
        input_file_path: PathBuf,
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
        /// Path to the source file.
        input_file_path: PathBuf,
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

/// Reads and classifies collected files for output.
///
/// Files are classified using `exts`; unsupported extensions are skipped.
/// Markdown and Liquid files are assigned rendered HTML output paths, while
/// asset files retain their asset extensions.
///
/// # Arguments
///
/// * `all_files` - Source paths to classify and read.
/// * `exts` - Extension groups and ignore configuration used for classification.
/// * `out_dir` - Base output directory used to construct destination paths.
/// * `assets_dir` - Optional asset root, used to make asset paths relative to it.
///
/// # Errors
///
/// Returns an error when the current directory cannot be read, a MIME type
/// cannot be determined, or a text file cannot be read. A worker-thread panic
/// is also returned as an error.
#[napi(namespace = "collect")]
pub async fn init_collect_files(
    all_files: Vec<PathBuf>,
    exts: DefaultFiles,
    out_dir: PathBuf,
    assets_dir: Option<PathBuf>,
) -> napi::Result<Vec<InitFile>> {
    tokio::task::spawn_blocking(move || {
        init_collect_files_sync(all_files, exts, out_dir, assets_dir)
    })
    .await
    .map_err(|e| napi::Error::from_reason(format!("Worker thread panicked: {e}")))?
}

/// Reads and classifies theme asset files for output.
///
/// Files are classified using `exts`; files with unsupported extensions are
/// skipped. The paths used for asset slugs and destinations are relative to
/// `theme_assets_dir` when possible.
///
/// # Arguments
///
/// * `theme_assets_files` - Source paths to classify and read.
/// * `exts` - Extension groups used to classify theme assets.
/// * `out_dir` - Base output directory used to construct destination paths.
/// * `theme_assets_dir` - Root directory used to derive relative asset paths.
///
/// # Errors
///
/// Returns an error when a MIME type cannot be determined or a text asset
/// cannot be read. A worker-thread panic is also returned as an error.
#[napi(namespace = "collect")]
pub async fn init_theme_assets(
    theme_assets_files: Vec<PathBuf>,
    exts: DefaultFiles,
    out_dir: PathBuf,
    theme_assets_dir: PathBuf,
) -> napi::Result<Vec<InitFile>> {
    tokio::task::spawn_blocking(move || {
        init_theme_assets_sync(theme_assets_files, exts, out_dir, theme_assets_dir)
    })
    .await
    .map_err(|e| napi::Error::from_reason(format!("Worker thread panicked: {e}")))?
}

fn init_theme_assets_sync(
    theme_assets_files: Vec<PathBuf>,
    exts: DefaultFiles,
    out_dir: PathBuf,
    theme_assets_dir: PathBuf,
) -> napi::Result<Vec<InitFile>> {
    let mut files: Vec<InitFile> = Vec::new();
    for file in theme_assets_files {
        let Some(ext) = file
            .extension()
            .and_then(|e| Some(e.to_string_lossy().to_string()))
        else {
            continue;
        };
        let mime_type = get_mime_type(ext.clone(), exts.to_owned())
            .map_err(|error| napi::Error::from_reason(error.reason))?;

        let relative_path = match file.strip_prefix(&theme_assets_dir) {
            Ok(rp) => rp,
            Err(_) => file.as_path(),
        };
        let collected_file = if exts.js_exts.contains(&ext) {
            let dir_name = relative_path.parent().unwrap();
            let Some(base_name) = relative_path.file_name().and_then(|e| e.to_str()) else {
                continue;
            };
            let parts: Vec<String> = base_name.split('.').map(str::to_string).collect();
            let file_name = parts[0].to_string();
            let base_dir = dir_name.to_string_lossy().to_string();
            let out_real_path = out_dir.join(relative_path);
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
            let raw_content = std::fs::read_to_string(&file).map_err(|error| {
                napi::Error::from_reason(format!("Failed to read '{}': {error}", file.display()))
            })?;
            let out_file_path = format!("/{}/{}.{}", out_dir_name, file_name, ext);
            InitFile::Js {
                input_file_path: file.clone(),
                out_file_path: PathBuf::from(out_file_path),
                raw_content,
                slug,
                mime_type,
                ext,
            }
        } else if exts.style_exts.contains(&ext) {
            let dir_name = relative_path.parent().unwrap();
            let Some(base_name) = relative_path.file_name().and_then(|e| e.to_str()) else {
                continue;
            };
            let parts: Vec<String> = base_name.split('.').map(str::to_string).collect();
            let file_name = parts[0].to_string();
            let base_dir = dir_name.to_string_lossy().to_string();
            let out_real_path = out_dir.join(relative_path);
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
            let raw_content = std::fs::read_to_string(&file).map_err(|error| {
                napi::Error::from_reason(format!("Failed to read '{}': {error}", file.display()))
            })?;
            let out_file_path = format!("/{}/{}.{}", out_dir_name, file_name, ext);
            InitFile::Css {
                input_file_path: file.clone(),
                out_file_path: PathBuf::from(out_file_path),
                raw_content,
                slug,
                mime_type,
                ext,
            }
        } else if exts.image_exts.contains(&ext) {
            let dir_name = relative_path.parent().unwrap();
            let Some(base_name) = relative_path.file_name().and_then(|e| e.to_str()) else {
                continue;
            };
            let parts: Vec<String> = base_name.split('.').map(str::to_string).collect();
            let file_name = parts[0].to_string();
            let base_dir = dir_name.to_string_lossy().to_string();
            let out_real_path = out_dir.join(relative_path);
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
            let raw_base64 = read_base64(file.clone());
            let base64_uri = img_base64(raw_base64.clone(), ext.clone());
            let out_file_path = format!("/{}/{}.{}", out_dir_name, file_name, ext.clone());
            InitFile::Image {
                input_file_path: file.clone(),
                out_file_path: PathBuf::from(out_file_path),
                raw_base64,
                base64_uri,
                slug,
                mime_type,
                ext,
            }
        } else if exts.binary_exts.contains(&ext) {
            let dir_name = relative_path.parent().unwrap();
            let Some(base_name) = relative_path.file_name().and_then(|e| e.to_str()) else {
                continue;
            };
            let parts: Vec<String> = base_name.split('.').map(str::to_string).collect();
            let file_name = parts[0].to_string();
            let base_dir = dir_name.to_string_lossy().to_string();
            let out_real_path = out_dir.join(relative_path);
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
            let raw_base64 = read_base64(file.clone());
            let out_file_path = format!("/{}/{}.{}", out_dir_name, file_name, ext.clone());
            InitFile::Binary {
                input_file_path: file.clone(),
                out_file_path: PathBuf::from(out_file_path),
                raw_base64,
                slug,
                mime_type,
                ext,
            }
        } else {
            continue;
        };
        files.push(collected_file);
    }
    Ok(files)
}

fn page_file_collector(
    file_path: PathBuf,
    exts: DefaultFiles,
    out_dir: PathBuf,
) -> napi::Result<InitFile> {
    let cwd: PathBuf = env::current_dir()?;
    let ext = file_path
        .extension()
        .and_then(|e| Some(e.to_string_lossy().to_string()))
        .unwrap();

    let mime_type = get_mime_type(ext.clone(), exts.to_owned())
        .map_err(|error| napi::Error::from_reason(error.reason))?;

    let relative_path = match file_path.strip_prefix(&cwd) {
        Ok(rp) => rp,
        Err(_) => file_path.as_path(),
    };
    let dir_name = relative_path.parent().unwrap();
    let base_name = relative_path.file_name().and_then(|e| e.to_str()).unwrap();
    let parts: Vec<String> = base_name.split('.').map(str::to_string).collect();
    let file_name = parts[0].to_string();
    let base_dir = dir_name.to_string_lossy().to_string();
    let out_real_path = out_dir.join(relative_path);
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
    let raw_content = std::fs::read_to_string(&file_path).map_err(|error| {
        napi::Error::from_reason(format!("Failed to read '{}': {error}", file_path.display()))
    })?;
    let collected_file = if exts.clone().markdown_exts.contains(&ext) {
        let split = split_matter::split_front_matter(raw_content.clone());
        InitFile::Markdown {
            input_file_path: file_path.clone(),
            out_file_path: PathBuf::from(out_file_path),
            raw_content,
            raw_frontmatter: split.front_matter,
            raw_body_content: split.body,
            slug,
            mime_type,
            ext,
        }
    } else {
        InitFile::Liquid {
            input_file_path: file_path.clone(),
            out_file_path: PathBuf::from(out_file_path),
            raw_content,
            slug,
            mime_type,
            ext,
        }
    };
    Ok(collected_file)
}

fn init_collect_files_sync(
    all_files: Vec<PathBuf>,
    exts: DefaultFiles,
    out_dir: PathBuf,
    assets_dir: Option<PathBuf>,
) -> napi::Result<Vec<InitFile>> {
    let mut files: Vec<InitFile> = Vec::new();
    let cwd: PathBuf = env::current_dir()?;
    for file in all_files {
        let Some(ext) = file
            .extension()
            .and_then(|e| Some(e.to_string_lossy().to_string()))
        else {
            continue;
        };
        let mime_type = get_mime_type(ext.clone(), exts.to_owned())
            .map_err(|error| napi::Error::from_reason(error.reason))?;

        let mut relative_path = match file.strip_prefix(&cwd) {
            Ok(rp) => rp,
            Err(_) => file.as_path(),
        };

        // ----------------------------------------------------------
        let collected_file = if exts.clone().markdown_exts.contains(&ext) {
            let dir_name = relative_path.parent().unwrap();
            let Some(base_name) = relative_path.file_name().and_then(|e| e.to_str()) else {
                continue;
            };
            let parts: Vec<String> = base_name.split('.').map(str::to_string).collect();
            let file_name = parts[0].to_string();
            let base_dir = dir_name.to_string_lossy().to_string();
            let out_real_path = out_dir.join(relative_path);
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
            let raw_content = std::fs::read_to_string(&file).map_err(|error| {
                napi::Error::from_reason(format!("Failed to read '{}': {error}", file.display()))
            })?;
            let split = split_matter::split_front_matter(raw_content.clone());
            InitFile::Markdown {
                input_file_path: file.clone(),
                out_file_path: PathBuf::from(out_file_path),
                raw_content,
                raw_frontmatter: split.front_matter,
                raw_body_content: split.body,
                slug,
                mime_type,
                ext,
            }
        } else if exts.liquid_exts.contains(&ext) {
            let dir_name = relative_path.parent().unwrap();
            let Some(base_name) = relative_path.file_name().and_then(|e| e.to_str()) else {
                continue;
            };
            let parts: Vec<String> = base_name.split('.').map(str::to_string).collect();
            let file_name = parts[0].to_string();
            let base_dir = dir_name.to_string_lossy().to_string();
            let out_real_path = out_dir.join(relative_path);
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
            let raw_content = std::fs::read_to_string(&file).map_err(|error| {
                napi::Error::from_reason(format!("Failed to read '{}': {error}", file.display()))
            })?;
            InitFile::Liquid {
                input_file_path: file.clone(),
                out_file_path: PathBuf::from(out_file_path),
                raw_content,
                slug,
                mime_type,
                ext,
            }
        } else if exts.js_exts.contains(&ext) {
            if !assets_dir.is_none()
                && file
                    .display()
                    .to_string()
                    .contains(&assets_dir.clone().unwrap().display().to_string())
            {
                relative_path = match file.strip_prefix(&assets_dir.clone().unwrap()) {
                    Ok(rp) => rp,
                    Err(_) => file.as_path(),
                };
            }
            let dir_name = relative_path.parent().unwrap();
            let Some(base_name) = relative_path.file_name().and_then(|e| e.to_str()) else {
                continue;
            };
            let parts: Vec<String> = base_name.split('.').map(str::to_string).collect();
            let file_name = parts[0].to_string();
            let base_dir = dir_name.to_string_lossy().to_string();
            let out_real_path = out_dir.join(relative_path);
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
            let raw_content = std::fs::read_to_string(&file).map_err(|error| {
                napi::Error::from_reason(format!("Failed to read '{}': {error}", file.display()))
            })?;
            let out_file_path = format!("/{}/{}.{}", out_dir_name, file_name, ext);
            InitFile::Js {
                input_file_path: file.clone(),
                out_file_path: PathBuf::from(out_file_path),
                raw_content,
                slug,
                mime_type,
                ext,
            }
        } else if exts.style_exts.contains(&ext) {
            if !assets_dir.is_none()
                && file
                    .display()
                    .to_string()
                    .contains(&assets_dir.clone().unwrap().display().to_string())
            {
                relative_path = match file.strip_prefix(&assets_dir.clone().unwrap()) {
                    Ok(rp) => rp,
                    Err(_) => file.as_path(),
                };
            }
            let dir_name = relative_path.parent().unwrap();
            let Some(base_name) = relative_path.file_name().and_then(|e| e.to_str()) else {
                continue;
            };
            let parts: Vec<String> = base_name.split('.').map(str::to_string).collect();
            let file_name = parts[0].to_string();
            let base_dir = dir_name.to_string_lossy().to_string();
            let out_real_path = out_dir.join(relative_path);
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
            let raw_content = std::fs::read_to_string(&file).map_err(|error| {
                napi::Error::from_reason(format!("Failed to read '{}': {error}", file.display()))
            })?;
            let out_file_path = format!("/{}/{}.{}", out_dir_name, file_name, ext);
            InitFile::Css {
                input_file_path: file.clone(),
                out_file_path: PathBuf::from(out_file_path),
                raw_content,
                slug,
                mime_type,
                ext,
            }
        } else if exts.image_exts.contains(&ext) {
            if !assets_dir.is_none()
                && file
                    .display()
                    .to_string()
                    .contains(&assets_dir.clone().unwrap().display().to_string())
            {
                relative_path = match file.strip_prefix(&assets_dir.clone().unwrap()) {
                    Ok(rp) => rp,
                    Err(_) => file.as_path(),
                };
            }
            let dir_name = relative_path.parent().unwrap();
            let Some(base_name) = relative_path.file_name().and_then(|e| e.to_str()) else {
                continue;
            };
            let parts: Vec<String> = base_name.split('.').map(str::to_string).collect();
            let file_name = parts[0].to_string();
            let base_dir = dir_name.to_string_lossy().to_string();
            let out_real_path = out_dir.join(relative_path);
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
            let raw_base64 = read_base64(file.clone());
            let base64_uri = img_base64(raw_base64.clone(), ext.clone());
            let out_file_path = format!("/{}/{}.{}", out_dir_name, file_name, ext.clone());
            InitFile::Image {
                input_file_path: file.clone(),
                out_file_path: PathBuf::from(out_file_path),
                raw_base64,
                base64_uri,
                slug,
                mime_type,
                ext,
            }
        } else if exts.binary_exts.contains(&ext) {
            if !assets_dir.is_none()
                && file
                    .display()
                    .to_string()
                    .contains(&assets_dir.clone().unwrap().display().to_string())
            {
                relative_path = match file.strip_prefix(&assets_dir.clone().unwrap()) {
                    Ok(rp) => rp,
                    Err(_) => file.as_path(),
                };
            }
            let dir_name = relative_path.parent().unwrap();
            let Some(base_name) = relative_path.file_name().and_then(|e| e.to_str()) else {
                continue;
            };
            let parts: Vec<String> = base_name.split('.').map(str::to_string).collect();
            let file_name = parts[0].to_string();
            let base_dir = dir_name.to_string_lossy().to_string();
            let out_real_path = out_dir.join(relative_path);
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
            let raw_base64 = read_base64(file.clone());
            let out_file_path = format!("/{}/{}.{}", out_dir_name, file_name, ext.clone());
            InitFile::Binary {
                input_file_path: file.clone(),
                out_file_path: PathBuf::from(out_file_path),
                raw_base64,
                slug,
                mime_type,
                ext,
            }
        } else {
            continue;
        };
        files.push(collected_file);
    }

    Ok(files)
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

fn read_base64(file_path: PathBuf) -> String {
    let bytes = fs::read(file_path.clone())
        .map_err(|error| {
            napi::Error::from_reason(format!(
                "Failed to read '{}': {error}",
                file_path.clone().display()
            ))
        })
        .unwrap();
    binary_to_base64(&bytes)
}

fn img_base64(encoded_string: String, ext: String) -> String {
    format!("{}/{};{},{}", "data:image", { ext }, "base64", {
        encoded_string
    })
}

#[cfg(test)]
mod tests {
    use super::{binary_to_base64, img_base64};

    #[test]
    fn encodes_bytes_as_padded_base64() {
        let cases: &[(&[u8], &str)] = &[
            (b"", ""),
            (b"f", "Zg=="),
            (b"fo", "Zm8="),
            (b"foo", "Zm9v"),
            (b"hello", "aGVsbG8="),
        ];

        for (input, expected) in cases {
            assert_eq!(binary_to_base64(input), *expected);
        }
    }

    #[test]
    fn formats_image_base64_as_a_data_uri() {
        assert_eq!(
            img_base64("aGVsbG8=".to_string(), "png".to_string()),
            "data:image/png;base64,aGVsbG8="
        );
    }
}
