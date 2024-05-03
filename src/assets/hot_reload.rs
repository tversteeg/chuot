//! Hot-reloading watcher thread and mechanism.

use hashbrown::HashSet;
use miette::{Context, IntoDiagnostic, Result};
use notify_debouncer_mini::{
    notify::{RecommendedWatcher, RecursiveMode},
    DebounceEventResult, DebouncedEventKind, Debouncer,
};
use web_time::Duration;

use std::{
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
};

use super::Id;

/// How long another change between multiple changes is triggered.
const DEBOUNCE_DELAY: Duration = Duration::from_millis(100);

/// Spawn a thread for watching filesystem events for assets.
///
/// Will stay alive as long as the handle is kept.
#[must_use]
pub(crate) fn watch_assets_folder(
    assets_dir: impl Into<PathBuf>,
) -> Result<Debouncer<RecommendedWatcher>> {
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
                        let id = match path_to_id(&event.path, &assets_dir) {
                            Ok(id_and_extension) => id_and_extension,
                            Err(err) => {
                                log::error!(
                                    "Error converting changed file asset path {:?} to ID: {err:?}",
                                    event.path
                                );
                                continue;
                            }
                        };

                        // Store in the global map of updated items
                        global_assets_updated().lock().unwrap().insert(id);
                    }
                }
                Err(err) => log::error!("Error while watching assets folder: {err}"),
            }
        })
        .into_diagnostic()
        .wrap_err("Error setting up notification debouncer for asset folder")?
    };

    // Watch the assets folder
    debouncer
        .watcher()
        .watch(&assets_dir, RecursiveMode::Recursive)
        .into_diagnostic()
        .wrap_err("Error setting up asset folder watcher")?;

    Ok(debouncer)
}

/// Convert a path to an ID.
fn path_to_id(path: &Path, assets_dir: &Path) -> Result<Id> {
    // Extract the extension, ignore the file if not found
    let extension = path
        .extension()
        .ok_or_else(|| miette::miette!("Error getting asset file {path:?} extension"))?
        .to_string_lossy();

    // Get the path relative to the asset dir
    let relative_path = path
        .strip_prefix(assets_dir)
        .into_diagnostic()
        .wrap_err("Error making path relative to asset dir")?;

    // Create an ID from the path
    let id = relative_path
        .iter()
        .map(|path| path.to_string_lossy())
        .collect::<Vec<_>>()
        .join(".");

    // Remove the extension
    let id = id
        .strip_suffix(&format!(".{}", extension))
        .expect("Error removing extension");

    Ok(Id::new(id))
}

/// Get a reference to the static map of all assets hat got updated and need to be reloaded.
pub(crate) fn global_assets_updated() -> &'static Mutex<HashSet<Id>> {
    /// Global list of files that got changed.
    static MAP: OnceLock<Mutex<HashSet<Id>>> = OnceLock::new();

    MAP.get_or_init(|| Mutex::new(HashSet::new()))
}
