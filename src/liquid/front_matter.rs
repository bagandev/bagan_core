//! Parsing for YAML and Liquid layout declarations at the start of templates.

use napi_derive::napi;

/// Parsed template metadata and the remaining Liquid source.
///
/// Front matter entries are available as a JSON object, with `layout` removed
/// and exposed separately through [`layout`](Self::layout).
#[napi(object, namespace = "liquid")]
#[derive(Debug, Default)]
pub struct ParsedFrontmatter {
    /// Layout name declared in YAML front matter or by a top-level layout tag.
    pub layout: Option<String>,
    /// Template source after the front-matter or layout declaration.
    pub body: String,
    /// Parsed YAML front-matter entries, excluding `layout`.
    #[napi(ts_type = "Record<string,any>")]
    pub front_matter: serde_json::Value,
}

/// Parses template metadata and separates it from the Liquid body.
///
/// The function first recognizes Jekyll-style YAML front matter beginning with
/// a standalone `---` line. It then recognizes a top-level
/// `{% layout "name" %}` tag. If neither form is present, the original source
/// is returned unchanged.
///
/// Leading whitespace before the declaration is ignored for recognition, but
/// is not included in the parsed front matter or layout-tag body.
///
/// # Arguments
///
/// * `source` - Complete Liquid template source.
///
/// # Errors
///
/// Returns an N-API error when YAML front matter is invalid or is not a
/// key-value mapping.
#[napi(namespace = "liquid")]
pub fn front_matter(source: String) -> napi::Result<ParsedFrontmatter> {
    let trimmed = source.trim_start();

    // 1. Check for Jekyll-style YAML front matter (--- ... ---)
    if let Some(rest) = trimmed.strip_prefix("---") {
        // Ensure "---" is followed by a newline so horizontal rules aren't misread
        if let Some(after_open) = rest
            .strip_prefix('\n')
            .or_else(|| rest.strip_prefix("\r\n"))
        {
            if let Some(end_idx) = after_open.find("\n---") {
                let yaml_str = &after_open[..end_idx];
                let body_start = end_idx + 4; // skip "\n---"
                let body = after_open[body_start..]
                    .trim_start_matches(['\r', '\n'])
                    .to_string();

                // Parse full YAML (arrays, nested objects, numbers, booleans) into JSON Value
                let parsed_yaml: serde_json::Value =
                    serde_yml::from_str(yaml_str).map_err(|e| {
                        napi::Error::from_reason(format!("Invalid YAML front matter: {}", e))
                    })?;

                let mut front_matter_map = match parsed_yaml {
                    serde_json::Value::Object(map) => map,
                    serde_json::Value::Null => serde_json::Map::new(),
                    _ => {
                        return Err(napi::Error::from_reason(
                            "YAML front matter must be a key-value mapping",
                        ));
                    }
                };

                // Extract and remove `layout` from `page` variables
                let layout = front_matter_map
                    .remove("layout")
                    .and_then(|v| v.as_str().map(|s| s.to_string()));

                return Ok(ParsedFrontmatter {
                    layout,
                    body,
                    front_matter: serde_json::Value::Object(front_matter_map),
                });
            }
        }
    }

    // 2. Fallback: support {% layout "default" %} tag at the top of a template
    if let Some(rest) = trimmed.strip_prefix("{%") {
        if let Some(end_tag) = rest.find("%}") {
            let tag_inner = rest[..end_tag].trim();
            if let Some(layout_arg) = tag_inner.strip_prefix("layout") {
                let layout_name = layout_arg
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'')
                    .to_string();
                let body = rest[end_tag + 2..]
                    .trim_start_matches(['\r', '\n'])
                    .to_string();

                return Ok(ParsedFrontmatter {
                    layout: if layout_name.is_empty() {
                        None
                    } else {
                        Some(layout_name)
                    },
                    body,
                    front_matter: serde_json::Value::Object(serde_json::Map::new()),
                });
            }
        }
    }

    Ok(ParsedFrontmatter {
        layout: None,
        body: source,
        front_matter: serde_json::Value::Object(serde_json::Map::new()),
    })
}
