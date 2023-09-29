//! Window & event management module.
//!
//! Requires the `window` feature flag.

use std::sync::Arc;

use game_loop::winit::{dpi::LogicalSize, window::WindowBuilder};
use miette::{IntoDiagnostic, Result};
use pixels::{
    wgpu::{BlendState, Color},
    PixelsBuilder, SurfaceTexture,
};
use vek::Extent2;
use winit::{
    event::{Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::EventLoop,
};

/// Create a new window with an event loop and run the game.
///
/// # Arguments
///
/// * `game_state` - Global state passed around in the render and update functions.
/// * `buffer_size` - Pixel dimensions, automatically upscaled.
/// * `updates_per_second` - Request interval for the update function to be called.
/// * `update` - Function called every update tick, arguments are the state and the time between this and the previous tick.
/// * `render` - Function called every render tick, arguments are the state and the time between this and the previous tick.
pub async fn window<G, U, R>(
    game_state: G,
    buffer_size: Extent2<usize>,
    updates_per_second: u32,
    mut update: U,
    mut render: R,
) -> Result<()>
where
    G: 'static,
    U: FnMut(&mut G, f32) + 'static,
    R: FnMut(&mut G, &mut [u32], f32) + 'static,
{
    #[cfg(target_arch = "wasm32")]
    let canvas = wasm::setup_canvas();

    // Build the window builder with the event loop the user supplied
    let event_loop = EventLoop::new();
    let logical_size = LogicalSize::new(buffer_size.w as f64, buffer_size.h as f64);
    #[allow(unused_mut)]
    let mut window_builder = WindowBuilder::new()
        .with_title("DINOJAM3 - Darwin's Ascent")
        .with_inner_size(logical_size)
        .with_min_inner_size(logical_size);

    // Setup the WASM canvas if running on the browser
    #[cfg(target_arch = "wasm32")]
    {
        use winit::platform::web::WindowBuilderExtWebSys;

        window_builder = window_builder.with_canvas(Some(canvas));
    }

    let window = window_builder.build(&event_loop).into_diagnostic()?;

    let pixels = {
        let surface_texture =
            SurfaceTexture::new(buffer_size.w as u32 * 2, buffer_size.h as u32 * 2, &window);
        PixelsBuilder::new(buffer_size.w as u32, buffer_size.h as u32, surface_texture)
            .clear_color(Color::WHITE)
            .blend_state(BlendState::REPLACE)
            .build_async()
            .await
    }
    .into_diagnostic()?;

    #[cfg(target_arch = "wasm32")]
    wasm::update_canvas(buffer_size);

    // Open the window and run the event loop
    let mut buffer = vec![0u32; buffer_size.w * buffer_size.h];

    game_loop::game_loop(
        event_loop,
        Arc::new(window),
        (game_state, pixels),
        updates_per_second,
        0.1,
        move |g| {
            update(&mut g.game.0, (updates_per_second as f32).recip());
        },
        move |g| {
            let frame_time = g.last_frame_time();
            render(&mut g.game.0, &mut buffer, frame_time as f32);

            // Blit draws the pixels in RGBA format, but the pixels crate expects BGRA, so convert it
            g.game
                .1
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
            if let Err(err) = g.game.1.render() {
                dbg!(err);
                // TODO: properly handle error
                g.exit();
            }
        },
        move |g, ev| {
            match ev {
                // Handle close event
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => g.exit(),

                // Resize the window
                Event::WindowEvent {
                    event: WindowEvent::Resized(new_size),
                    ..
                } => {
                    g.game
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
                } => g.exit(),

                _ => (),
            }
        },
    );
}

#[cfg(target_arch = "wasm32")]
mod wasm {
    use vek::Extent2;
    use wasm_bindgen::JsCast;
    use web_sys::HtmlCanvasElement;

    /// Attach the winit window to a canvas.
    pub fn setup_canvas() -> HtmlCanvasElement {
        log::debug!("Binding window to HTML canvas");

        let window = web_sys::window().unwrap();

        let document = window.document().unwrap();
        let body = document.body().unwrap();
        body.style().set_css_text("text-align: center");

        let canvas = document
            .create_element("canvas")
            .unwrap()
            .dyn_into::<HtmlCanvasElement>()
            .unwrap();

        canvas.set_id("canvas");
        body.append_child(&canvas).unwrap();

        let header = document.create_element("h2").unwrap();
        header.set_text_content(Some("DINOJAM3 - Darwin's Ascent"));
        body.append_child(&header).unwrap();

        canvas
    }

    /// Update the size of the canvas.
    pub fn update_canvas(size: Extent2<usize>) {
        let window = web_sys::window().unwrap();

        let document = window.document().unwrap();

        let canvas = document
            .get_element_by_id("canvas")
            .unwrap()
            .dyn_into::<HtmlCanvasElement>()
            .unwrap();

        canvas.style().set_css_text(&format!(
            "display:block; margin: auto; image-rendering: pixelated; width: {}px; height: {}px",
            size.w * 2,
            size.h * 2
        ));
        canvas.set_width(size.w as u32 * 2);
        canvas.set_height(size.h as u32 * 2);
    }
}
