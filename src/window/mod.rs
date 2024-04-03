//! Spawn a winit window and run the game loop.

#[cfg(not(target_arch = "wasm32"))]
mod desktop;
#[cfg(feature = "in-game-profiler")]
pub(crate) mod in_game_profiler;
#[cfg(target_arch = "wasm32")]
mod web;

// Allow passing the profiler without having to change function signatures
#[cfg(feature = "in-game-profiler")]
pub(crate) use in_game_profiler::InGameProfiler;
#[cfg(not(feature = "in-game-profiler"))]
pub(crate) type InGameProfiler = ();

use std::sync::Arc;

use glamour::{Size2, Vector2};
use miette::{IntoDiagnostic, Result, WrapErr};
use winit::{
    dpi::LogicalSize,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};
use winit_input_helper::WinitInputHelper;

use crate::{graphics::state::MainRenderState, Context, GameConfig};

/// Tick function signature.
pub(crate) trait TickFn<G>: FnMut(&mut G, Context) {}

impl<G, T: FnMut(&mut G, Context)> TickFn<G> for T {}

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
pub(crate) fn window<G, T>(game_state: G, window_config: GameConfig, tick: T) -> Result<()>
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
        console_log::init_with_level(log::Level::Warn).expect("Error setting up logger");

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
    game_config: GameConfig,
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
    let mut render_state = MainRenderState::new(&game_config, window.clone())
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

                // Resize render surface if window is resized on the desktop, on the web the size is always the same
                #[cfg(not(target_arch = "wasm32"))]
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

                    // Reset the sprite instances
                    ctx.instances.clear();
                });

                // Call the tick function with the context
                {
                    profiling::scope!("Tick");

                    // Profile the allocations
                    #[cfg(feature = "in-game-profiler")]
                    let profile_region = InGameProfiler::start_profile_heap();

                    tick(&mut game_state, ctx.clone());

                    #[cfg(feature = "in-game-profiler")]
                    in_game_profiler.finish_profile_heap("Tick", profile_region);
                }

                {
                    profiling::scope!("Render");

                    // Render everything
                    #[cfg(feature = "in-game-profiler")]
                    render_state.render(&mut ctx, &mut in_game_profiler, &window);
                    #[cfg(not(feature = "in-game-profiler"))]
                    render_state.render(&mut ctx, &mut (), &window);
                }

                // Tell the profiler we've executed a tick
                profiling::finish_frame!();
            }
        })
        .into_diagnostic()
        .wrap_err("Error running game loop")?;

    Ok(())
}
