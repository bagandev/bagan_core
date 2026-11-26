//! Utilities for separating Liquid front matter from the template body.
//!
//! Front matter must start on the first non-whitespace character of the source
//! and use standalone `---` delimiter lines.

use napi_derive::napi;

/// The result of separating a Liquid template into front matter and body.
#[napi(object, namespace = "collect")]
pub struct SplitTemplateResult {
    /// Raw front matter, including both `---` delimiter lines, if present.
    pub front_matter: Option<String>,
    /// The template content after the closing front-matter delimiter.
    pub body: String,
}

/// Separates YAML-style front matter from a Liquid template source.
///
/// The source has front matter only when it begins with a standalone `---`
/// line and contains a second standalone `---` line. Opening and closing
/// delimiters may use Unix or Windows line endings.
///
/// # Arguments
///
/// * `source` - Complete Liquid template source.
///
/// # Returns
///
/// A [`SplitTemplateResult`] containing the raw front matter and the remaining
/// body. If valid front matter is not found, `front_matter` is `None` and
/// `body` contains the original source.
#[napi(namespace = "collect")]
pub fn split_front_matter(source: String) -> SplitTemplateResult {
    let (fm, body) = split_front_matter_helper(&source);
    SplitTemplateResult {
        front_matter: fm.map(str::to_string),
        body: body.to_string(),
    }
}

/// Extracts raw front matter and the remaining template body.
///
/// Returns `(Some(front_matter), body)` when both delimiter lines are valid,
/// or `(None, source)` when front matter is absent or unclosed.
fn split_front_matter_helper(source: &str) -> (Option<&str>, &str) {
    let trimmed = source.trim_start();
    if !trimmed.starts_with("---") {
        return (None, source);
    }

    // Verify the opening line is strictly "---"
    let Some(first_newline) = trimmed.find('\n') else {
        return (None, source);
    };

    let open_line = trimmed[..first_newline].trim_end_matches('\r');
    if open_line != "---" {
        return (None, source);
    }

    // Scan line-by-line for a standalone closing "---" line
    let mut pos = first_newline + 1;
    while pos <= trimmed.len() {
        let remaining = &trimmed[pos..];
        let line_end = remaining
            .find('\n')
            .map(|i| pos + i)
            .unwrap_or(trimmed.len());

        let line = trimmed[pos..line_end].trim_end_matches('\r');

        if line == "---" {
            // Exclude trailing '\r' on Windows CRLF line endings so front_matter ends cleanly on "---"
            let fm_end = if trimmed[..line_end].ends_with('\r') {
                line_end - 1
            } else {
                line_end
            };

            let front_matter = &trimmed[..fm_end];
            let body = if line_end < trimmed.len() {
                &trimmed[line_end + 1..]
            } else {
                ""
            };

            return (Some(front_matter), body);
        }

        if line_end == trimmed.len() {
            break;
        }
        pos = line_end + 1;
    }

    // Unclosed "---" delimiter: treat entire input as body
    (None, source)
}
