//! Setting up a window for WASM platforms.

use miette::{Context, IntoDiagnostic, Result};
use wasm_bindgen::JsCast;
use web_sys::HtmlCanvasElement;
use winit::{event_loop::EventLoop, platform::web::WindowBuilderExtWebSys, window::WindowBuilder};

use super::{GameConfig, TickFn};

/// Desktop implementation of opening a window.
#[inline(always)]
pub(crate) async fn window<G, U, R>(
    window_builder: WindowBuilder,
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

    log::debug!("Creating window attached to canvas");

    // Create the window
    let event_loop = EventLoop::new()
        .into_diagnostic()
        .wrap_err("Error setting up event loop for window")?;
    let window = window_builder
        .with_canvas(Some(canvas.clone()))
        .with_prevent_default(true)
        .build(&event_loop)
        .into_diagnostic()
        .wrap_err("Error setting up window")?;

    // Ensure the pixels are not rendered with wrong filtering and that the size is correct
    canvas.style().set_css_text(&format!(
        "display:block; margin: auto; image-rendering: pixelated; outline: none; border: none;",
    ));
    // canvas.set_width((window_config.buffer_size.width * window_config.scaling).floor() as u32);
    // canvas.set_height((window_config.buffer_size.height * window_config.scaling).floor() as u32);

    crate::window::winit_start(
        event_loop,
        window,
        game_state,
        update,
        render,
        window_config,
    )
    .await
}
