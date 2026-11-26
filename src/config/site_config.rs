use napi_derive::napi;
use std::{env, path::PathBuf};

///
#[napi(object, namespace = "config")]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SiteConfig {
    /// The title of the site.
    pub title: Option<String>,
    /// A brief description of the site.
    pub description: Option<String>,
    /// The author's name, optionally including an email address.
    /// Example : Jane Doe <jane.doe@example.com>
    pub author: Option<String>,
    /// Keywords for site
    pub keywords: Option<Vec<String>>,
    /// The repository of site
    pub repository: Option<String>,
    /// Site url
    pub url: Option<String>,
    /// Site base url
    pub base_url: Option<String>,
    /// Project root
    pub root_dir: Option<PathBuf>,
    /// The directory where static files are located
    pub assets_dir: Option<PathBuf>,
    /// Cache dir
    pub cache_dir: Option<PathBuf>,
    /// Build output
    pub out_dir: Option<PathBuf>,
    /// The directory where post markdown files or liquid files are placed
    pub source_dir: Option<PathBuf>,
    pub data_dir: Option<PathBuf>,
}
#[napi(object, namespace = "config")]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ResolvedSiteConfig {
    /// The title of the site.
    pub title: String,
    /// A brief description of the site.
    pub description: Option<String>,
    /// The author's name, optionally including an email address.
    /// Example : Jane Doe <jane.doe@example.com>
    pub author: Option<String>,
    /// Keywords for site
    pub keywords: Option<Vec<String>>,
    /// The repository of site
    pub repository: Option<String>,
    /// Site url
    pub url: Option<String>,
    /// Site base url
    pub base_url: String,
    /// Project root
    pub root_dir: PathBuf,
    /// The directory where static files are located
    pub assets_dir: PathBuf,
    /// Cache dir
    pub cache_dir: PathBuf,
    /// Build output
    pub out_dir: PathBuf,
    /// The directory where post markdown files or liquid files are placed
    pub source_dir: PathBuf,
    pub data_dir: PathBuf,
}
#[napi(namespace = "config")]
pub fn resolve_site_config(site: Option<SiteConfig>) -> ResolvedSiteConfig {
    let cwd = env::current_dir().unwrap_or(PathBuf::from("."));
    let mut project_root = cwd.clone();
    let site_default = ResolvedSiteConfig {
        title: "Bagan SSG".to_string(),
        description: None,
        author: None,
        keywords: None,
        repository: None,
        url: None,
        base_url: "/".to_string(),
        root_dir: cwd.clone(),
        assets_dir: project_root.join("_assets"),
        cache_dir: project_root.join(".bagan_cache"),
        out_dir: project_root.join(".bagan"),
        source_dir: project_root.join("src"),
        data_dir: project_root.join("_data"),
    };
    match site {
        None => site_default,
        Some(site) => {
            if !site.root_dir.is_none() {
                project_root = cwd.clone().join(site.root_dir.unwrap());
            }
            ResolvedSiteConfig {
                title: site.title.unwrap_or(site_default.title),
                description: site.description,
                author: site.author,
                keywords: site.keywords,
                repository: site.repository,
                url: site.url,
                base_url: site.base_url.unwrap_or(site_default.base_url),
                root_dir: project_root.clone(),
                assets_dir: project_root
                    .clone()
                    .join(site.assets_dir.unwrap_or(PathBuf::from("_assets"))),
                cache_dir: project_root
                    .clone()
                    .join(site.cache_dir.unwrap_or(PathBuf::from(".bagan_cache"))),
                out_dir: project_root
                    .clone()
                    .join(site.out_dir.unwrap_or(PathBuf::from(".bagan"))),
                source_dir: project_root
                    .clone()
                    .join(site.source_dir.unwrap_or(PathBuf::from("src"))),
                data_dir: project_root
                    .clone()
                    .join(site.data_dir.unwrap_or(PathBuf::from("_data"))),
            }
        }
    }
}
