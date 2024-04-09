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

use glamour::Vector2;
use miette::{IntoDiagnostic, Result, WrapErr};
use web_time::Instant;
use winit::{
    dpi::{LogicalSize, PhysicalSize},
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};

use crate::{graphics::state::MainRenderState, Context, GameConfig};

/// How fast old FPS values decay in the smoothed average.
const FPS_SMOOTHED_AVERAGE_ALPHA: f32 = 0.8;

/// Update and render tick function signature.
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
pub(crate) fn window<G, U, R>(
    game_state: G,
    window_config: GameConfig,
    update: U,
    render: R,
) -> Result<()>
where
    G: 'static,
    U: TickFn<G> + 'static,
    R: TickFn<G> + 'static,
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
            desktop::window(window_builder, game_state, window_config, update, render).await
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
            web::window(window_builder, game_state, window_config, update, render)
                .await
                .expect("Error opening WASM window")
        });

        Ok(())
    }
}

/// Open a winit window with an event loop.
async fn winit_start<G, U, R>(
    event_loop: EventLoop<()>,
    window: Window,
    mut game_state: G,
    mut update: U,
    mut render: R,
    game_config: GameConfig,
) -> Result<()>
where
    G: 'static,
    U: TickFn<G> + 'static,
    R: TickFn<G> + 'static,
{
    // Wrap the window in an atomic reference counter so it can be shared in multiple places
    let window = Arc::new(window);

    // Scale to the scaled size
    let _ = window.request_inner_size(PhysicalSize::new(
        game_config.buffer_size.width * game_config.scaling,
        game_config.buffer_size.height * game_config.scaling,
    ));

    // Create a surface on the window and setup the render state to it
    let mut render_state = MainRenderState::new(&game_config, window.clone())
        .await
        .wrap_err("Error setting up the rendering pipeline")?;

    // Setup the context passed to the tick function implemented by the user
    let mut ctx = Context::new(&game_config);

    // Setup the in-game profiler
    #[cfg(feature = "in-game-profiler")]
    let mut in_game_profiler =
        in_game_profiler::InGameProfiler::new(render_state.device(), window.clone());

    log::debug!("Opening window with game loop");

    // Current time
    let mut last_time = Instant::now();
    // Timestep accumulator
    let mut accumulator = 0.0;

    // Window events to send to the input update
    let mut window_events = Vec::with_capacity(4);

    // Start the window and game loop
    event_loop
        .run(move |event, elwt| {
            match event {
                Event::WindowEvent { window_id, event } if window_id == window.id() => {
                    // Update egui inside in-game-profiler
                    #[cfg(feature = "in-game-profiler")]
                    in_game_profiler.handle_window_event(&window, &event);

                    // Redraw the window when requested
                    match event {
                        // Resize render surface if window is resized on the desktop, on the web the size is always the same
                        #[cfg(not(target_arch = "wasm32"))]
                        WindowEvent::Resized(new_size) => {
                            // Resize GPU surface
                            render_state
                                .resize(glamour::Size2::new(new_size.width, new_size.height));

                            // On MacOS the window needs to be redrawn manually after resizing
                            window.request_redraw();
                        }
                        // Render the frame
                        WindowEvent::RedrawRequested => {
                            // Set the updated state for the context
                            ctx.write(|ctx| {
                                // Set the mouse position
                                ctx.mouse = ctx.input.cursor().and_then(|(x, y)| {
                                    render_state.map_coordinate(Vector2::new(x, y))
                                });
                            });

                            // Update the timestep
                            let current_time = Instant::now();
                            let frame_time = (current_time - last_time).as_secs_f32();
                            last_time = current_time;

                            accumulator += frame_time
                                // Ensure the frametime will never surpass this amount
                                .min(game_config.max_frame_time_secs);

                            // Call the update tick function with the context
                            while accumulator >= game_config.update_delta_time {
                                let should_exit = ctx.write(|ctx| {
                                    // Handle the accumulated window events
                                    ctx.input.step_with_window_events(&window_events);
                                    window_events.clear();

                                    // Exit when the window is destroyed or closed
                                    let should_exit = ctx.input.close_requested()
                                        || ctx.input.destroyed()
                                        || ctx.exit;

                                    if should_exit {
                                        elwt.exit();
                                    }

                                    should_exit
                                });

                                if should_exit {
                                    // Exit was requested
                                    return;
                                }

                                profiling::scope!("Update");

                                // Profile the allocations
                                #[cfg(feature = "in-game-profiler")]
                                let profile_region = InGameProfiler::start_profile_heap();

                                // Call the implemented update function on the 'PixelGame' trait
                                update(&mut game_state, ctx.clone());

                                #[cfg(feature = "in-game-profiler")]
                                in_game_profiler.finish_profile_heap("Update", profile_region);

                                // Mark this tick as executed
                                accumulator -= game_config.update_delta_time;
                            }

                            // Set the blending factor for the render loop
                            ctx.write(|ctx| {
                                // Set the blending factor
                                ctx.blending_factor = accumulator / game_config.update_delta_time;

                                // Set the FPS with a smoothed average function
                                ctx.frames_per_second = FPS_SMOOTHED_AVERAGE_ALPHA
                                    * ctx.frames_per_second
                                    + (1.0 - FPS_SMOOTHED_AVERAGE_ALPHA) * frame_time.recip();

                                // Reset the renderable instances
                                ctx.instances.clear();
                            });

                            // Call the render tick function with the context
                            {
                                profiling::scope!("Render");

                                // Profile the allocations
                                #[cfg(feature = "in-game-profiler")]
                                let profile_region = InGameProfiler::start_profile_heap();

                                // Call the implemented render function on the 'PixelGame' trait
                                render(&mut game_state, ctx.clone());

                                #[cfg(feature = "in-game-profiler")]
                                in_game_profiler.finish_profile_heap("Render", profile_region);
                            }

                            // Render the saved render state
                            {
                                profiling::scope!("Render Internal");

                                // Render everything
                                #[cfg(feature = "in-game-profiler")]
                                render_state.render(&mut ctx, &mut in_game_profiler, &window);
                                #[cfg(not(feature = "in-game-profiler"))]
                                render_state.render(&mut ctx, &mut (), &window);
                            }

                            // Tell the profiler we've executed a tick
                            profiling::finish_frame!();
                        }
                        _ => (),
                    }

                    // Push all events so we can handle them at once in the update tick
                    window_events.push(event);
                }
                Event::AboutToWait => {
                    // Request a redraw so the render loop can be executed
                    window.request_redraw();
                }
                _ => (),
            };
        })
        .into_diagnostic()
        .wrap_err("Error running game loop")?;

    Ok(())
}
