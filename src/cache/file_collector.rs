use super::helpers::{
    def_files::DefaultFiles, mime_types::get_mime_type, split_matter::split_front_matter,
};
use napi_derive::napi;
use std::path::PathBuf;

/// A file prepared for collection, with content and output metadata.
///
/// The variant identifies how the file should be handled: text files retain
/// their source, images include a data URI, and other binary files include
/// Base64-encoded content.
#[napi(namespace = "collect")]
pub enum InitFile {
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
