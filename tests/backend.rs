//! Create a back-end for running tests which doesn't actually interface with any OS functionality.

use chuot::{backend::Backend, config::GameConfig, PixelGame};

/// Test back-end.
pub struct TestBackend {}

impl Backend for TestBackend {
    fn run(game: impl PixelGame<Self>, config: GameConfig) {
        todo!()
    }
}
