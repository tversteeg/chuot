use std::sync::Arc;

use miette::{Context, IntoDiagnostic, Result};
use pixels::{
    wgpu::{BlendState, Color},
    PixelsBuilder, SurfaceTexture,
};
use vek::Vec2;
use wasm_bindgen::JsCast;
use web_sys::HtmlCanvasElement;
use winit::{event_loop::EventLoop, platform::web::WindowBuilderExtWebSys, window::WindowBuilder};
use winit_input_helper::WinitInputHelper;

use crate::canvas::Canvas;

use super::WindowConfig;

/// Desktop implementation of opening a window.
pub(crate) async fn window<G, U, R>(
    window_builder: WindowBuilder,
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
    header.set_text_content(Some(window_config.title.as_str()));
    body.append_child(&header)
        .map_err(|err| miette::miette!("Error appending header to body: {err:?}"))?;

    // Create the window
    let event_loop = EventLoop::new()
        .into_diagnostic()
        .wrap_err("Error setting up event loop for window")?;
    let window = window_builder
        .with_canvas(Some(canvas.clone()))
        .build(&event_loop)
        .into_diagnostic()
        .wrap_err("Error setting up window")?;

    // Setup the pixel surface
    let surface_texture = SurfaceTexture::new(
        (window_config.buffer_size.w * window_config.scaling) as u32,
        (window_config.buffer_size.h * window_config.scaling) as u32,
        &window,
    );
    let pixels = PixelsBuilder::new(
        window_config.buffer_size.w as u32,
        window_config.buffer_size.h as u32,
        surface_texture,
    )
    .clear_color(Color::WHITE)
    .blend_state(BlendState::REPLACE)
    .build_async()
    .await
    .into_diagnostic()
    .wrap_err("Error setting up pixels buffer")?;

    // Ensure the pixels are not rendered with wrong filtering and that the size is correct
    canvas.style().set_css_text(&format!(
        "display:block; margin: auto; image-rendering: pixelated; width: {}px; height: {}px",
        window_config.buffer_size.w * window_config.scaling,
        window_config.buffer_size.h * window_config.scaling
    ));
    canvas.set_width((window_config.buffer_size.w * window_config.scaling) as u32);
    canvas.set_height((window_config.buffer_size.h * window_config.scaling) as u32);

    crate::window::winit_start(
        event_loop,
        Arc::new(window),
        pixels,
        game_state,
        update,
        render,
        window_config,
    )
}
