#[cfg(not(target_arch = "wasm32"))]
mod desktop;
#[cfg(target_arch = "wasm32")]
mod web;

/// Re-export winit types.
pub use winit::{dpi::PhysicalSize, keyboard::KeyCode};
/// Re-export winit_input_helper type.
pub use winit_input_helper::{TextChar, WinitInputHelper as Input};

use std::rc::Rc;

use game_loop::{GameLoop, Time};
use miette::{Context, IntoDiagnostic, Result};
use pixels::Pixels;
use vek::{Extent2, Vec2};
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};
use winit_input_helper::WinitInputHelper;

use crate::canvas::Canvas;

/// Window configuration.
#[derive(Debug, Clone)]
pub struct WindowConfig {
    /// Amount of pixels for the canvas.
    ///
    /// Defaults to `(320, 280)`.
    pub buffer_size: Extent2<usize>,
    /// How many times the buffer should be scaled to fit the window.
    ///
    /// Defaults to `1`.
    pub scaling: usize,
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
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            buffer_size: Extent2::new(320, 280),
            scaling: 1,
            title: "Pixel Game".to_string(),
            updates_per_second: 60,
        }
    }
}

/// Create a new window with an event loop and run the game.
///
/// # Arguments
///
/// * `game_state` - Global state passed around in the render and update functions.
/// * `window_config` - Configuration options for the window.
/// * `update` - Function called every update tick, arguments are the state, window input event that can be used to handle input events, mouse position in pixels and the time between this and the previous tick. When `true` is returned the window will be closed.
/// * `render` - Function called every render tick, arguments are the state and the time between this and the previous tick.
pub fn window<G, U, R>(
    game_state: G,
    window_config: WindowConfig,
    update: U,
    render: R,
) -> Result<()>
where
    G: 'static,
    U: FnMut(&mut G, &WinitInputHelper, Option<Vec2<usize>>, f32) -> bool + 'static,
    R: FnMut(&mut G, &mut Canvas, f32) + 'static,
{
    // Build the window builder with the event loop the user supplied
    let logical_size = LogicalSize::new(
        window_config.buffer_size.w as f64,
        window_config.buffer_size.h as f64,
    );
    let window_builder = WindowBuilder::new()
        .with_title(window_config.title.clone())
        .with_inner_size(logical_size)
        .with_min_inner_size(logical_size);

    #[cfg(not(target_arch = "wasm32"))]
    {
        desktop::window(window_builder, game_state, window_config, update, render)
    }
    #[cfg(target_arch = "wasm32")]
    {
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
fn winit_start<G, U, R>(
    event_loop: EventLoop<()>,
    window: Rc<Window>,
    pixels: Pixels,
    game_state: G,
    mut update: U,
    mut render: R,
    WindowConfig {
        buffer_size,
        updates_per_second,
        ..
    }: WindowConfig,
) -> Result<()>
where
    U: FnMut(&mut G, &WinitInputHelper, Option<Vec2<usize>>, f32) -> bool + 'static,
    R: FnMut(&mut G, &mut Canvas, f32) + 'static,
{
    // Helper for input handling
    let mut input = WinitInputHelper::new();

    /// Pass multiple fields to the game state.
    struct State<G> {
        /// User passed game state.
        game_state: G,
        /// Pixels to draw.
        buffer: Vec<u32>,
        /// Pixels itself.
        pixels: Pixels,
    }

    // Buffer for pixels
    let buffer = vec![0u32; buffer_size.w * buffer_size.h];

    let state = State {
        game_state,
        buffer,
        pixels,
    };

    // Setup the game loop
    let mut game_loop: GameLoop<_, Time, _> =
        GameLoop::new(state, updates_per_second, 0.1, window.clone());

    event_loop
        .run(move |event, elwt| {
            // Don't wait for winit events to run the next tick
            elwt.set_control_flow(ControlFlow::Poll);

            // Update the input helper
            input.update(&event);

            match event {
                // Handle close event
                Event::WindowEvent { event, window_id } if window_id == window.id() => {
                    match event {
                        // Set the occluded state in the game loop
                        WindowEvent::Occluded(occluded) => game_loop.window_occluded = occluded,
                        // Resize pixels surface if window resized
                        WindowEvent::Resized(new_size) => game_loop
                            .game
                            .pixels
                            .resize_surface(new_size.width, new_size.height)
                            .into_diagnostic()
                            .unwrap(),
                        // Draw the next frame
                        WindowEvent::RedrawRequested => {
                            // Update loop
                            let loop_update = |g: &mut GameLoop<State<G>, _, _>| {
                                // Calculate mouse in pixels
                                let mouse = input.cursor().and_then(|mouse| {
                                    g.game
                                        .pixels
                                        .window_pos_to_pixel(mouse)
                                        .map(|(x, y)| Vec2::new(x, y))
                                        .ok()
                                });

                                // Call update and exit when it returns true
                                if update(
                                    &mut g.game.game_state,
                                    &input,
                                    mouse,
                                    (updates_per_second as f32).recip(),
                                ) {
                                    g.exit();
                                }
                            };

                            // Render loop
                            let loop_render = |g: &mut GameLoop<State<G>, _, _>| {
                                let frame_time = g.last_frame_time();

                                // Wrap the buffer in a canvas with the size
                                let buffer = g.game.buffer.as_mut_slice();
                                let size = buffer_size;
                                let mut canvas = Canvas { size, buffer };

                                render(&mut g.game.game_state, &mut canvas, frame_time as f32);

                                // Blit draws the pixels in RGBA format, but the pixels crate expects BGRA, so convert it
                                g.game
                                    .pixels
                                    .frame_mut()
                                    .chunks_exact_mut(4)
                                    .zip(buffer.iter())
                                    .for_each(|(target, source)| {
                                        let source = source.to_ne_bytes();
                                        target[0] = source[2];
                                        target[1] = source[1];
                                        target[2] = source[0];
                                        target[3] = source[3];
                                    });

                                // Render the pixel buffer
                                if let Err(err) = g.game.pixels.render() {
                                    dbg!(err);
                                    // TODO: properly handle error
                                    g.exit();
                                }
                            };

                            if !game_loop.next_frame(loop_update, loop_render) {
                                elwt.exit();
                            }
                        }
                        _ => (),
                    }
                }

                // Never wait for events
                Event::AboutToWait => {
                    game_loop.window.request_redraw();
                }

                _ => (),
            }
        })
        .into_diagnostic()
        .wrap_err("Error running main loop")?;

    Ok(())
}
