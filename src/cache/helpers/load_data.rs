use async_recursion::async_recursion;
use napi::bindgen_prelude::*;
use napi_derive::napi;
use serde_json::{Map, Value};
use std::path::{Path, PathBuf};
use tokio::fs;

fn deep_merge(base: &mut Value, override_val: Value) {
    match (base, override_val) {
        (Value::Object(base_map), Value::Object(override_map)) => {
            for (k, v) in override_map {
                match base_map.get_mut(&k) {
                    Some(existing) => deep_merge(existing, v),
                    None => {
                        base_map.insert(k, v);
                    }
                }
            }
        }
        (base_slot, new_val) => {
            *base_slot = new_val;
        }
    }
}

#[async_recursion]
async fn read_data_dir_recursive(dir: PathBuf) -> std::result::Result<Value, String> {
    let mut root = Map::new();

    if fs::metadata(&dir).await.is_err() {
        return Ok(Value::Object(root));
    }

    let mut entries = fs::read_dir(&dir)
        .await
        .map_err(|e| format!("Failed to read directory '{}': {}", dir.display(), e))?;

    while let Some(entry) = entries.next_entry().await.map_err(|e| e.to_string())? {
        let path = entry.path();

        let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        let key = stem.to_string();

        let file_type = entry.file_type().await.map_err(|e| e.to_string())?;

        if file_type.is_dir() {
            let sub_tree = read_data_dir_recursive(path).await?;
            root.insert(key, sub_tree);
        } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if ext.eq_ignore_ascii_case("yml") || ext.eq_ignore_ascii_case("yaml") {
                let content = fs::read_to_string(&path)
                    .await
                    .map_err(|e| format!("Failed to read '{}': {}", path.display(), e))?;

                let parsed: Value = if content.trim().is_empty() {
                    Value::Null
                } else {
                    serde_yml::from_str(&content)
                        .map_err(|e| format!("YAML error in '{}': {}", path.display(), e))?
                };

                root.insert(key, parsed);
            }
        }
    }

    Ok(Value::Object(root))
}
#[napi(namespace = "cache", ts_return_type = "Promise<DataContext>")]
pub async fn load_merged_data_dirs(base_dir: String, override_dir: String) -> Result<Value> {
    let (base_res, override_res) = tokio::join!(
        read_data_dir_recursive(Path::new(&base_dir).to_path_buf()),
        read_data_dir_recursive(Path::new(&override_dir).to_path_buf())
    );

    let mut merged = base_res.map_err(Error::from_reason)?;
    let override_data = override_res.map_err(Error::from_reason)?;

    deep_merge(&mut merged, override_data);
    Ok(merged)
}

pub async fn load_and_deep_merge_dirs(dirs: &[String]) -> Result<Value> {
    let data = load_merged_data_dirs(dirs[0].clone(), dirs[1].clone()).await?;
    Ok(serde_json::json!(data))
}
