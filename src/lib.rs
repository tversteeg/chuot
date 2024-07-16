#![forbid(unsafe_code)]

//! AGPL licensed and opinionated game engine for 2D pixel-art games.
//!
//! # Features
//!
//! - Pixel-perfect pixel art rendering with built-in rotsprite rotation shader.
//! - Window creation with independent update and render game loop.
//! - Hot-reloadable assets, seeing your assets update live in the game when you save them is a great boost in productivity for quickly iterating on ideas.
//! - Single-binary, all non-texture assets will be embedded directly, and textures will be diced into a single atlas map embedded in the binary when deploying.
//! - Simple bitmap font drawing.
//! - OGG audio playback.
//! - First-class gamepad support.
//!
//! # Goals
//!
//! - Ergonomic API with a focus on quickly creating small games, especially for game jams.
//! - Reasonable performance, drawing thousands of animated sprites at the same time shouldn't be a problem.
//! - Proper web support, it should be very easy to bundle as WASM for the web.
//!
//! # Non-Goals
//!
//! - An ECS (Entity component system), although an ECS architecture is great for cache locality and thus performance, I feel that it's overkill for most small games. Nothing is stopping you to add your own on top of this engine if that's what you want though!
//! - 3D, this engine is only for 2D pixel art.
//! - Vector graphics, similar to the above, this engine is focused specifically on pixel art with lower resolutions.
//! - Reinventing the wheel for everything, when there's a proper crate with good support I prefer to use that instead of creating additional maintainer burden.
//! - Support all possible file formats, this bloats the engine.
//!
//! # Usage
//!
//! Using this crate is quite simple, there is a single trait [`Game`] with two required functions, [`Game::update`] and [`Game::render`], that need to be implemented for a game state object.
//!
//! ```
//! use chuot::{Config, Context, Game};
//!
//! struct MyGame;
//!
//! impl Game for MyGame {
//!     fn update(&mut self, ctx: Context) {
//!         // ..
//!     }
//!
//!     fn render(&mut self, ctx: Context) {
//!         // ..
//!     }
//! }
//!
//! # fn try_main() {
//! // In main
//!
//! let game = MyGame;
//!
//! game.run(chuot::load_assets!(), Config::default());
//! # }
//! ```
//!
//! # Features
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
//! ## `read-texture` (default)
//!
//! Expose read operations on images, if disabled sprites will be uploaded to the GPU and their data will be removed from memory.
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
//! ```
//! use chuot::{
//!   Game, Context, Config,
//!   context::{MouseButton, KeyCode},
//! };
//!
//! /// Object holding all game state.
//! struct MyGame {
//!   /// A simple counter we increment by clicking on the screen.
//!   counter: u32,
//! }
//!
//! impl Game for MyGame {
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
//! # fn try_main()  {
//! // In main
//!
//! // Initialize the game state
//! let game = MyGame { counter: 0 };
//!
//! // Run the game until exit is requested
//! game.run(chuot::load_assets!(), Config::default().with_title("My Game"));
//! # }
//! ```
//!
//! [^left-mouse]: [`Context::mouse_pressed`]
//! [^text]: [`Context::text`]
//! [^escape-key]: [`Context::key_pressed`]

pub mod assets;
mod camera;
pub mod config;
pub mod context;
mod graphics;
mod input;
mod random;

pub use assets::source::AssetSource;
/// Define the directory of the assets.
///
/// *MUST* be passed as first argument to [`Game::run`].
///
/// The assets will be embedded in the binary when using the `embed-assets` feature flag.
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
pub use config::Config;
pub use context::Context;
pub use random::random;
pub use rgb::RGBA8;
use web_time::Instant;
use winit::{
    application::ApplicationHandler,
    dpi::{LogicalSize, PhysicalSize},
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{WindowAttributes, WindowId},
};

/// How fast old FPS values decay in the smoothed average.
const FPS_SMOOTHED_AVERAGE_ALPHA: f32 = 0.8;

/// Maximum of the amount of `update` calls for a single `render` call.
const MAX_UPDATE_CALLS_PER_RENDER: f32 = 20.0;

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
    /// use chuot::{context::KeyCode, Context, Game};
    ///
    /// struct MyGame;
    ///
    /// impl Game for MyGame {
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
    /// use chuot::{Context, Game};
    ///
    /// struct MyGame;
    ///
    /// impl Game for MyGame {
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

    /// Optionally implement this method to run this function at startup.
    ///
    /// Will run after the window is set up and the context is created.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Game context, used to obtain information and queue draw calls.
    ///
    /// # Example
    ///
    /// ```
    /// use chuot::{Context, Game};
    ///
    /// struct MyGame;
    ///
    /// impl Game for MyGame {
    ///     fn init(&mut self, ctx: Context) {
    ///         // Do something you only want to do once, such as setting the camera offset:
    ///         ctx.main_camera().set_top_left();
    ///     }
    ///
    ///     fn render(&mut self, ctx: Context) {
    ///         // ..
    ///     }
    ///
    ///     fn update(&mut self, ctx: Context) {
    ///         // ..
    ///     }
    /// }
    /// ```
    #[inline(always)]
    #[allow(unused_variables)]
    fn init(&mut self, ctx: Context) {}

    /// Run the game, spawning the window.
    ///
    /// <div class="warning">
    ///
    /// Don't implement/override this method.
    ///
    /// </div>
    ///
    /// # Arguments
    ///
    /// * `asset_source` - Source of the assets, should probably be `chuot::load_assets!()`, unless you don't need any assets.
    /// * `config` - Configuration for the window, can be used to set the buffer size, the window title and other things.
    ///
    /// # Errors
    ///
    /// - When a window could not be opened (desktop only).
    /// - If no GPU could be found or accessed.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use chuot::{context::KeyCode, Config, Context, Game};
    ///
    /// struct MyGame;
    ///
    /// impl Game for MyGame {
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
    /// // In main
    /// let game = MyGame;
    ///
    /// game.run(chuot::load_assets!(), Config::default());
    /// ```
    #[inline(always)]
    fn run(self, asset_source: AssetSource, config: Config) {
        // Show panics in the browser console log
        #[cfg(target_arch = "wasm32")]
        console_error_panic_hook::set_once();

        // Setup the timestep variables for calculating the update loop
        let accumulator = 0.0;
        let last_time = Instant::now();

        // Context must be initialized later when creating the window
        let ctx = None;

        // Create a polling event loop, which redraws the window whenever possible
        let event_loop = EventLoop::with_user_event().build().unwrap();
        event_loop.set_control_flow(ControlFlow::Poll);

        #[cfg(target_arch = "wasm32")]
        {
            use winit::platform::web::{EventLoopExtWebSys, PollStrategy, WaitUntilStrategy};

            // Ensure the game on the web runs as smooth as possible
            event_loop.set_poll_strategy(PollStrategy::IdleCallback);
            event_loop.set_wait_until_strategy(WaitUntilStrategy::Worker);
        }

        // Put the asset source on the heap
        let asset_source = Some(Box::new(asset_source));

        // Get the event loop proxy so we can instantiate on the web
        #[cfg(target_arch = "wasm32")]
        let event_loop_proxy = Some(event_loop.create_proxy());

        // Move the game struct to the state
        let game = self;

        // Run the game
        let _ = event_loop.run_app(&mut State {
            ctx,
            asset_source,
            game,
            config,
            last_time,
            accumulator,
            #[cfg(target_arch = "wasm32")]
            event_loop_proxy,
        });
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
    /// Source of all assets.
    ///
    /// Will be taken from the option once.
    asset_source: Option<Box<AssetSource>>,
    /// User supplied game.
    game: G,
    /// User supplied configuration.
    config: Config,
    /// Time for calculating the update rate.
    last_time: Instant,
    /// Timestep accumulator for the update rate.
    accumulator: f32,
    /// Proxy required to send the context on the web platform.
    #[cfg(target_arch = "wasm32")]
    event_loop_proxy: Option<winit::event_loop::EventLoopProxy<Context>>,
}

impl<G: Game> ApplicationHandler<Context> for State<G> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Setup the window
        if self.ctx.is_none() {
            // Define the properties of the window
            #[allow(unused_mut)]
            let mut window_attributes = WindowAttributes::default()
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
                ));

            #[cfg(target_arch = "wasm32")]
            {
                use web_sys::{wasm_bindgen::JsCast, HtmlCanvasElement};
                use winit::platform::web::WindowAttributesExtWebSys;

                // Create or find a canvas the winit window can be attached to
                let window = web_sys::window().unwrap();
                let document = window.document().unwrap();
                let canvas = document
                    .get_element_by_id("chuot")
                    .map(|canvas| canvas.dyn_into::<HtmlCanvasElement>().unwrap());

                // If the canvas is not found a new one will be created
                window_attributes = window_attributes
                    .with_canvas(canvas)
                    // Add the canvas to the web page
                    .with_append(true)
                    // Handle all input events
                    .with_prevent_default(true);
            }

            // Spawn a new window using the event loop
            let window = event_loop
                .create_window(window_attributes)
                .expect("Error creating window");

            // Adjust the canvas for proper integer scale rendering
            #[cfg(target_arch = "wasm32")]
            {
                use winit::platform::web::WindowExtWebSys;

                // Ensure the pixels are not rendered with wrong filtering and that the size is correct
                window
                    .canvas()
                    .unwrap()
                    .style()
                    .set_css_text(
                        &format!(
                            "image-rendering: pixelated; outline: none; border: none; width: {}px; height: {}px",
                            self.config.buffer_width * self.config.scaling,
                            self.config.buffer_height * self.config.scaling,
                        )
                    );
            }

            // Setup the context
            #[cfg(not(target_arch = "wasm32"))]
            {
                // Because pollster returns the value we can set it immediately
                let ctx = pollster::block_on(async {
                    Context::new(
                        self.config.clone(),
                        self.asset_source.take().unwrap(),
                        window,
                    )
                    .await
                });

                // Set the context
                self.ctx = Some(ctx.clone());

                // Call user passed init function
                self.game.init(ctx);
            }
            #[cfg(target_arch = "wasm32")]
            {
                // We only need the proxy once to send the context
                let event_loop_proxy = self.event_loop_proxy.take().unwrap();
                let asset_source = self.asset_source.take().unwrap();
                let config = self.config.clone();

                wasm_bindgen_futures::spawn_local(async move {
                    // Because WASM futures can't block we need to send it with a user event
                    let ctx = Context::new(config, asset_source, window).await;

                    let _ = event_loop_proxy.send_event(ctx);
                });
            }
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
                let frame_time = (current_time - self.last_time)
                    .as_secs_f32()
                    // Ensure that the update loop cannot be called too often
                    .min(MAX_UPDATE_CALLS_PER_RENDER * self.config.update_delta_time);
                self.last_time = current_time;

                self.accumulator += frame_time
                    // Ensure the frametime will never surpass this amount
                    .min(self.config.max_frame_time_secs);

                // Call the user update function with the context
                while self.accumulator >= self.config.update_delta_time {
                    // Call the implemented update function on the 'Game' trait
                    self.game.update(ctx.clone());

                    // Mark this tick as executed
                    self.accumulator -= self.config.update_delta_time;

                    ctx.write(|ctx| {
                        // Update the input so pressed and released events can be handled
                        ctx.input.update();

                        // Update cameras
                        ctx.main_camera.update();
                        ctx.ui_camera.update();

                        // Handle hot reloaded assets
                        #[cfg(not(target_arch = "wasm32"))]
                        assets::hot_reload::handle_changed_asset_files(ctx);
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

                // Only call render loop when the window is not minimized
                let not_minimized = !ctx.is_minimized();

                // Call the user render function with the context
                if not_minimized {
                    self.game.render(ctx.clone());
                }

                ctx.write(|ctx| {
                    // Draw the window and GPU graphics
                    if not_minimized {
                        ctx.graphics.render();
                    }

                    if ctx.exit {
                        // Tell winit that we want to exit
                        event_loop.exit();
                    }
                });
            }
            // Resize the render surface
            WindowEvent::Resized(PhysicalSize { width, height }) => {
                ctx.write(|ctx| {
                    // Resize the GPU surface
                    ctx.graphics.resize(width, height);

                    // On MacOS the window needs to be redrawn manually after resizing
                    #[cfg(target_os = "macos")]
                    ctx.window.request_redraw();
                });
            }
            // Close the window if requested
            WindowEvent::CloseRequested => {
                // Tell winit that we want to exit
                event_loop.exit();
            }
            // Handle other window events with the input manager
            WindowEvent::KeyboardInput { .. }
            | WindowEvent::CursorMoved { .. }
            | WindowEvent::MouseWheel { .. }
            | WindowEvent::MouseInput { .. } => {
                ctx.write(|ctx| ctx.input.handle_event(event, &ctx.graphics));
            }
            // Ignore the rest of the events
            _ => (),
        }
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, ctx: Context) {
        // Call user passed init function
        self.game.init(ctx.clone());

        // We received the context from initializing, set it
        self.ctx = Some(ctx);
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let Some(ctx) = &mut self.ctx else {
            return;
        };

        // Ensure the control flow doesn't change
        event_loop.set_control_flow(ControlFlow::Poll);

        // Application is about to wait, request a redraw
        ctx.write(|ctx| ctx.window.request_redraw());
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        // Destroy all state(s), anarchy for all
        self.ctx = None;
        self.asset_source = None;
        #[cfg(target_arch = "wasm32")]
        {
            self.event_loop_proxy = None;
        }
    }
}
