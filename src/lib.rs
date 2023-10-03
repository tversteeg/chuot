//! Utility library for games.
//!
//! # Features
//!
//! ### `window` (default)
//!
//! Creates a desktop window and a WASM based web "window" for drawing pixels.
//! Runs a configurable game loop which splits a render and an update function.
//!
//! ### `font` (default)
//!
//! Render a simple ASCII bitmap font.
//!
//! ### `default-font`
//!
//! Implements [`Default`] for [`font::Font`] with a font that's embedded into memory.

#[cfg(feature = "window")]
mod window;
#[cfg(feature = "window")]
pub use window::{window, WindowConfig};

#[cfg(feature = "font")]
pub mod font;

/// Re-export vek types.
pub use vek::*;
/// Re-export winit types.
#[cfg(feature = "window")]
pub use winit::{dpi::PhysicalSize, event::VirtualKeyCode as Key};
/// Re-export winit_input_helper key type.
#[cfg(feature = "window")]
pub use winit_input_helper::TextChar;
