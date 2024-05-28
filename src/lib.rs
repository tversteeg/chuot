#![forbid(unsafe_code)]

//! AGPL licensed and opinionated game engine for 2D pixel-art games.
//!
//! # Features
//!
//! - Pixel-perfect pixel art rendering with built-in rotsprite rotation shader.
//! - Window creation with independent update and render game loop.
//! - Hot-reloadable assets, seeing your assets update live in the game when you save them is a great boost in productivity for quickly iterating on ideas.
//! - Single-binary, all non-texture assets should be embedded directly, and textures should be diced into a single atlas map embedded in the binary when deploying.
//! - Simple bitmap font drawing.
//! - Dialogue scripting system.
//! - OGG audio playback.
//! - First-class gamepad support.
//! - In game CPU & memory profiler GUI.
//!
//! # Goals
//!
//! - Ergonomic API with a focus on quickly creating small games, especially for game jams.
//! - Reasonable performance, drawing thousands of animated sprites at the same time shouldn't be a problem.
//! - Proper web support, it should be very easy to bundle as WASM for the web.
//!
//! # Non-Goals
//!
//! - An ECS (Entity component system), although an ECS architecture is great for cache locality and performance, I feel that it's overkill for most small games. Nothing is stopping you to add your own on top of this engine if that's what you want though!
//! - 3D, this engine is only for 2D pixel art.
//! - Vector graphics, similar to the above, this engine is focused specifically on pixel art with lower resolutions.
//! - Reinventing the wheel for everything, when there's a proper crate with good support I prefer to use that instead of creating additional maintainer burden.
//! - Support all possible file formats, this bloats the engine.
//!
//! # Usage
//!
//! Using this crate is quite simple, there is a single trait [`PixelGame`] with two required functions, [`PixelGame::update`] and [`PixelGame::render`], that need to be implemented for a game state object.
//!
//! ```no_run
//! use chuot::{PixelGame, Context, GameConfig};
//!
//! struct MyGame;
//!
//! impl PixelGame for MyGame {
//!   fn update(&mut self, ctx: Context) {
//!     // ..
//!   }
//!
//!   fn render(&mut self, ctx: Context) {
//!     // ..
//!   }
//! }
//!
//! # fn try_main() -> miette::Result<()> {
//! // In main
//!
//! let game = MyGame;
//!
//! game.run(chuot::load_assets!(), GameConfig::default())?;
//! # Ok(()) }
//! # try_main().unwrap();
//! ```
//!
//! ## `embed-assets`
//!
//! Embed all assets into the binary when building.
//!
//! _Must_ be enabled when building for the web.
//! If disabled all assets will be loaded from disk.
//!
//! This will dice all PNG assets into a single tiny optimized PNG atlas.
//! On startup this diced atlas will be efficiently uploaded to the GPU as a single bigger atlas, which will be used for all static sprites.
//!
//! ## `hot-reload-assets`
//!
//! Hot-reload assets from disk when they are saved.
//! Has no effect on the web target.
//! If disabled _all_ assets will be baked into the binary.
//!
//! ## `dialogue` (default)
//!
//! A thin wrapper around [Yarn Spinner](https://www.yarnspinner.dev/).
//! Allows creating hot-reloadable dialogue systems.
//!
//! ## `in-game-profiler`
//!
//! A profiler window overlay, implemented with [puffin_egui](https://docs.rs/puffin_egui/latest/puffin_egui/).
//!
//! Other profiling methods in your game can also be implemented, the [profiling](https://docs.rs/profiling/latest/profiling/) crate is enabled even when this feature flag is disabled.
//!
//! # Install Requirements
//!
//! On Linux you need to install `asound2-dev` for audio and `udev-dev` for gamepads:
//!
//! ```sh
//! sudo apt install libasound2-dev libudev-dev
//! ```
//!
//! # Example
//!
//! This example will show a window with a counter that's incremented when pressing the left mouse button[^left-mouse].
//! The counter is rendered as text[^text] loaded from a font in the top-left corner.
//! When the 'Escape' key is pressed[^escape-key] the game will exit and the window will close.
//!
//! ```no_run
//! use chuot::{PixelGame, Context, GameConfig, MouseButton, KeyCode, glamour::Vector2};
//!
//! /// Object holding all game state.
//! struct MyGame {
//!   /// A simple counter we increment by clicking on the screen.
//!   counter: u32,
//! }
//!
//! impl PixelGame for MyGame {
//!   fn update(&mut self, ctx: Context) {
//!     // ^1
//!     // Increment the counter when we press the left mouse button
//!     if ctx.mouse_pressed(MouseButton::Left) {
//!       self.counter += 1;
//!     }
//!
//!     // ^3
//!     // Exit the game if 'Escape' is pressed
//!     if ctx.key_pressed(KeyCode::Escape) {
//!       ctx.exit();
//!     }
//!   }
//!
//!   fn render(&mut self, ctx: Context) {
//!     // ^2
//!     // Display the counter with a font called 'font' automatically loaded from the `assets/` directory
//!     // It will be shown in the top-left corner
//!     ctx.text("font", &format!("Counter: {}", self.counter)).draw();
//!   }
//! }
//!
//! # fn try_main() -> miette::Result<()> {
//! // In main
//!
//! // Initialize the game state
//! let game = MyGame { counter: 0 };
//!
//! // Run the game until exit is requested
//! game.run(chuot::load_assets!(), GameConfig::default().with_title("My Game"))?;
//! # Ok(()) }
//! # try_main().unwrap();
//! ```
//!
//! [^left-mouse]: [`Context::mouse_pressed`]
//! [^text]: [`Context::text`]
//! [^escape-key]: [`Context::key_pressed`]

// Check for various illegal target and feature combinations
#[cfg(all(feature = "embed-assets", feature = "hot-reload-assets"))]
compile_error!("Features `embed-assets` and `hot-reload-assets` are mutually exclusive and can't be enabled at the same time!");
#[cfg(all(not(feature = "embed-assets"), target_arch = "wasm32"))]
compile_error!("Feature `embed-assets` must be enabled when building for the web!");

pub mod assets;
pub mod config;
pub mod context;
#[cfg(feature = "dialogue")]
pub mod dialogue;
pub(crate) mod font;
pub(crate) mod graphics;
pub(crate) mod random;
pub(crate) mod sprite;
pub(crate) mod window;

/// Re-exported gilrs type used in [`Context`].
pub use gilrs::ev::{Axis, Button};
/// Re-exported vector math type.
pub use glamour;
/// Re-exported winit type used in [`Context`].
pub use winit::{
    dpi::PhysicalSize,
    event::MouseButton,
    keyboard::{Key, KeyCode},
};

/// Define the directory of the assets.
///
/// *MUST* be passed as first argument to [`PixelGame::run`].
///
/// The assets will be embedded in the binary when not using the `hot-reload-assets` feature flag.
///
/// # Arguments
///
/// * `path` - Local directory where the game assets reside. Defaults to `"assets/"`.
///
/// # Example
///
/// ```
/// chuot::load_assets!("assets/");
/// // Is the same as..
/// chuot::load_assets!();
/// ```
pub use chuot_macros::load_assets;

pub use config::GameConfig;
pub use context::Context;
pub use random::random;

use assets::EmbeddedAssets;
use miette::Result;

/// Main entrypoint containing game state for running the game.
///
/// This is the main interface with the game engine.
///
/// See [`Context`] for all functions interfacing with the game engine from both functions.
pub trait PixelGame: Sized
where
    Self: 'static,
{
    /// A single update tick in the game loop.
    ///
    /// Will run on a different rate from the render loop specified in the game configuration.
    ///
    /// Must be used for updating the game state.
    /// It's possible to queue draw calls on the update context since that's the same object as render, but that will result in very erratic drawing since the render loop is uncoupled from the update loop.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Game context, used to obtain information and mutate the game state.
    ///
    /// # Example
    ///
    /// ```
    /// use chuot::{PixelGame, Context, GameConfig, KeyCode};
    ///
    /// struct MyGame;
    ///
    /// impl PixelGame for MyGame {
    ///   fn update(&mut self, ctx: Context) {
    ///     // Stop the game and close the window when 'Escape' is pressed
    ///     if ctx.key_pressed(KeyCode::Escape) {
    ///       ctx.exit();
    ///     }
    ///   }
    ///
    ///   fn render(&mut self, ctx: Context) {
    ///     // ..
    ///   }
    /// }
    /// ```
    fn update(&mut self, ctx: Context);

    /// A single render tick in the game loop.
    ///
    /// Will run on a different rate from the update loop specified in the game configuration.
    ///
    /// Must be used for rendering the game.
    /// Be careful with mutating game state here, when it's dependent on external state the result will be erratic and incorrect, such as handling any form of input.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Game context, used to obtain information and queue draw calls.
    ///
    /// # Example
    ///
    /// ```
    /// use chuot::{PixelGame, Context, GameConfig, KeyCode};
    ///
    /// struct MyGame;
    ///
    /// impl PixelGame for MyGame {
    ///   fn render(&mut self, ctx: Context) {
    ///     // Draw a sprite on the screen
    ///     ctx.sprite("sprite").draw();
    ///   }
    ///
    ///   fn update(&mut self, ctx: Context) {
    ///     // ..
    ///   }
    /// }
    /// ```
    fn render(&mut self, ctx: Context);

    /// Run the game, spawning the window.
    ///
    /// # Arguments
    ///
    /// * `assets` - Source of the assets, needs to be `chuot::load_assets!()`.
    /// * `game_config` - Configuration for the window, can be used to set the buffer size, the window title and other things.
    ///
    /// # Errors
    ///
    /// - When a window could not be opened (desktop only).
    /// - When `hot-reload-assets` feature is enabled and the assets folder could not be watched.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use chuot::{PixelGame, Context, GameConfig, KeyCode};
    ///
    /// struct MyGame;
    ///
    /// impl PixelGame for MyGame {
    ///   fn update(&mut self, ctx: Context) {
    ///     // Stop the game and close the window when 'Escape' is pressed
    ///     if ctx.key_pressed(KeyCode::Escape) {
    ///       ctx.exit();
    ///     }
    ///   }
    ///
    ///   fn render(&mut self, ctx: Context) {
    ///     // ..
    ///   }
    /// }
    ///
    /// # fn try_main() -> miette::Result<()> {
    /// // In main
    /// let game = MyGame;
    ///
    /// game.run(chuot::load_assets!(), GameConfig::default())?;
    /// # Ok(()) }
    /// # try_main().unwrap();
    /// ```
    #[inline]
    fn run(self, assets: EmbeddedAssets, game_config: GameConfig) -> Result<()> {
        // Spawn the window with the game loop
        window::window(self, game_config, Self::update, Self::render, assets)
    }
}
