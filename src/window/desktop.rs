use miette::{Context, IntoDiagnostic, Result};

use vek::Vec2;
use winit::{event_loop::EventLoop, window::WindowBuilder};
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
    let event_loop = EventLoop::new()
        .into_diagnostic()
        .wrap_err("Error setting up event loop for window")?;
    let window = window_builder
        .build(&event_loop)
        .into_diagnostic()
        .wrap_err("Error setting up window")?;

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
