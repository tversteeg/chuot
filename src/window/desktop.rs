use std::sync::Arc;

use miette::{Context, IntoDiagnostic, Result};
use pixels::{
    wgpu::{BlendState, Color},
    PixelsBuilder, SurfaceTexture,
};

use vek::Vec2;
use winit::{event_loop::EventLoop, window::WindowBuilder};
use winit_input_helper::WinitInputHelper;

use crate::canvas::Canvas;

use super::WindowConfig;

/// Desktop implementation of opening a window.
pub(crate) fn window<G, U, R>(
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
    let event_loop = EventLoop::new()
        .into_diagnostic()
        .wrap_err("Error setting up event loop for window")?;
    let window = window_builder
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
    .build()
    .into_diagnostic()
    .wrap_err("Error setting up pixels buffer")?;

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
