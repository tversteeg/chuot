//! Render context passed to each [`crate::PixelGame::render`] call.

use std::collections::HashMap;

use assets_manager::SharedString;
use vek::Vec2;

use crate::sprite::Sprite;

/// Shared render context in [`crate::PixelGame::render`] calls.
///
/// Exposes functionalities for stateful rendering assets.
#[derive(Default)]
pub struct RenderContext {
    /// All sprites instances to render.
    pub(crate) sprites: HashMap<SharedString, Sprite>,
}

impl RenderContext {
    /// Draw a sprite on the screen.
    ///
    /// This will load the sprite asset from disk and upload it to the GPU the first time this sprite is referenced.
    pub fn draw_sprite(&mut self, path: &str, position: Vec2<f64>) {
        // Load the asset if not accessed yet
        if !self.sprites.contains_key(path) {
            let sprite = crate::asset_owned(path);
            self.sprites.insert(path.into(), sprite);
        }

        // Add an instance of the sprite
        self.sprites
            .get_mut(path)
            .expect("Error accessing sprite in context")
            .render(position);
    }

    /// Clear everything after rendering a frame for the next frame.
    pub fn clear(&mut self) {
        self.sprites
            .iter_mut()
            .for_each(|(_, sprite)| sprite.clear_instances());
    }
}
