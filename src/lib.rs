#![forbid(unsafe_code)]

pub mod backend;
pub mod config;
pub mod context;

use backend::{wgpu::WgpuWinitBackend, Backend};
use config::GameConfig;
use context::Context;

/// Main entrypoint containing game state for running the game.
///
/// This is the main interface with the game engine.
///
/// See [`Context`] for all functions interfacing with the game engine from both functions.
pub trait PixelGame<B = WgpuWinitBackend>: Sized
where
    Self: 'static,
    B: Backend,
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
    /// use chuot::{Context, GameConfig, KeyCode, PixelGame};
    ///
    /// struct MyGame;
    ///
    /// impl PixelGame for MyGame {
    ///     fn update(&mut self, ctx: Context) {
    ///         // Stop the game and close the window when 'Escape' is pressed
    ///         if ctx.key_pressed(KeyCode::Escape) {
    ///             ctx.exit();
    ///         }
    ///     }
    ///
    ///     fn render(&mut self, ctx: Context) {
    ///         // ..
    ///     }
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
    /// use chuot::{Context, GameConfig, KeyCode, PixelGame};
    ///
    /// struct MyGame;
    ///
    /// impl PixelGame for MyGame {
    ///     fn render(&mut self, ctx: Context) {
    ///         // Draw a sprite on the screen
    ///         ctx.sprite("sprite").draw();
    ///     }
    ///
    ///     fn update(&mut self, ctx: Context) {
    ///         // ..
    ///     }
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
    /// use chuot::{Context, GameConfig, KeyCode, PixelGame};
    ///
    /// struct MyGame;
    ///
    /// impl PixelGame for MyGame {
    ///     fn update(&mut self, ctx: Context) {
    ///         // Stop the game and close the window when 'Escape' is pressed
    ///         if ctx.key_pressed(KeyCode::Escape) {
    ///             ctx.exit();
    ///         }
    ///     }
    ///
    ///     fn render(&mut self, ctx: Context) {
    ///         // ..
    ///     }
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
    fn run(self, game_config: GameConfig) {
        // Construct the backend
        let backend = B::new(&game_config);

        // Create the context with this backend
        let context = Context::new(backend);

        B::run(self, game_config);
    }
}
