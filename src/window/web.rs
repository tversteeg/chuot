use std::sync::Arc;

use game_loop::{GameLoop, Time, TimeTrait};
use miette::{Context, IntoDiagnostic, Result};
use pixels::{
    wgpu::{BlendState, Color},
    Pixels, PixelsBuilder, SurfaceTexture,
};
use vek::Extent2;
use wasm_bindgen::JsCast;
use web_sys::HtmlCanvasElement;
use winit::{
    dpi::LogicalSize,
    event::{Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::EventLoop,
    platform::web::WindowBuilderExtWebSys,
    window::{Window, WindowBuilder},
};

use super::WindowConfig;

/// Desktop implementation of opening a window.
pub(crate) async fn window<G, U, R, H>(
    window_builder: WindowBuilder,
    game_state: G,
    WindowConfig {
        buffer_size,
        scaling,
        title,
        updates_per_second,
    }: WindowConfig,
    mut update: U,
    mut render: R,
    mut event: H,
) -> Result<()>
where
    G: 'static,
    U: FnMut(&mut G, f32) + 'static,
    R: FnMut(&mut G, &mut [u32], f32) + 'static,
    H: FnMut(&mut GameLoop<(G, Pixels), Time, Arc<Window>>, &Event<'_, ()>) + 'static,
{
    // Create a canvas the winit window can be attached to
    let window = web_sys::window().ok_or_else(|| miette::miette!("Error finding web window"))?;
    let document = window
        .document()
        .ok_or_else(|| miette::miette!("Error finding web document"))?;
    let body = document
        .body()
        .ok_or_else(|| miette::miette!("Error finding web body"))?;
    body.style().set_css_text("text-align: center");
    let canvas = document
        .create_element("canvas")
        .map_err(|err| miette::miette!("Error creating canvas: {err:?}"))?
        .dyn_into::<HtmlCanvasElement>()
        .map_err(|err| miette::miette!("Error casting canvas: {err:?}"))?;
    canvas.set_id("canvas");
    body.append_child(&canvas)
        .map_err(|err| miette::miette!("Error appending canvas to body: {err:?}"))?;

    // Create a header with the title
    let header = document
        .create_element("h2")
        .map_err(|err| miette::miette!("Error creating h2 element: {err:?}"))?;
    header.set_text_content(Some(title.as_str()));
    body.append_child(&header)
        .map_err(|err| miette::miette!("Error appending header to body: {err:?}"))?;

    // Create the window
    let event_loop = EventLoop::new();
    let window = window_builder
        .with_canvas(Some(canvas.clone()))
        .build(&event_loop)
        .into_diagnostic()
        .wrap_err("Error setting up window")?;

    // Setup the pixel surface
    let surface_texture = SurfaceTexture::new(
        (buffer_size.w * scaling) as u32,
        (buffer_size.h * scaling) as u32,
        &window,
    );
    let pixels = PixelsBuilder::new(buffer_size.w as u32, buffer_size.h as u32, surface_texture)
        .clear_color(Color::WHITE)
        .blend_state(BlendState::REPLACE)
        .build_async()
        .await
        .into_diagnostic()
        .wrap_err("Error setting up pixels buffer")?;

    // Ensure the pixels are not rendered with wrong filtering and that the size is correct
    canvas.style().set_css_text(&format!(
        "display:block; margin: auto; image-rendering: pixelated; width: {}px; height: {}px",
        buffer_size.w * scaling,
        buffer_size.h * scaling
    ));
    canvas.set_width((buffer_size.w * scaling) as u32);
    canvas.set_height((buffer_size.h * scaling) as u32);

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
        event,
    );
}
