use super::def_files::DefaultFiles;
use napi_derive::napi;

#[napi(namespace = "collect")]
pub fn get_mime_type(ext: String, extensions: DefaultFiles) -> napi::Result<String> {
    let mut result = match ext.as_str() {
        "pdf" => "application/pdf",
        "wasm" => "application/wasm",
        "webmanifest" => "application/manifest+json",
        _ => "application/octet-stream",
    };
    if extensions.markdown_exts.contains(&ext) || extensions.liquid_exts.contains(&ext) {
        result = "text/html; charset=utf-8";
    } else if extensions.js_exts.contains(&ext) {
        result = "text/javascript; charset=utf-8";
    } else if extensions.style_exts.contains(&ext) {
        result = "text/css; charset=utf-8";
    } else if extensions.image_exts.contains(&ext) {
        result = match ext.as_str() {
            "png" => "image/png",
            "jpg" => "image/jpeg",
            "jpeg" => "image/jpeg",
            "gif" => "image/gif",
            "svg" => "image/svg+xml",
            "webp" => "image/webp",
            "avif" => "image/avif",
            "ico" => "image/vnd.microsoft.icon",
            "bmp" => "image/bmp",
            "tiff" => "image/tiff",
            "tif" => "image/tiff",
            "apng" => "image/apng",
            _ => "application/octet-stream",
        }
    } else if extensions.font_exts.contains(&ext) {
        result = match ext.as_str() {
            "woff" => "font/woff",
            "woff2" => "font/woff2",
            "ttf" => "font/ttf",
            "eot" => "font/eot",
            "otf" => "font/otf",
            _ => "application/octet-stream",
        }
    } else if extensions.audio_exts.contains(&ext) {
        result = match ext.as_str() {
            "aac" => "audio/aac",
            "mid" => "audio/midi",
            "midi" => "audio/midi",
            "m4a" => "audio/m4a",
            "mp3" => "audio/mp3",
            "oga" => "audio/oga",
            "opus" => "audio/opus",
            "wav" => "audio/wav",
            "weba" => "audio/weba",
            _ => "application/octet-stream",
        }
    } else if extensions.video_exts.contains(&ext) {
        result = match ext.as_str() {
            "avi" => "video/x-msvideo",
            "mp4" => "video/mp4",
            "mpeg" => "video/mpeg",
            "ogv" => "video/ogv",
            "ts" => "video/mp2t",
            "webm" => "video/webm",
            _ => "application/octet-stream",
        }
    }
    Ok(result.to_string())
}
