//! Hot-reloading watcher thread and mechanism.

use hashbrown::HashSet;
use notify_debouncer_mini::{
    notify::{RecommendedWatcher, RecursiveMode},
    DebounceEventResult, DebouncedEventKind, Debouncer,
};

use std::{
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
    time::Duration,
};

use crate::context::ContextInner;

use super::Id;

/// How long another change between multiple changes is triggered.
const DEBOUNCE_DELAY: Duration = Duration::from_millis(100);

/// Remove reloaded assets from asset manager so they can be reloaded.
#[inline]
pub(crate) fn handle_changed_asset_files(ctx: &mut ContextInner) {
    // Remove each asset so they will be reloaded
    global_assets_updated()
        .lock()
        .unwrap()
        .drain()
        .for_each(|changed_asset| {
            ctx.remove(&changed_asset);
        });
}

/// Spawn a thread for watching filesystem events for assets.
///
/// Will stay alive as long as the handle is kept.
#[must_use]
pub(crate) fn watch_assets_folder(assets_dir: impl Into<PathBuf>) -> Debouncer<RecommendedWatcher> {
    let assets_dir = assets_dir.into();

    // Setup what must happen when an event is received
    let mut debouncer = {
        let assets_dir = assets_dir.clone();

        notify_debouncer_mini::new_debouncer(DEBOUNCE_DELAY, move |res: DebounceEventResult| {
            match res {
                Ok(events) => {
                    for event in events {
                        // Only check for events that are triggered once
                        if event.kind != DebouncedEventKind::AnyContinuous {
                            continue;
                        }

                        // Convert the changed file's path to ID
                        let Some(id) = path_to_id(&event.path, &assets_dir) else {
                            eprintln!(
                                "Error converting changed file asset path {} to ID",
                                event.path.display()
                            );

                            continue;
                        };

                        // Store in the global map of updated items
                        global_assets_updated().lock().unwrap().insert(id);
                    }
                }
                Err(err) => eprintln!("Error while watching assets folder: {err}"),
            }
        })
        .unwrap()
    };

    // Watch the assets folder
    debouncer
        .watcher()
        .watch(&assets_dir, RecursiveMode::Recursive)
        .unwrap();

    debouncer
}

/// Convert a path to an ID.
#[inline]
fn path_to_id(path: &Path, assets_dir: &Path) -> Option<Id> {
    // Extract the extension, ignore the file if not found
    let extension = path.extension()?.to_string_lossy();

    // Get the path relative to the asset dir
    let relative_path = path.strip_prefix(assets_dir).ok()?;

    // Create an ID from the path
    let id = relative_path
        .iter()
        .map(|path| path.to_string_lossy())
        .collect::<Vec<_>>()
        .join(".");

    // Remove the extension
    let id = id.strip_suffix(&format!(".{extension}"))?;

    Some(Id::new(id))
}

/// Get a reference to the static map of all assets hat got updated and need to be reloaded.
#[inline]
#[must_use]
fn global_assets_updated() -> &'static Mutex<HashSet<Id>> {
    /// Global list of files that got changed.
    static MAP: OnceLock<Mutex<HashSet<Id>>> = OnceLock::new();

    MAP.get_or_init(|| Mutex::new(HashSet::new()))
}
