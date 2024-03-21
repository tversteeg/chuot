//! Spawn a winit window and run the game loop.

#[cfg(not(target_arch = "wasm32"))]
mod desktop;
#[cfg(feature = "in-game-profiler")]
mod in_game_profiler;
#[cfg(target_arch = "wasm32")]
mod web;

use std::sync::Arc;

use glamour::{Size2, Vector2};
use miette::{IntoDiagnostic, Result, WrapErr};
use winit::{
    dpi::LogicalSize,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};
use winit_input_helper::WinitInputHelper;

use crate::{graphics::state::MainRenderState, Context};

/// Tick function signature.
pub(crate) trait TickFn<G>: FnMut(&mut G, Context) {}

impl<G, T: FnMut(&mut G, Context)> TickFn<G> for T {}

/// Window configuration.
#[derive(Debug, Clone)]
pub struct WindowConfig {
    /// Amount of pixels for the canvas.
    ///
    /// Defaults to `(320, 280)`.
    pub buffer_size: Size2,
    /// Factor applied to the buffer size for the requested window size.
    ///
    /// Defaults to `1`.
    pub scaling: f32,
    /// Name in the title bar.
    ///
    /// On WASM this will display as a header underneath the rendered content.
    ///
    /// Defaults to `"Pixel Game"`.
    pub title: String,
    /// Updates per second for the update loop.
    ///
    /// Defaults to `60`.
    pub updates_per_second: u32,
    /// Color of the viewport.
    ///
    /// The viewport is the area outside of the buffer when inside a bigger window.
    ///
    /// Defaults to `0xFF76428A` (purple).
    pub viewport_color: u32,
    /// Color of the background of the buffer.
    ///
    /// Defaults to `0xFF9BADB7` (gray).
    pub background_color: u32,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            buffer_size: Size2::new(320.0, 280.0),
            scaling: 1.0,
            title: "Pixel Game".to_string(),
            updates_per_second: 60,
            viewport_color: 0xFF76428A,
            background_color: 0xFF9BADB7,
        }
    }
}

/// Manually create a new window with an event loop and run the game.
///
/// For a more integrated and easier use it's recommended to use [`crate::PixelGame`].
///
/// If the `audio` feature is enabled this will also start a new audio backend.
///
/// # Arguments
///
/// * `game_state` - Global state passed around in the render and update functions.
/// * `window_config` - Configuration options for the window.
/// * `update` - Function called every update tick, arguments are the state, window input event that can be used to handle input events, mouse position in pixels and the time between this and the previous tick. When `true` is returned the window will be closed.
/// * `render` - Function called every render tick, arguments are the state and the time between this and the previous tick.
///
/// # Errors
///
/// - When the audio manager could not find a device to play audio on.
pub(crate) fn window<G, T>(game_state: G, window_config: WindowConfig, tick: T) -> Result<()>
where
    G: 'static,
    T: TickFn<G> + 'static,
{
    // Build the window builder with the event loop the user supplied
    let window_builder = WindowBuilder::new()
        .with_title(window_config.title.clone())
        // Apply scaling for the requested size
        .with_inner_size(LogicalSize::new(
            window_config.buffer_size.width * window_config.scaling,
            window_config.buffer_size.height * window_config.scaling,
        ))
        // Don't allow the window to be smaller than the pixel size
        .with_min_inner_size(LogicalSize::new(
            window_config.buffer_size.width,
            window_config.buffer_size.height,
        ));

    #[cfg(not(target_arch = "wasm32"))]
    {
        // Enable environment logger for winit
        env_logger::init();

        pollster::block_on(async {
            desktop::window(window_builder, game_state, window_config, tick).await
        })
    }
    #[cfg(target_arch = "wasm32")]
    {
        // Show logs
        console_log::init_with_level(log::Level::Debug).expect("Error setting up logger");

        // Show panics in the browser console log
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));

        // Web window function is async, so we need to spawn it into a local async runtime
        wasm_bindgen_futures::spawn_local(async {
            web::window(window_builder, game_state, window_config, tick)
                .await
                .expect("Error opening WASM window")
        });

        Ok(())
    }
}

/// Open a winit window with an event loop.
async fn winit_start<G, T>(
    event_loop: EventLoop<()>,
    window: Window,
    mut game_state: G,
    mut tick: T,
    WindowConfig {
        buffer_size,
        updates_per_second,
        background_color,
        viewport_color,
        ..
    }: WindowConfig,
) -> Result<()>
where
    G: 'static,
    T: TickFn<G> + 'static,
{
    // Wrap the window in an atomic reference counter so it can be shared in multiple places
    let window = Arc::new(window);

    // Setup the winit input helper state
    let mut input = WinitInputHelper::new();

    // Create a surface on the window and setup the render state to it
    let mut render_state = MainRenderState::new(
        buffer_size,
        window.clone(),
        background_color,
        viewport_color,
    )
    .await
    .wrap_err("Error setting up the rendering pipeline")?;

    // Setup the context passed to the tick function implemented by the user
    let mut ctx = Context::new();

    // Setup the in-game profiler
    #[cfg(feature = "in-game-profiler")]
    let mut in_game_profiler =
        in_game_profiler::InGameProfiler::new(render_state.device(), window.clone());

    log::debug!("Opening window with game loop");

    // Set the event loop to polling so we don't have to wait for new events to draw new frames
    event_loop.set_control_flow(ControlFlow::Poll);

    // Start the window and game loop
    event_loop
        .run(move |event, elwt| {
            // Update egui inside in-game-profiler
            #[cfg(feature = "in-game-profiler")]
            if let winit::event::Event::WindowEvent { event, .. } = &event {
                in_game_profiler.handle_window_event(&window, event);
            };

            // Pass every event to the input helper, when it returns `true` it's time to run the logic
            if input.update(&event) {
                // Exit when the window is destroyed or closed
                if input.close_requested() || input.destroyed() || ctx.read(|ctx| ctx.exit) {
                    elwt.exit();
                    return;
                }

                // Resize render surface if window is resized
                if let Some(new_size) = input.window_resized() {
                    // Resize GPU surface

                    render_state.resize(Size2::new(new_size.width, new_size.height));

                    // On MacOS the window needs to be redrawn manually after resizing
                    window.request_redraw();
                }

                // Set the updated state for the context
                ctx.write(|ctx| {
                    // Set the mouse position
                    ctx.mouse = input
                        .cursor()
                        .and_then(|(x, y)| render_state.map_coordinate(Vector2::new(x, y)));

                    // Embed the input
                    // TODO: remove clone
                    ctx.input = input.clone();
                });

                // Call the tick function with the context
                {
                    profiling::scope!("Tick");

                    tick(&mut game_state, ctx.clone());
                }

                {
                    profiling::scope!("Render");

                    // Render everything
                    #[cfg(feature = "in-game-profiler")]
                    {
                        let screen_size = render_state.screen_size();

                        render_state.render(&mut ctx, |device, queue, encoder, view| {
                            in_game_profiler.render(
                                device,
                                queue,
                                encoder,
                                view,
                                screen_size,
                                &window,
                            )
                        });
                    }
                    #[cfg(not(feature = "in-game-profiler"))]
                    render_state.render(&mut ctx, |_device, _queue, _encoder, _view| ());
                }

                // Tell the profiler we've executed a tick
                profiling::finish_frame!();
            }
        })
        .into_diagnostic()
        .wrap_err("Error running game loop")?;

    Ok(())
}
