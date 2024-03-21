//! User-facing context used in [`crate::PixelGame::tick`].

use std::sync::{Arc, RwLock};

use glam::Affine2;
use glamour::Vector2;
use hashbrown::HashMap;
use winit::{event::MouseButton, keyboard::KeyCode};
use winit_input_helper::WinitInputHelper;

use crate::{
    graphics::{Render, TextureRef},
    sprite::Sprite,
};

/// Context containing most functionality for interfacing with the game engine.
///
/// Exposed in [`crate::PixelGame::tick`].
///
/// [`Context`] is safe and cheap to clone due to being a `Arc<RwLock<..>>` under the hood.
/// Inspired by [`egui::Context`].
#[derive(Clone)]
pub struct Context {
    /// Implementation of all non-primitive parts.
    inner: Arc<RwLock<ContextInner>>,
}

impl Context {
    /// Draw a sprite on the screen at the set position with a rotation of `0`.
    ///
    /// This will load the sprite asset from disk and upload it to the GPU the first time this sprite is referenced.
    ///
    /// # Arguments
    ///
    /// * `path` - Asset path of the sprite, see [`crate::assets`] for more information about asset loading and storing.
    /// * `position` - Absolute position of the target sprite on the buffer in pixels, will be offset by the sprite offset metadata.
    ///
    /// # Panics
    ///
    /// - When asset failed loading.
    #[inline]
    pub fn draw_sprite(&mut self, path: &str, position: Vector2) {
        // Add an instance of the sprite
        self.write(|ctx| {
            ctx.sprites
                .get_mut(path)
                .expect("Error accessing sprite in context")
                .push_instance(Affine2::from_translation(position.into()));
        });
    }

    /// Draw a sprite on the screen at the set position with the set rotation.
    ///
    /// This will load the sprite asset from disk and upload it to the GPU the first time this sprite is referenced.
    ///
    /// # Arguments
    ///
    /// * `path` - Asset path of the sprite, see [`crate::assets`] for more information about asset loading and storing.
    /// * `position` - Absolute position of the target sprite on the buffer in pixels, will be offset by the sprite offset metadata.
    /// * `rotation` - Rotation of the target sprite in radians, will be applied using the RotSprite algorithm.
    ///
    /// # Panics
    ///
    /// - When asset failed loading.
    #[inline]
    pub fn draw_sprite_rotated(&self, path: &str, position: Vector2, rotation: f32) {
        // Add an instance of the sprite
        self.write(|ctx| {
            ctx.load_sprite_if_not_loaded(path);

            ctx.sprites
                .get_mut(path)
                .expect("Error accessing sprite in context")
                .push_instance(Affine2::from_angle_translation(rotation, position.into()));
        });
    }

    /// Tell the game to exit, this will close the window and return from the [`crate::PixelGame::run`] function.
    ///
    /// The rest of the tick function will still be executed.
    ///
    /// # Example
    ///
    /// ```
    /// use pixel_game_lib::{Context, KeyCode};
    ///
    /// # struct Empty; impl pixel_game_lib::PixelGame for Empty {
    /// // In `PixelGame::tick` trait implementation
    /// // ..
    /// fn tick(&mut self, ctx: Context) {
    ///   // Stop game when 'Escape' is pressed
    ///   if ctx.key_pressed(KeyCode::Escape) {
    ///     ctx.exit();
    ///   }
    /// }
    /// # }

    #[inline]
    pub fn exit(&self) {
        self.write(|ctx| ctx.exit = true)
    }

    /// Get the delta time in seconds, how long since the last tick call.
    ///
    /// Useful for calculating frame-independent physics.
    #[inline]
    pub fn delta_time(&self) -> f32 {
        self.read(|ctx| ctx.input.delta_time().unwrap_or_default().as_secs_f32())
    }

    /// Get the position if the mouse is inside the viewport frame.
    ///
    /// This is `Some(..`) if the mouse is inside the viewport frame, not the entire window.
    /// The value of the coordinates corresponds to the pixel, when the frame is scaled this also encodes the subpixel in the fractional part.
    #[inline]
    pub fn mouse(&self) -> Option<Vector2> {
        self.read(|ctx| ctx.mouse)
    }

    /// Whether the mouse button goes from "not pressed" to "pressed".
    ///
    /// # Arguments
    ///
    /// * `mouse_button` - Mouse button to check the state of.
    #[inline]
    pub fn mouse_pressed(&self, mouse_button: MouseButton) -> bool {
        self.read(|ctx| ctx.input.mouse_pressed(mouse_button))
    }

    /// Whether the mouse button goes from "pressed" to "not pressed".
    ///
    /// # Arguments
    ///
    /// * `mouse_button` - Mouse button to check the state of.
    #[inline]
    pub fn mouse_released(&self, mouse_button: MouseButton) -> bool {
        self.read(|ctx| ctx.input.mouse_released(mouse_button))
    }

    /// Whether the mouse button is in a "pressed" state.
    ///
    /// # Arguments
    ///
    /// * `mouse_button` - Mouse button to check the state of.
    #[inline]
    pub fn mouse_held(&self, mouse_button: MouseButton) -> bool {
        self.read(|ctx| ctx.input.mouse_held(mouse_button))
    }

    /// Whether the key goes from "not pressed" to "pressed".
    ///
    /// Uses physical keys in the US layout, so for example the W key will be in the same physical key on both US and french keyboards.
    ///
    /// # Arguments
    ///
    /// * `keycode` - Key to check the state of.
    #[inline]
    pub fn key_pressed(&self, keycode: KeyCode) -> bool {
        self.read(|ctx| ctx.input.key_pressed(keycode))
    }

    /// Whether the key goes from "pressed" to "not pressed".
    ///
    /// Uses physical keys in the US layout, so for example the W key will be in the same physical key on both US and french keyboards.
    ///
    /// # Arguments
    ///
    /// * `keycode` - Key to check the state of.
    #[inline]
    pub fn key_released(&self, keycode: KeyCode) -> bool {
        self.read(|ctx| ctx.input.key_released(keycode))
    }

    /// Whether the key is in a "pressed" state.
    ///
    /// Uses physical keys in the US layout, so for example the W key will be in the same physical key on both US and french keyboards.
    ///
    /// # Arguments
    ///
    /// * `keycode` - Key to check the state of.
    #[inline]
    pub fn key_held(&self, keycode: KeyCode) -> bool {
        self.read(|ctx| ctx.input.key_held(keycode))
    }

    /// Create a new empty context.
    pub(crate) fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(ContextInner::default())),
        }
    }

    /// Get a read-only reference to the inner struct.
    ///
    /// # Panics
    ///
    /// - When internal [`RwLock`] is poisoned.
    pub(crate) fn read<R>(&self, reader: impl FnOnce(&ContextInner) -> R) -> R {
        profiling::scope!("Context read");

        reader(&self.inner.read().expect("RwLock is poisoned"))
    }

    /// Get a mutable reference to the inner struct.
    ///
    /// # Panics
    ///
    /// - When internal [`RwLock`] is poisoned.
    pub(crate) fn write<R>(&self, writer: impl FnOnce(&mut ContextInner) -> R) -> R {
        profiling::scope!("Context write");

        writer(&mut self.inner.write().expect("RwLock is poisoned"))
    }
}

/// Internal wrapped implementation for [`Context`].
#[derive(Default)]
pub(crate) struct ContextInner {
    /// Whether to exit after the update loop.
    pub(crate) exit: bool,
    /// Mouse position.
    pub(crate) mouse: Option<Vector2>,
    /// Parsed game input.
    ///
    /// Exoses methods for detecting mouse and keyboard events.
    pub(crate) input: WinitInputHelper,
    /// All sprite instances to render.
    pub(crate) sprites: HashMap<TextureRef, Sprite>,
}

impl ContextInner {
    /// Load the sprite asset if it doesn't exist yet.
    fn load_sprite_if_not_loaded(&mut self, path: &str) {
        if !self.sprites.contains_key(path) {
            let sprite = crate::asset_owned(path);
            self.sprites.insert(path.into(), sprite);
        }
    }
}
