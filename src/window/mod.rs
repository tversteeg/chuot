#[cfg(not(target_arch = "wasm32"))]
mod desktop;
#[cfg(target_arch = "wasm32")]
mod web;

use std::sync::Arc;

use game_loop::{GameLoop, Time};
use miette::{IntoDiagnostic, Result};
use pixels::Pixels;
use vek::Extent2;
use winit::{
    dpi::LogicalSize,
    event::{Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    window::{Window, WindowBuilder},
};

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
/// * `update` - Function called every update tick, arguments are the state and the time between this and the previous tick.
/// * `render` - Function called every render tick, arguments are the state and the time between this and the previous tick.
pub fn window<G, U, R>(
    game_state: G,
    window_config: WindowConfig,
    update: U,
    render: R,
) -> Result<()>
where
    G: 'static,
    U: FnMut(&mut G, f32) + 'static,
    R: FnMut(&mut G, &mut [u32], f32) + 'static,
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
        desktop::window(
            window_builder,
            game_state,
            window_config,
            update,
            render,
            winit_handler,
        )
    }
    #[cfg(target_arch = "wasm32")]
    {
        // Show panics in the browser console log
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));

        // Web window function is async, so we need to spawn it into a local async runtime
        wasm_bindgen_futures::spawn_local(async {
            web::window(
                window_builder,
                game_state,
                window_config,
                update,
                render,
                winit_handler,
            )
            .await
            .expect("Error opening WASM window")
        });

        Ok(())
    }
}

/// Handle winit events.
fn winit_handler<G>(game_loop_state: &mut GameLoop<(G, Pixels), Time, Arc<Window>>, ev: &Event<()>)
where
    G: 'static,
{
    match ev {
        // Handle close event
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } => game_loop_state.exit(),

        // Resize the window
        Event::WindowEvent {
            event: WindowEvent::Resized(new_size),
            ..
        } => {
            game_loop_state
                .game
                .1
                .resize_surface(new_size.width, new_size.height)
                .into_diagnostic()
                .unwrap();
        }

        // Handle key presses
        Event::WindowEvent {
            event:
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        },
                    ..
                },
            ..
        } => game_loop_state.exit(),

        _ => (),
    }
}
