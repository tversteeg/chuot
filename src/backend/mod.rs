//! Platform and graphics handling.

pub mod wgpu;

use crate::{config::GameConfig, PixelGame};

/// How the game interfaces with the platform it runs on.
pub trait Backend
where
    Self: Sized,
{
    /// Initialize the backend to be put inside a context.
    fn new(config: &GameConfig) -> Self;

    /// Run the game.
    ///
    /// Implementation must handle the event loop for [`chuot::PixelGame::update`] and [`chuot::PixelGame::render`].
    fn run(game: impl PixelGame<Self>, config: GameConfig);
}
