//! Utility library for games.
//!
//! # Features
//!
//! ## `window`
//!
//! Creates a desktop window and a WASM based web "window" for drawing pixels.
//! Runs a configurable game loop which splits a render and an update function.

#[cfg(feature = "window")]
pub mod window;

/// Re-export vek.
pub use vek;
