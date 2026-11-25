use std::path::Path;
use std::sync::Arc;

use globset::{Glob, GlobSet, GlobSetBuilder};
use napi::bindgen_prelude::*;
use napi::threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode};
use napi_derive::napi;
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};

#[napi(object, namespace = "watcher")]
pub struct WatchEvent {
    pub kind: String,
    pub paths: Vec<String>,
}

#[napi(object, namespace = "watcher")]
pub struct WatchOptions {
    pub ignore: Option<Vec<String>>,
}

#[napi(namespace = "watcher")]
pub struct FsWatcher {
    watcher: Option<RecommendedWatcher>,
}

#[napi(namespace = "watcher")]
impl FsWatcher {
    #[napi(constructor, namespace = "watcher")]
    pub fn new(
        callback: Arc<ThreadsafeFunction<WatchEvent, ()>>,
        options: Option<WatchOptions>,
    ) -> Result<Self> {
        // 1. Compile ignore glob patterns once
        let mut builder = GlobSetBuilder::new();
        if let Some(opts) = options {
            if let Some(patterns) = opts.ignore {
                for pat in patterns {
                    let glob = Glob::new(&pat).map_err(|e| {
                        Error::from_reason(format!("Invalid glob '{}': {}", pat, e))
                    })?;
                    builder.add(glob);
                }
            }
        }
        let ignore_set: Arc<GlobSet> = Arc::new(
            builder
                .build()
                .map_err(|e| Error::from_reason(e.to_string()))?,
        );

        // 2. Filter ignored paths inside the background OS thread before crossing to JS
        let watcher = notify::recommended_watcher(move |res: notify::Result<Event>| {
            match res {
                Ok(event) => {
                    let filtered_paths: Vec<String> = event
                        .paths
                        .into_iter()
                        .filter(|p| !ignore_set.is_match(p))
                        .map(|p| p.to_string_lossy().into_owned())
                        .collect();

                    // Skip invoking JS callback if all paths in the event were ignored
                    if filtered_paths.is_empty() {
                        return;
                    }

                    let payload = WatchEvent {
                        kind: format!("{:?}", event.kind),
                        paths: filtered_paths,
                    };
                    callback.call(Ok(payload), ThreadsafeFunctionCallMode::NonBlocking);
                }
                Err(err) => {
                    let napi_err = Error::from_reason(err.to_string());
                    callback.call(Err(napi_err), ThreadsafeFunctionCallMode::NonBlocking);
                }
            }
        })
        .map_err(|e| Error::from_reason(e.to_string()))?;

        Ok(Self {
            watcher: Some(watcher),
        })
    }

    /// Accepts a single file/folder path OR an array of files and folders
    #[napi(namespace = "watcher")]
    pub fn watch(
        &mut self,
        targets: Either<String, Vec<String>>,
        recursive: Option<bool>,
    ) -> Result<()> {
        let mode = if recursive.unwrap_or(true) {
            RecursiveMode::Recursive
        } else {
            RecursiveMode::NonRecursive
        };

        let paths = match targets {
            Either::A(single) => vec![single],
            Either::B(multiple) => multiple,
        };

        if let Some(watcher) = &mut self.watcher {
            for target in paths {
                watcher.watch(Path::new(&target), mode).map_err(|e| {
                    Error::from_reason(format!("Failed to watch '{}': {}", target, e))
                })?;
            }
        }
        Ok(())
    }

    #[napi(namespace = "watcher")]
    pub fn unwatch(&mut self, targets: Either<String, Vec<String>>) -> Result<()> {
        let paths = match targets {
            Either::A(single) => vec![single],
            Either::B(multiple) => multiple,
        };

        if let Some(watcher) = &mut self.watcher {
            for target in paths {
                watcher
                    .unwatch(Path::new(&target))
                    .map_err(|e| Error::from_reason(e.to_string()))?;
            }
        }
        Ok(())
    }

    #[napi(namespace = "watcher")]
    pub fn close(&mut self) {
        self.watcher.take();
    }
}
