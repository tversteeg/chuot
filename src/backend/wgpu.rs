//! winit + wgpu backend.

use crate::{config::GameConfig, context::Context, PixelGame};

use super::Backend;

/// Backend implementation that uses winit for window handling and wgpu for drawing stuff on the screen.
pub(crate) struct WgpuWinitBackend {}

impl Backend for WgpuWinitBackend {
    #[inline(always)]
    fn new(config: &GameConfig) -> Self {
        todo!()
    }

    #[inline(always)]
    fn run(game: impl PixelGame<Self>, config: GameConfig) {}
}
