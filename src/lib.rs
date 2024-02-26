//! Utility library for games, not a game engine.
//!
//! # Features
//!
//! - Window creation with game loop and pixel buffer.
//! - Asset management.
//! - Bitmap font drawing.
//! - Sprite loading.
//! - Simple GUI.
//! - Physics engine.
//!
//! # Feature Flags
//!
//! ### `default-font`
//!
//! Implements [`Default`] for [`font::Font`] with a font that's embedded into memory.
//!
//! ### `default-gui`
//!
//! Implements [`Default`] for different GUI elements with a images embedded into memory.
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
//!
//! ### `physics`
//!
//! Enable the 2D XPBD-based physics engine.
//!
//! ### `dialogue`
//!
//! A thin wrapper around [Yarn Spinner](https://www.yarnspinner.dev/).
//! Allows creating hot-reloadable dialogue systems.

pub mod canvas;
pub mod font;
pub mod sprite;

pub mod window;
pub use window::window;

pub mod assets;
pub use assets::{asset, asset_owned};

pub mod gui;

pub mod math;

#[cfg(feature = "physics")]
pub mod physics;

#[cfg(feature = "dialogue")]
pub mod dialogue;

/// Re-export taffy types.
pub use taffy;
/// Re-export vek types.
pub use vek;
