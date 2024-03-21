//! AGPL licensed and opinionated game engine for pixel-art games.
//!
//! # Features
//!
//! - Pixel-perfect pixel art rendering with built-in rotsprite rotation shader.
//! - Window creation with independent update and render game loop.
//! - Hot-reloadable asset management.
//! - Sprite loading.
//! - Dialogue scripting system.
//! - Audio playback.
//! - In game profiler GUI.
//!
//! # Usage
//!
//! Using this crate is quite simple, there is a single trait [`PixelGame`] with a single required function, [`PixelGame::tick`] that needs to be implemented for a state.
//!
//! ```no_run
//! use pixel_game_lib::{PixelGame, Context, WindowConfig};
//!
//! struct MyGame;
//!
//! impl PixelGame for MyGame {
//!   fn tick(&mut self, ctx: Context) {
//!     // ..
//!   }
//! }
//!
//! # fn try_main() -> miette::Result<()> {
//! // In main
//! let game = MyGame;
//!
//! game.run(WindowConfig::default())?;
//! # Ok(()) }
//! # try_main().unwrap();
//! ```
//!
//! # Feature Flags
//!
//! All major feature flags are enabled by default, I would recommend installing `pixel_game_lib` with `default-features = false` and adding the required features as needed.
//!
//! ```sh
//! cargo add pixel_game_lib --no-default-features
//! ```
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
//! ## `dialogue` (default)
//!
//! A thin wrapper around [Yarn Spinner](https://www.yarnspinner.dev/).
//! Allows creating hot-reloadable dialogue systems.
//!
//! ## `audio` (default)
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
//!
//! ## `in-game-profiler` (default)
//!
//! A profiler window overlay, implemented with [puffin_egui](https://docs.rs/puffin_egui/latest/puffin_egui/).
//!
//! Other profiling methods in your game can also be implemented, the [profiling](https://docs.rs/profiling/latest/profiling/) crate is enabled even when this feature flag is disabled.

pub mod assets;
#[cfg(feature = "audio")]
pub mod audio;
mod context;
#[cfg(feature = "dialogue")]
pub mod dialogue;
pub mod graphics;
mod sprite;
mod window;

use assets_manager::{AssetReadGuard, Compound};
/// Re-exported vector math type.
pub use glamour;
/// Re-exported winit type used in [`Context`].
pub use winit::{
    dpi::PhysicalSize,
    event::MouseButton,
    keyboard::{Key, KeyCode},
};

pub use context::Context;
pub use window::WindowConfig;

use miette::Result;

/// Setup a game with a shared state and run it.
///
/// This is only a helper for constructing a global game state around the [`window`] function, which can also be easily used standalone.
pub trait PixelGame: Sized
where
    Self: 'static,
{
    /// A single tick in the game loop.
    ///
    /// Must be used for rendering and updating the game state.
    ///
    /// # Arguments
    ///
    /// * `context` - Game context, used to queue draw calls and obtain information about the state.
    ///
    /// # Example
    ///
    /// ```
    /// use pixel_game_lib::{PixelGame, Context, WindowConfig, KeyCode};
    ///
    /// struct MyGame;
    ///
    /// impl PixelGame for MyGame {
    ///   fn tick(&mut self, ctx: Context) {
    ///     // Stop the game and close the window when 'Escape' is pressed
    ///     if ctx.key_pressed(KeyCode::Escape) {
    ///       ctx.exit();
    ///     }
    ///   }
    /// }
    fn tick(&mut self, context: Context);

    /// Run the game, spawning the window.
    ///
    /// # Arguments
    ///
    /// * `window_config` - Configuration for the window, can be used to set the buffer size, the window title and other things.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use pixel_game_lib::{PixelGame, Context, WindowConfig};
    ///
    /// struct MyGame;
    ///
    /// impl PixelGame for MyGame {
    ///   fn tick(&mut self, ctx: Context) {
    ///     // ..
    ///   }
    /// }
    ///
    /// # fn try_main() -> miette::Result<()> {
    /// // In main
    /// let game = MyGame;
    ///
    /// game.run(WindowConfig::default())?;
    /// # Ok(()) }
    /// # try_main().unwrap();
    /// ```
    fn run(self, window_config: WindowConfig) -> Result<()> {
        // Setup the audio
        #[cfg(feature = "audio")]
        crate::audio::init_audio()?;

        // Spawn the window with the game loop
        window::window(self, window_config, |state, ctx| state.tick(ctx))
    }
}

/// Load a reference to any non-renderable asset.
///
/// Sets up the asset manager once, which can be accessed with the global function in this module.
///
/// # Arguments
///
/// * `path` - Directory structure of the asset file in `assets/` where every `/` is a `.`.
///
/// # Panics
///
/// - When asset with path does not exist.
/// - When asset could not be loaded to to an invalid format.
pub fn asset<T>(path: impl AsRef<str>) -> AssetReadGuard<'static, T>
where
    T: Compound,
{
    profiling::scope!("Load asset");

    assets::asset_cache().load_expect(path.as_ref()).read()
}

/// Load a clone of any non-renderable asset.
///
/// Sets up the asset manager once, which can be accessed with the global function in this module.
///
/// # Arguments
///
/// * `path` - Directory structure of the asset file in `assets/` where every `/` is a `.`.
///
/// # Panics
///
/// - When asset with path does not exist.
/// - When asset could not be loaded to to an invalid format.
pub fn asset_owned<T>(path: impl AsRef<str>) -> T
where
    T: Compound,
{
    profiling::scope!("Load owned asset");

    assets::asset_cache()
        .load_owned(path.as_ref())
        .expect("Could not load owned asset")
}
