use std::sync::Arc;

use game_loop::{GameLoop, Time};
use miette::{Context, IntoDiagnostic, Result};
use pixels::{
    wgpu::{BlendState, Color},
    Pixels, PixelsBuilder, SurfaceTexture,
};

use vek::Vec2;
use winit::{
    event::Event,
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};
use winit_input_helper::WinitInputHelper;

use super::WindowConfig;

/// Desktop implementation of opening a window.
pub(crate) fn window<G, U, R, H>(
    window_builder: WindowBuilder,
    game_state: G,
    WindowConfig {
        buffer_size,
        scaling,
        title: _,
        updates_per_second,
    }: WindowConfig,
    mut update: U,
    mut render: R,
    event: H,
) -> Result<()>
where
    G: 'static,
    U: FnMut(&mut G, &WinitInputHelper, Option<Vec2<usize>>, f32) -> bool + 'static,
    R: FnMut(&mut G, &mut [u32], f32) + 'static,
    H: FnMut(&mut GameLoop<(G, Pixels, WinitInputHelper), Time, Arc<Window>>, &Event<'_, ()>)
        + 'static,
{
    let event_loop = EventLoop::new();
    let window = window_builder
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
        .build()
        .into_diagnostic()
        .wrap_err("Error setting up pixels buffer")?;

    // Open the window and run the event loop
    let mut buffer = vec![0u32; buffer_size.w * buffer_size.h];

    // Handle input
    let input = WinitInputHelper::new();

    game_loop::game_loop(
        event_loop,
        Arc::new(window),
        (game_state, pixels, input),
        updates_per_second,
        0.1,
        move |g| {
            // Calculate mouse in pixels
            let mouse = g.game.2.mouse().and_then(|mouse| {
                g.game
                    .1
                    .window_pos_to_pixel(mouse)
                    .map(|(x, y)| Vec2::new(x, y))
                    .ok()
            });

            // Call update and exit when it returns true
            if update(
                &mut g.game.0,
                &g.game.2,
                mouse,
                (updates_per_second as f32).recip(),
            ) {
                g.exit();
            }
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
