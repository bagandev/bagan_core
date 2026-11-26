use napi_derive::napi;
use serde::{Deserialize, Serialize};
use std::{
    env,
    path::{Path, PathBuf},
};

#[napi(namespace = "config")]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemeConfig {
    Npm {
        /// Theme package name.
        name: String,
        /// Theme package version.
        version: String,
        /// Optional theme author.
        #[serde(default)]
        author: Option<String>,
        #[serde(default)]
        theme_dirs: Option<ThemeDirs>,
    },
    Local {
        #[serde(default)]
        theme_dirs: Option<ThemeDirs>,
    },
}

#[napi(object, namespace = "config")]
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ThemeDirs {
    /// Relative path to the theme's HTML layout templates (default `"_layouts"`).
    pub layouts_dir: Option<PathBuf>,
    /// Relative path to the theme's include templates (default `"_includes"`).
    pub includes_dir: Option<PathBuf>,
    /// Relative path to the theme's static assets (default `"_assets"`).
    pub assets_dir: Option<PathBuf>,
    /// Relative path to the theme's data files (default `"_data"`).
    pub data_dir: Option<PathBuf>,
}
#[napi(object, namespace = "config")]
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedThemeDirs {
    /// Relative path to the theme's HTML layout templates (default `"_layouts"`).
    pub layouts_dir: PathBuf,
    /// Relative path to the theme's include templates (default `"_includes"`).
    pub includes_dir: PathBuf,
    /// Relative path to the theme's static assets (default `"_assets"`).
    pub assets_dir: PathBuf,
    /// Relative path to the theme's data files (default `"_data"`).
    pub data_dir: PathBuf,
}

fn resolve_theme_config<P: AsRef<Path>>(
    root_dir: P,
    theme: Option<ThemeConfig>,
) -> ResolvedThemeConfig {
    let cwd = env::current_dir().unwrap_or(PathBuf::from("."));
    let dir_path = if root_dir.as_ref() == Path::new(".") {
        cwd.clone()
    } else {
        cwd.join(root_dir)
    };
    let default_dirs = ResolvedThemeDirs {
        layouts_dir: dir_path.join("theme").join("_layouts"),
        includes_dir: dir_path.join("theme").join("_includes"),
        assets_dir: dir_path.join("theme").join("_assets"),
        data_dir: dir_path.join("theme").join("_data"),
    };

    if theme.is_none() {
        ResolvedThemeConfig::Local {
            theme_dirs: default_dirs,
        }
    } else {
        match theme.unwrap() {
            ThemeConfig::Local { theme_dirs } => {
                let local_dirs = ResolvedThemeDirs {
                    layouts_dir: dir_path.join("theme").join(
                        theme_dirs
                            .clone()
                            .unwrap()
                            .layouts_dir
                            .unwrap_or(PathBuf::from("_layouts")),
                    ),
                    includes_dir: dir_path.join("theme").join(
                        theme_dirs
                            .clone()
                            .unwrap()
                            .includes_dir
                            .unwrap_or(PathBuf::from("_includes")),
                    ),
                    assets_dir: dir_path.join("theme").join(
                        theme_dirs
                            .clone()
                            .unwrap()
                            .assets_dir
                            .unwrap_or(PathBuf::from("_assets")),
                    ),
                    data_dir: dir_path.join("theme").join(
                        theme_dirs
                            .unwrap()
                            .data_dir
                            .unwrap_or(PathBuf::from("_data")),
                    ),
                };
                ResolvedThemeConfig::Local {
                    theme_dirs: local_dirs,
                }
            }
            ThemeConfig::Npm {
                name,
                version,
                author,
                theme_dirs,
            } => {
                let node_modules_path = cwd.join("node_modules").join(&name);
                let npm_dirs = ResolvedThemeDirs {
                    layouts_dir: node_modules_path.join(
                        theme_dirs
                            .clone()
                            .unwrap()
                            .layouts_dir
                            .unwrap_or(PathBuf::from("_layouts")),
                    ),
                    includes_dir: node_modules_path.join(
                        theme_dirs
                            .clone()
                            .unwrap()
                            .includes_dir
                            .unwrap_or(PathBuf::from("_includes")),
                    ),
                    assets_dir: node_modules_path.join(
                        theme_dirs
                            .clone()
                            .unwrap()
                            .assets_dir
                            .unwrap_or(PathBuf::from("_assets")),
                    ),
                    data_dir: node_modules_path.join(
                        theme_dirs
                            .clone()
                            .unwrap()
                            .data_dir
                            .unwrap_or(PathBuf::from("_data")),
                    ),
                };
                ResolvedThemeConfig::Npm {
                    name,
                    version,
                    author,
                    theme_dirs: npm_dirs,
                }
            }
        }
    }
}

#[napi(namespace = "config")]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ResolvedThemeConfig {
    Npm {
        /// Theme package name.
        name: String,
        /// Theme package version.
        version: String,
        /// Optional theme author.
        #[serde(default)]
        author: Option<String>,
        theme_dirs: ResolvedThemeDirs,
    },
    Local {
        theme_dirs: ResolvedThemeDirs,
    },
}

impl ResolvedThemeConfig {
    pub fn resolve<P: AsRef<Path>>(root_dir: P, theme: Option<ThemeConfig>) -> Self {
        resolve_theme_config(root_dir, theme)
    }

    pub fn get_layouts_dir(self) -> PathBuf {
        match self {
            ResolvedThemeConfig::Npm { theme_dirs, .. } => theme_dirs.layouts_dir,
            ResolvedThemeConfig::Local { theme_dirs } => theme_dirs.layouts_dir,
        }
    }
    pub fn get_includes_dir(self) -> PathBuf {
        match self {
            ResolvedThemeConfig::Npm { theme_dirs, .. } => theme_dirs.includes_dir,
            ResolvedThemeConfig::Local { theme_dirs } => theme_dirs.includes_dir,
        }
    }
    pub fn get_assets_dir(self) -> PathBuf {
        match self {
            ResolvedThemeConfig::Npm { theme_dirs, .. } => theme_dirs.assets_dir,
            ResolvedThemeConfig::Local { theme_dirs } => theme_dirs.assets_dir,
        }
    }
    pub fn get_data_dir(self) -> PathBuf {
        match self {
            ResolvedThemeConfig::Npm { theme_dirs, .. } => theme_dirs.data_dir,
            ResolvedThemeConfig::Local { theme_dirs } => theme_dirs.data_dir,
        }
    }
}
