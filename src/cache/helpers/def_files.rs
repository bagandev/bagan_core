use napi_derive::napi;
/// File extension groups and paths excluded from filesystem traversal.
#[napi(object, namespace = "collect")]
#[derive(Debug, Default, Clone)]
pub struct DefaultFiles {
    /// Markdown file extensions.
    pub markdown_exts: Vec<String>,
    /// Static asset file extensions.
    pub binary_exts: Vec<String>,
    /// Liquid template file extensions.
    pub liquid_exts: Vec<String>,
    /// YAML file extensions.
    pub yaml_exts: Vec<String>,
    /// All file extensions recognized by the build.
    pub all_exts: Vec<String>,
    /// File and directory names excluded from traversal.
    pub ignore_list: Vec<String>,
    /// Image file extensions.
    pub image_exts: Vec<String>,
    /// Font file extensions.
    pub font_exts: Vec<String>,
    /// Audio file extensions.
    pub audio_exts: Vec<String>,
    /// Video file extensions.
    pub video_exts: Vec<String>,
    /// JS file extensions.
    pub js_exts: Vec<String>,
    /// Style(CSS,SCSS) file extensions.
    pub style_exts: Vec<String>,
}
#[napi(namespace = "collect")]
/// Returns the default extension groups and ignore list.
pub fn default_files() -> DefaultFiles {
    let markdown_exts = vec![
        "md".to_string(),
        "markdown".to_string(),
        "mdown".to_string(),
        "mkdn".to_string(),
        "mkd".to_string(),
        "mdwn".to_string(),
        "mkdown".to_string(),
        "ron".to_string(),
    ];
    let image_exts = vec![
        "png".to_string(),
        "jpg".to_string(),
        "jpeg".to_string(),
        "gif".to_string(),
        "svg".to_string(),
        "webp".to_string(),
        "avif".to_string(),
        "ico".to_string(),
        "bmp".to_string(),
        "tiff".to_string(),
        "apng".to_string(),
        "tif".to_string(),
    ];
    let font_exts = vec![
        "woff".to_string(),
        "woff2".to_string(),
        "ttf".to_string(),
        "eot".to_string(),
        "otf".to_string(),
    ];
    let audio_exts = vec![
        "aac".to_string(),
        "mid".to_string(),
        "midi".to_string(),
        "m4a".to_string(),
        "mp3".to_string(),
        "oga".to_string(),
        "opus".to_string(),
        "wav".to_string(),
        "weba".to_string(),
    ];
    let video_exts = vec![
        "avi".to_string(),
        "mp4".to_string(),
        "mpeg".to_string(),
        "ogv".to_string(),
        "webm".to_string(),
    ];
    let js_exts = vec!["js".to_string(), "mjs".to_string(), "map".to_string()];
    let style_exts = vec!["css".to_string(), "scss".to_string(), "map".to_string()];
    let binary_exts = [
        vec!["pdf".to_string(), "wasm".to_string()],
        font_exts.clone(),
        audio_exts.clone(),
        video_exts.clone(),
    ]
    .concat();
    let liquid_exts = vec!["html".to_string(), "liquid".to_string()];
    let yaml_exts = vec!["yml".to_string(), "yaml".to_string()];
    let all_exts = [
        markdown_exts.clone(),
        binary_exts.clone(),
        liquid_exts.clone(),
        yaml_exts.clone(),
        image_exts.clone(),
        js_exts.clone(),
        style_exts.clone(),
    ]
    .concat();
    let ignore_list = vec![
        "node_modules".to_string(),
        "target".to_string(),
        ".skills".to_string(),
        "skills".to_string(),
        ".github".to_string(),
        "README.md".to_string(),
        "ACCESSIBILITY.md".to_string(),
        "CHANGELOG.md".to_string(),
        "CODE_OF_CONDUCT.md".to_string(),
        "CONTRIBUTING.md".to_string(),
        "SECURITY.md".to_string(),
        ".git".to_string(),
    ];
    DefaultFiles {
        markdown_exts,
        binary_exts,
        liquid_exts,
        yaml_exts,
        all_exts,
        ignore_list,
        image_exts,
        font_exts,
        audio_exts,
        js_exts,
        style_exts,
        video_exts,
    }
}
