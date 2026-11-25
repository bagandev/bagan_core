use napi::bindgen_prelude::*;
use napi_derive::napi;
use regex::Regex;

// Keep IgnoreRule private: JS only interacts with IgnoreMatcher
struct IgnoreRule {
    is_negated: bool,
    regex: Regex,
}

#[napi(namespace = "ignore")]
pub struct IgnoreMatcher {
    rules: Vec<IgnoreRule>,
}

#[napi(namespace = "ignore")]
impl IgnoreMatcher {
    #[napi(constructor, namespace = "ignore")]
    pub fn new(patterns: Vec<String>) -> Result<Self> {
        let mut rules = Vec::new();

        for raw_pattern in patterns {
            let trimmed = raw_pattern.trim();

            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let (is_negated, pattern) = if let Some(stripped) = trimmed.strip_prefix('!') {
                (true, stripped)
            } else if let Some(escaped_bang) = trimmed.strip_prefix(r"\!") {
                (false, escaped_bang)
            } else {
                (false, trimmed)
            };

            let regex = Self::compile_glob(pattern)?;
            rules.push(IgnoreRule { is_negated, regex });
        }

        Ok(Self { rules })
    }

    fn compile_glob(pattern: &str) -> Result<Regex> {
        let normalized = pattern.replace('\\', "/").trim_end_matches('/').to_string();
        let escaped = regex::escape(&normalized);

        let regex_body = escaped
            .replace(r"\*\*", ".*")
            .replace(r"\*", r"[^/]*")
            .replace(r"\?", r"[^/]");

        let full_pattern = format!(r"(^|/)({})(/|$)", regex_body);

        // Convert regex compilation errors into clean JS Exceptions
        Regex::new(&full_pattern)
            .map_err(|e| Error::new(Status::InvalidArg, format!("Invalid pattern: {e}")))
    }

    /// Exported to JS as `isIgnored(path: string): boolean`
    #[napi(namespace = "ignore")]
    pub fn is_ignored(&self, path: String) -> bool {
        let normalized_path = path.replace('\\', "/");
        let mut ignored = false;

        for rule in &self.rules {
            if rule.regex.is_match(&normalized_path) {
                ignored = !rule.is_negated;
            }
        }

        ignored
    }

    /// Optional batch helper: exported to JS as `filterIgnored(paths: string[]): string[]`
    #[napi(namespace = "ignore")]
    pub fn filter_ignored(&self, paths: Vec<String>) -> Vec<String> {
        paths
            .into_iter()
            .filter(|p| !self.is_ignored(p.clone()))
            .collect()
    }
}
