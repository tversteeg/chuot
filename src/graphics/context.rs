//! Render context passed to each [`crate::PixelGame::render`] call.

use assets_manager::SharedString;
use hashbrown::HashMap;
use vek::{Mat3, Vec2};

use crate::sprite::Sprite;

use super::Render;

/// Shared render context in [`crate::PixelGame::render`] calls.
///
/// Exposes functionalities for stateful rendering assets.
#[derive(Default)]
pub struct RenderContext {
    /// All sprites instances to render.
    pub(crate) sprites: HashMap<SharedString, Sprite>,
}

impl RenderContext {
    /// Draw a sprite on the screen at the set position with a rotation of `0`.
    ///
    /// This will load the sprite asset from disk and upload it to the GPU the first time this sprite is referenced.
    pub fn draw_sprite(&mut self, path: &str, position: Vec2<f64>) {
        self.load_sprite_if_not_loaded(path);

        // Add an instance of the sprite
        self.sprites
            .get_mut(path)
            .expect("Error accessing sprite in context")
            .push_instance(Mat3::identity().translated_2d(position));
    }

    /// Draw a sprite on the screen at the set position with the set rotation.
    ///
    /// This will load the sprite asset from disk and upload it to the GPU the first time this sprite is referenced.
    pub fn draw_sprite_rotated(&mut self, path: &str, position: Vec2<f64>, rotation: f64) {
        self.load_sprite_if_not_loaded(path);

        // Add an instance of the sprite
        self.sprites
            .get_mut(path)
            .expect("Error accessing sprite in context")
            .push_instance(Mat3::identity().rotated_z(rotation).translated_2d(position));
    }

    /// Load the sprite asset if it doesn't exist yet.
    fn load_sprite_if_not_loaded(&mut self, path: &str) {
        if !self.sprites.contains_key(path) {
            let sprite = crate::asset_owned(path);
            self.sprites.insert(path.into(), sprite);
        }
    }
}
