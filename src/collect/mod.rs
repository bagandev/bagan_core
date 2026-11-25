pub mod def_files;
pub mod get_files;
pub mod mime_types;
pub mod split_matter;

use def_files::DefaultFiles;
use mime_types::get_mime_type;
use napi_derive::napi;
use std::env;
use std::path::PathBuf;

#[napi(namespace = "collect")]
pub enum InitFile {
    Markdown {
        input_file_path: PathBuf,
        out_file_path: PathBuf,
        raw_content: String,
        raw_frontmatter: Option<String>,
        raw_body_content: String,
        slug: String,
        mime_type: String,
        ext: String,
    },
    Liquid {
        input_file_path: PathBuf,
        out_file_path: PathBuf,
        raw_content: String,
        slug: String,
        mime_type: String,
        ext: String,
    },
    Js {
        input_file_path: PathBuf,
        out_file_path: PathBuf,
        raw_content: String,
        slug: String,
        mime_type: String,
        ext: String,
    },
    Css {
        input_file_path: PathBuf,
        out_file_path: PathBuf,
        raw_content: String,
        slug: String,
        mime_type: String,
        ext: String,
    },
}

#[napi(namespace = "collect")]
pub fn init_page_files(
    all_files: Vec<PathBuf>,
    exts: DefaultFiles,
    out_dir: PathBuf,
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

        let relative_path = match file.strip_prefix(&cwd) {
            Ok(rp) => rp,
            Err(_) => file.as_path(),
        };
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
        // ----------------------------------------------------------
        let page = if exts.clone().markdown_exts.contains(&ext) {
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
            let slug = if base_dir == "." {
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
            let slug = if base_dir == "." {
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
        } else {
            continue;
        };
        files.push(page);
    }

    Ok(files)
}
