//! Game engine with library features that can be used standalone.
//!
//! # Features
//!
//! - Window creation with game loop and pixel buffer.
//! - Asset management.
//! - Bitmap font drawing.
//! - Sprite loading.
//! - Simple GUI.
//! - Physics engine.
//! - Audio playback.
//!
//! # Feature Flags
//!
//! ## `default-font`
//!
//! Implements [`Default`] for [`font::Font`] with a font that's embedded into memory.
//!
//! ## `default-gui`
//!
//! Implements [`Default`] for different GUI elements with a images embedded into memory.
//!
//! ## `hot-reloading-assets` (default)
//!
//! Hot-reload assets from disk when they are saved.
//! Has no effect on the web target.
//!
//! ## `embedded-assets` (default on web)
//!
//! Bake _all_ assets in the `assets/` folder in the binary.
//! When creating a release binary this feature flag should be enabled.
//!
//! ## `physics`
//!
//! Enable the 2D XPBD-based physics engine.
//!
//! ## `dialogue`
//!
//! A thin wrapper around [Yarn Spinner](https://www.yarnspinner.dev/).
//! Allows creating hot-reloadable dialogue systems.
//!
//! ## `audio`
//!
//! A thin wrapper around [Kira](https://docs.rs/kira/latest/kira/).
//! Play sounds and music files which can be hot-reloadable using assets.
//!
//! To keep the binary and compile-times small only `.ogg` audio files are supported.
//!
//! ### Requirements
//!
//! On Linux you need to install `asound2-dev`:
//!
//! ```sh
//! sudo apt install libasound2-dev
//! ```

pub mod assets;
#[cfg(feature = "audio")]
pub mod audio;
pub mod canvas;
#[cfg(feature = "dialogue")]
pub mod dialogue;
pub mod font;
pub mod gui;
pub mod math;
#[cfg(feature = "physics")]
pub mod physics;
pub mod sprite;
pub mod window;

pub use assets::{asset, asset_owned};
/// Re-export taffy types.
pub use taffy;
/// Re-export vek types.
pub use vek;
pub use window::window;

use canvas::Canvas;
use miette::Result;
use vek::Vec2;
use window::{Input, WindowConfig};

/// Setup a game with a shared state and run it.
///
/// This is only a helper for constructing a global game state around the [`window`] function, which can also be easily used standalone.
pub trait PixelGame: Sized
where
    Self: 'static,
{
    /// Update loop, called every update tick.
    ///
    /// # Arguments
    ///
    /// * `input` - Input helper that can be used to handle different input states.
    /// * `mouse_pos` - Mouse position on the buffer if `Some`, if `None` mouse is outside of the buffer, not necessarily the window.
    /// * `dt` - Delta time, time in seconds since the last update call. Can be used to handle physics.
    ///
    /// # Returns
    ///
    /// * `true` if the window and thus the game should be closed
    fn update(&mut self, input: &Input, mouse_pos: Option<Vec2<usize>>, dt: f32) -> bool;

    /// Render loop, called every render tick.
    ///
    /// # Arguments
    ///
    /// * `canvas` - Pixel buffer where all the graphics are rendered on.
    fn render(&mut self, canvas: &mut Canvas<'_>);

    /// Run the game, spawning the window.
    ///
    /// # Arguments
    ///
    /// * `window_config` - Configuration for the window, can be used to set the buffer size, the window title and other things.
    fn run(self, window_config: WindowConfig) -> Result<()> {
        window::window(
            self,
            window_config,
            |state, input, mouse_pos, dt| state.update(input, mouse_pos, dt),
            |state, canvas, _dt| state.render(canvas),
        )
    }
}
