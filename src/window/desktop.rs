//! Setting up a window for desktop platforms.

use miette::{Context, IntoDiagnostic, Result};
use winit::{event_loop::EventLoop, window::WindowBuilder};

use crate::assets::AssetSource;

use super::{GameConfig, TickFn};

/// Desktop implementation of opening a window.
#[inline(always)]
pub(crate) async fn window<G, U, R>(
    window_builder: WindowBuilder,
    game_state: G,
    game_config: GameConfig,
    update: U,
    render: R,
    assets: AssetSource,
) -> Result<()>
where
    G: 'static,
    U: TickFn<G> + 'static,
    R: TickFn<G> + 'static,
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
        game_config,
        assets,
    )
    .await
}
