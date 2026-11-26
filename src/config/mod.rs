pub mod site_config;
pub mod theme_config;

use napi_derive::napi;
use serde_json::Value;
pub use site_config::{ResolvedSiteConfig, SiteConfig};
use std::collections::HashMap;
pub use theme_config::{ResolvedThemeConfig, ResolvedThemeDirs, ThemeConfig, ThemeDirs};

#[napi(object, namespace = "config")]
#[derive(Debug, Clone)]
pub struct BaganConfig {
    pub site: Option<SiteConfig>,
    pub theme: Option<ThemeConfig>,
    /// GFM
    pub gfm: Option<bool>,
    /// Kramdown
    pub kramdown: Option<bool>,
    pub ignore_list: Option<Vec<String>>,
    pub extra_config: Option<HashMap<String, Value>>,
}

#[napi(object, namespace = "config")]
#[derive(Debug, Clone)]
pub struct ResolvedBaganConfig {
    pub site: ResolvedSiteConfig,
    pub theme: ResolvedThemeConfig,
    /// GFM
    pub gfm: bool,
    /// Kramdown
    pub kramdown: bool,
    pub ignore_list: Option<Vec<String>>,
    pub extra_config: Option<HashMap<String, Value>>,
}
#[napi(namespace = "config")]
pub fn resolve_bagan_config(config: Option<BaganConfig>) -> ResolvedBaganConfig {
    let site = site_config::resolve_site_config(config.clone().unwrap().site);
    let theme = ResolvedThemeConfig::resolve(site.clone().root_dir, config.clone().unwrap().theme);
    let gfm = config.clone().unwrap().gfm.unwrap_or(true);
    let kramdown = config.clone().unwrap().kramdown.unwrap_or(true);
    let ignore_list = config.clone().unwrap().ignore_list;
    let extra_config = config.clone().unwrap().extra_config;
    ResolvedBaganConfig {
        site,
        theme,
        gfm,
        kramdown,
        ignore_list,
        extra_config,
    }
}
