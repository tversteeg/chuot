//! Setting up a window for desktop platforms.

use miette::{Context, IntoDiagnostic, Result};
use winit::{event_loop::EventLoop, window::WindowBuilder};

use super::{GameConfig, TickFn};

/// Desktop implementation of opening a window.
pub(crate) async fn window<G, T>(
    window_builder: WindowBuilder,
    game_state: G,
    window_config: GameConfig,
    tick: T,
) -> Result<()>
where
    G: 'static,
    T: TickFn<G> + 'static,
{
    let event_loop = EventLoop::new()
        .into_diagnostic()
        .wrap_err("Error setting up event loop for window")?;
    let window = window_builder
        .build(&event_loop)
        .into_diagnostic()
        .wrap_err("Error setting up window")?;

    crate::window::winit_start(event_loop, window, game_state, tick, window_config).await
}
