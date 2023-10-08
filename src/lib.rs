//! Utility library for games.
//!
//! # Features
//!
//! ### `window` (default)
//!
//! Creates a desktop window and a WASM based web "window" for drawing pixels.
//! Runs a configurable game loop which splits a render and an update function.
//!
//! ### `gui`
//!
//! Draw GUI widgets with flexbox based layouting using the [`taffy`](https://docs.rs/taffy) crate.
//!
//! ### `default-font`
//!
//! Implements [`Default`] for [`font::Font`] with a font that's embedded into memory.
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

#[cfg(any(feature = "hot-reloading-assets", feature = "embedded-assets"))]
pub mod assets;
#[cfg(any(feature = "hot-reloading-assets", feature = "embedded-assets"))]
pub use assets::{asset, asset_owned};

#[cfg(feature = "gui")]
pub mod gui;

/// Re-export taffy types.
#[cfg(feature = "gui")]
pub use taffy;
/// Re-export vek types.
pub use vek;
