//! Debounced filesystem watching for Node.js.
//!
//! [`FsWatcher`] reports create, modify, and remove events for watched paths.
//! Optional glob patterns can exclude paths, and the debounce window can be
//! configured when creating the watcher.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use globset::{Glob, GlobSet, GlobSetBuilder};
use napi::bindgen_prelude::*;
use napi::threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode};
use napi_derive::napi;
use notify_debouncer_full::notify::event::{ModifyKind, RenameMode};
use notify_debouncer_full::notify::{Event, EventKind, RecommendedWatcher, RecursiveMode};
use notify_debouncer_full::{DebounceEventResult, Debouncer, RecommendedCache, new_debouncer};

#[napi(object, namespace = "watch")]
/// A filesystem change reported to the JavaScript callback.
pub struct WatchEvent {
    /// Change type: `"create"`, `"modify"`, or `"remove"`.
    #[napi(ts_type = "'create' | 'modify' | 'remove'")]
    pub kind: String,
    /// Path associated with the change.
    pub path: String,
}

#[napi(object, namespace = "watch")]
/// Options for configuring an [`FsWatcher`].
pub struct WatchOptions {
    /// Glob patterns for paths to exclude from reported events.
    pub ignore: Option<Vec<String>>,
    /// Debounce window in milliseconds. Defaults to 100 ms.
    pub debounce_ms: Option<u32>,
}

#[napi(namespace = "watch")]
/// Watches filesystem paths and sends debounced changes to a JavaScript callback.
pub struct FsWatcher {
    debouncer: Option<Debouncer<RecommendedWatcher, RecommendedCache>>,
}

fn classify_event(event: Event, ignore_set: &GlobSet) -> Vec<WatchEvent> {
    let mut actions = Vec::new();

    let push_if_valid = |kind: &str, path: PathBuf, out: &mut Vec<WatchEvent>| {
        if !ignore_set.is_match(&path) {
            out.push(WatchEvent {
                kind: kind.to_string(),
                path: path.to_string_lossy().into_owned(),
            });
        }
    };

    match event.kind {
        EventKind::Create(_) => {
            for path in event.paths {
                push_if_valid("create", path, &mut actions);
            }
        }
        EventKind::Modify(ModifyKind::Name(rename_mode)) => match rename_mode {
            RenameMode::Both => {
                let mut iter = event.paths.into_iter();
                if let Some(old_path) = iter.next() {
                    push_if_valid("remove", old_path, &mut actions);
                }
                if let Some(new_path) = iter.next() {
                    push_if_valid("create", new_path, &mut actions);
                }
            }
            RenameMode::From => {
                for path in event.paths {
                    push_if_valid("remove", path, &mut actions);
                }
            }
            RenameMode::To => {
                for path in event.paths {
                    push_if_valid("create", path, &mut actions);
                }
            }
            _ => {
                for path in event.paths {
                    let kind = if path.exists() { "create" } else { "remove" };
                    push_if_valid(kind, path, &mut actions);
                }
            }
        },
        EventKind::Modify(_) => {
            for path in event.paths {
                push_if_valid("modify", path, &mut actions);
            }
        }
        EventKind::Remove(_) => {
            for path in event.paths {
                push_if_valid("remove", path, &mut actions);
            }
        }
        _ => {}
    }

    actions
}

#[napi(namespace = "watch")]
impl FsWatcher {
    #[napi(constructor, namespace = "watch")]
    /// Creates a watcher with a callback and optional configuration.
    ///
    /// The callback receives one [`WatchEvent`] per reported path. Invalid
    /// ignore patterns and watcher initialization failures are returned as
    /// errors.
    pub fn new(
        callback: Arc<ThreadsafeFunction<WatchEvent, ()>>,
        options: Option<WatchOptions>,
    ) -> Result<Self> {
        let mut debounce_ms = 100u64;
        let mut builder = GlobSetBuilder::new();

        if let Some(opts) = options {
            if let Some(ms) = opts.debounce_ms {
                debounce_ms = ms as u64;
            }
            if let Some(patterns) = opts.ignore {
                for pat in patterns {
                    let glob = Glob::new(&pat).map_err(|e| {
                        Error::from_reason(format!("Invalid glob '{}': {}", pat, e))
                    })?;
                    builder.add(glob);
                }
            }
        }

        let ignore_set = Arc::new(
            builder
                .build()
                .map_err(|e| Error::from_reason(e.to_string()))?,
        );

        // Passing `None` for tick_rate defaults to 1/4 of the timeout duration
        let debouncer = new_debouncer(
            Duration::from_millis(debounce_ms),
            None,
            move |result: DebounceEventResult| match result {
                Ok(debounced_events) => {
                    for debounced in debounced_events {
                        for payload in classify_event(debounced.event, &ignore_set) {
                            callback.call(Ok(payload), ThreadsafeFunctionCallMode::NonBlocking);
                        }
                    }
                }
                Err(errors) => {
                    for err in errors {
                        let napi_err = Error::from_reason(err.to_string());
                        callback.call(Err(napi_err), ThreadsafeFunctionCallMode::NonBlocking);
                    }
                }
            },
        )
        .map_err(|e| Error::from_reason(e.to_string()))?;

        Ok(Self {
            debouncer: Some(debouncer),
        })
    }

    #[napi(namespace = "watch")]
    /// Starts watching one path or a list of paths.
    ///
    /// Watching is recursive by default. Set `recursive` to `false` to watch
    /// only the specified paths' immediate entries.
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

        if let Some(debouncer) = &mut self.debouncer {
            for target in paths {
                debouncer.watch(Path::new(&target), mode).map_err(|e| {
                    Error::from_reason(format!("Failed to watch '{}': {}", target, e))
                })?;
            }
        }
        Ok(())
    }

    #[napi(namespace = "watch")]
    /// Stops watching one path or a list of paths.
    pub fn unwatch(&mut self, targets: Either<String, Vec<String>>) -> Result<()> {
        let paths = match targets {
            Either::A(single) => vec![single],
            Either::B(multiple) => multiple,
        };

        if let Some(debouncer) = &mut self.debouncer {
            for target in paths {
                debouncer
                    .unwatch(Path::new(&target))
                    .map_err(|e| Error::from_reason(e.to_string()))?;
            }
        }
        Ok(())
    }

    #[napi(namespace = "watch")]
    /// Stops the watcher and releases its callback.
    pub fn close(&mut self) {
        // Dropping the debouncer stops its background worker thread and releases the callback
        self.debouncer.take();
    }
}
