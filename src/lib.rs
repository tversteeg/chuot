#![forbid(unsafe_code)]

pub mod assets;
pub mod config;
pub mod context;
mod graphics;
mod input;
mod random;

pub use config::Config;
pub use context::Context;
pub use random::random;

use std::time::Instant;

use winit::{
    application::ApplicationHandler,
    dpi::{LogicalSize, PhysicalSize},
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{WindowAttributes, WindowId},
};

/// How fast old FPS values decay in the smoothed average.
const FPS_SMOOTHED_AVERAGE_ALPHA: f32 = 0.8;

/// Main entrypoint containing game state for running the game.
///
/// This is the main interface with the game engine.
///
/// See [`Context`] for all functions interfacing with the game engine from both functions.
pub trait Game: Sized
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
    /// * `config` - Configuration for the window, can be used to set the buffer size, the window title and other things.
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
    fn run(self, config: Config) {
        run(self, config);
    }
}

/// State of setting up a window that can still be uninitialized.
///
/// All optional fields are tied to the window creation flow of winit.
struct State<G: Game> {
    /// Game context.
    ///
    /// `None` if the window still needs to be initialized.
    ctx: Option<Context>,
    /// User supplied game.
    game: G,
    /// User supplied configuration.
    config: Config,
    /// Time for calculating the update rate.
    last_time: Instant,
    /// Timestep accumulator for the update rate.
    accumulator: f32,
}

impl<G: Game> ApplicationHandler<()> for State<G> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Setup the window
        if self.ctx.is_none() {
            // Spawn a new window using the event loop
            let window = event_loop
                .create_window(
                    WindowAttributes::default()
                        .with_title(&self.config.title)
                        // Apply scaling for the requested size
                        .with_inner_size(LogicalSize::new(
                            self.config.buffer_width * self.config.scaling,
                            self.config.buffer_height * self.config.scaling,
                        ))
                        // Don't allow the window to be smaller than the pixel size
                        .with_min_inner_size(LogicalSize::new(
                            self.config.buffer_width,
                            self.config.buffer_height,
                        )),
                )
                .expect("Error creating window");

            // Setup the context
            let context =
                pollster::block_on(async { Context::new(self.config.clone(), window).await });

            self.ctx = Some(context);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        // Do nothing if the window is not set up yet
        let Some(ctx) = &mut self.ctx else {
            return;
        };

        // Handle the window events
        match event {
            // Handle the update loop and render loop
            WindowEvent::RedrawRequested => {
                // Update the timestep
                let current_time = Instant::now();
                let frame_time = (current_time - self.last_time).as_secs_f32();
                self.last_time = current_time;

                self.accumulator += frame_time
                    // Ensure the frametime will never surpass this amount
                    .min(self.config.max_frame_time_secs);

                // Call the user update function with the context
                while self.accumulator >= self.config.update_delta_time {
                    // Call the implemented update function on the 'PixelGame' trait
                    self.game.update(ctx.clone());

                    // Mark this tick as executed
                    self.accumulator -= self.config.update_delta_time;

                    ctx.write(|ctx| {
                        // Update the input so pressed and released events can be handled
                        ctx.input.update();
                    });
                }

                ctx.write(|ctx| {
                    // Set the blending factor
                    ctx.blending_factor = self.accumulator / self.config.update_delta_time;

                    // Set the FPS with a smoothed average function
                    ctx.frames_per_second = FPS_SMOOTHED_AVERAGE_ALPHA.mul_add(
                        ctx.frames_per_second,
                        (1.0 - FPS_SMOOTHED_AVERAGE_ALPHA) * frame_time.recip(),
                    );
                });

                // Call the user render function with the context
                self.game.render(ctx.clone());

                ctx.write(|ctx| {
                    // Draw the window and GPU graphics
                    ctx.graphics.render();

                    // Request another frame for the window
                    ctx.window.request_redraw();
                });
            }
            // Resize the render surface
            WindowEvent::Resized(PhysicalSize { width, height }) => {
                ctx.write(|ctx| {
                    // Resize the GPU surface
                    ctx.graphics.resize(width, height);

                    // On MacOS the window needs to be redrawn manually after resizing
                    ctx.window.request_redraw();
                });
            }
            // Close the window if requested
            WindowEvent::CloseRequested => {
                // Destroy the context
                self.ctx = None;

                // Tell winit that we want to exit
                event_loop.exit();
            }
            // Handle other window events with the input manager
            other => ctx.write(|ctx| ctx.input.handle_event(other, &ctx.graphics)),
        }
    }
}

/// Run the game.
fn run(game: impl Game, config: Config) {
    // Setup the timestep variables for calculating the update loop
    let accumulator = 0.0;
    let last_time = Instant::now();

    // Context must be initialized later when creating the window
    let context = None;

    // Create a polling event loop, which redraws the window whenever possible
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    // Run the game
    let _ = event_loop.run_app(&mut State {
        ctx: context,
        game,
        config,
        last_time,
        accumulator,
    });
}
