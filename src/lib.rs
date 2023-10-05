//! Utility library for games.
//!
//! # Features
//!
//! ### `window` (default)
//!
//! Creates a desktop window and a WASM based web "window" for drawing pixels.
//! Runs a configurable game loop which splits a render and an update function.

//! ### `default-font`
//!
//! Implements [`Default`] for [`font::Font`] with a font that's embedded into memory.
//!
//! ### `assets` (default)
//!
//! Allow loading of external assets.
//! All assets should reside in a `assets/` folder in the root directory of the project.
//!
//! ### `hot-reloading-assets` (default)
//!
//! Hot-reload assets from disk when they are saved.
//! Has no effect on the web target.
//!
//! ### `embedded-assets` (default on web)
//!
//! Bake _all_ assets in the `assets/` folder in the binary.
//! When creating a release binary this feature flag should be enabled.

pub mod font;
pub mod sprite;

#[cfg(feature = "window")]
pub mod window;
#[cfg(feature = "window")]
pub use window::window;

#[cfg(feature = "assets")]
pub mod assets;
#[cfg(feature = "assets")]
pub use assets::{asset, asset_owned};

/// Re-export vek types.
pub use vek;
