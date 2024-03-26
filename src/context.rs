//! User-facing context used in [`crate::PixelGame::tick`].

use std::{cell::RefCell, rc::Rc};

use assets_manager::SharedString;
use glamour::{Angle, Rect, Vector2};
use hashbrown::HashMap;
use winit::{event::MouseButton, keyboard::KeyCode};
use winit_input_helper::WinitInputHelper;

use crate::{font::Font, graphics::instance::Instances, sprite::Sprite};

/// Context containing most functionality for interfacing with the game engine.
///
/// Exposed in [`crate::PixelGame::tick`].
///
/// [`Context`] is safe and cheap to clone due to being a `Rc<RefCell<..>>` under the hood.
#[derive(Clone)]
pub struct Context {
    /// Implementation of all non-primitive parts.
    inner: Rc<RefCell<ContextInner>>,
}

/// Render methods.
///
/// All methods use a `path` as the first argument, which is then used to retrieve the assets when they haven't been loaded before with [`crate::asset`].
impl Context {
    /// Draw a sprite on the screen at the set position with a rotation of `0`.
    ///
    /// This will load the sprite asset from disk and upload it to the GPU the first time this sprite is referenced.
    ///
    /// # Arguments
    ///
    /// * `path` - Asset path of the sprite, see [`crate::asset`] for more information about asset loading and storing.
    /// * `position` - Absolute position of the target sprite on the buffer in pixels, will be offset by the sprite offset metadata.
    ///
    /// # Panics
    ///
    /// - When asset failed loading.
    #[inline]
    pub fn draw_sprite(&mut self, path: &str, position: impl Into<Vector2>) {
        // Add an instance of the sprite
        self.write(|ctx| {
            ctx.load_sprite_if_not_loaded(path);

            let sprite = ctx
                .sprites
                .get_mut(path)
                .expect("Error accessing sprite in context");

            // Push the instance if the texture is already uploaded
            sprite.draw(position.into(), &mut ctx.instances);
        });
    }

    /// Draw a sprite on the screen at the set position with the set rotation.
    ///
    /// This will load the sprite asset from disk and upload it to the GPU the first time this sprite is referenced.
    ///
    /// # Arguments
    ///
    /// * `path` - Asset path of the sprite, see [`crate::asset`] for more information about asset loading and storing.
    /// * `position` - Absolute position of the target sprite on the buffer in pixels, will be offset by the sprite offset metadata.
    /// * `rotation` - Rotation of the target sprite in radians, will be applied using the RotSprite algorithm.
    ///
    /// # Panics
    ///
    /// - When asset failed loading.
    #[inline]
    pub fn draw_sprite_rotated(
        &self,
        path: &str,
        position: impl Into<Vector2>,
        rotation: impl Into<Angle>,
    ) {
        // Add an instance of the sprite
        self.write(|ctx| {
            ctx.load_sprite_if_not_loaded(path);

            let sprite = ctx
                .sprites
                .get_mut(path)
                .expect("Error accessing sprite in context");

            // Push the instance if the texture is already uploaded
            sprite.draw_rotated(position.into(), rotation.into(), &mut ctx.instances);
        });
    }

    /// Draw some text on the screen at the set position with a rotation of `0`.
    ///
    /// This will load the font asset from disk and upload it to the GPU the first time this font is referenced.
    ///
    /// # Arguments
    ///
    /// * `path` - Asset path of the font, see [`crate::asset`] for more information about asset loading and storing.
    /// * `position` - Absolute position of the target top-left text on the buffer in pixels.
    ///
    /// # Panics
    ///
    /// - When asset failed loading.
    #[inline]
    pub fn draw_text(&self, path: &str, position: impl Into<Vector2>, text: impl AsRef<str>) {
        self.write(|ctx| {
            ctx.load_font_if_not_loaded(path);

            ctx.fonts
                .get_mut(path)
                .expect("Error accessing font in context")
                .draw(position.into(), text.as_ref(), &mut ctx.instances)
        });
    }

    /// Update the pixels of a portion of the sprite.
    ///
    /// # Arguments
    ///
    /// * `path` - Asset path of the sprite, see [`crate::asset`] for more information about asset loading and storing.
    /// * `sub_rectangle` - Sub rectangle within the sprite to update. Width * height must be equal to the amount of pixels, and fall within the sprite's rectangle.
    /// * `pixels` - Array of ARGB pixels, amount must match size of the sub rectangle.
    ///
    /// # Panics
    ///
    /// - When asset failed loading.
    /// - When the sub rectangle does not fit inside the sprite's rectangle.
    /// - When the size of the sub rectangle does not match the amount of pixels
    #[inline]
    pub fn update_sprite_pixels(
        &self,
        path: &str,
        sub_rectangle: impl Into<Rect>,
        pixels: impl Into<Vec<u32>>,
    ) {
        self.write(|ctx| {
            ctx.load_sprite_if_not_loaded(path);

            // Put the update the pixels of the sprite on a queue
            ctx.texture_update_queue
                .push((path.to_string(), sub_rectangle.into(), pixels.into()));
        });
    }
}

/// State methods.
impl Context {
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
    ///
    /// # Example
    ///
    /// ```
    /// use pixel_game_lib::{Context, KeyCode, glamour::Vector2};
    ///
    /// # struct Empty; impl pixel_game_lib::PixelGame for Empty {
    /// // In `PixelGame::tick` trait implementation
    /// // ..
    /// fn tick(&mut self, ctx: Context) {
    ///   // Draw a simple FPS counter on the top-left of the screen
    ///   let fps = ctx.delta_time().recip();
    ///   ctx.draw_text("Beachball", Vector2::ZERO, format!("{fps:.1}"));
    /// }
    /// # }
    #[inline]
    pub fn delta_time(&self) -> f32 {
        self.read(|ctx| ctx.input.delta_time().unwrap_or_default().as_secs_f32())
    }
}

/// Input methods.
impl Context {
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
            inner: Rc::new(RefCell::new(ContextInner::default())),
        }
    }
}

/// Internally used methods.
impl Context {
    /// Get a read-only reference to the inner struct.
    ///
    /// # Panics
    ///
    /// - When internal [`RwLock`] is poisoned.
    #[inline]
    pub(crate) fn read<R>(&self, reader: impl FnOnce(&ContextInner) -> R) -> R {
        reader(&self.inner.borrow())
    }

    /// Get a mutable reference to the inner struct.
    ///
    /// # Panics
    ///
    /// - When internal [`RwLock`] is poisoned.
    #[inline]
    pub(crate) fn write<R>(&self, writer: impl FnOnce(&mut ContextInner) -> R) -> R {
        writer(&mut self.inner.borrow_mut())
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
    /// Instances of all sprites drawn this tick, also includes sprites from the fonts.
    pub(crate) instances: Instances,
    /// All sprite textures to render.
    sprites: HashMap<SharedString, Sprite>,
    /// All font textures to render.
    fonts: HashMap<SharedString, Font>,
    /// Portions of textures that need to be re-written.
    texture_update_queue: Vec<(String, Rect, Vec<u32>)>,
}

impl ContextInner {
    /// Get all sprites from any container with sprites.
    pub(crate) fn sprites_iter_mut(&mut self) -> impl Iterator<Item = &mut Sprite> {
        self.sprites.values_mut().chain(
            self.fonts
                .values_mut()
                .flat_map(|font| font.sprites.iter_mut()),
        )
    }

    /// Take all updates to textures that need to be done.
    pub(crate) fn take_texture_updates(
        &mut self,
    ) -> impl Iterator<Item = (&'_ Sprite, Rect, Vec<u32>)> + '_ {
        self.texture_update_queue
            .drain(..)
            .map(|(path, rect, pixels)| {
                (
                    self.sprites
                        .get(path.as_str())
                        .expect("Sprite update did't yield proper sprite path in 'sprites'"),
                    rect,
                    pixels,
                )
            })
    }

    /// Load the sprite asset if it doesn't exist yet.
    fn load_sprite_if_not_loaded(&mut self, path: &str) {
        if !self.sprites.contains_key(path) {
            profiling::scope!("Load sprite");

            // Load the sprite from disk
            let sprite = crate::asset_owned(path);

            // Keep track of it, to see if it needs to be updated or not
            self.sprites.insert(path.into(), sprite);
        }
    }

    /// Load the font asset if it doesn't exist yet.
    fn load_font_if_not_loaded(&mut self, path: &str) {
        if !self.fonts.contains_key(path) {
            profiling::scope!("Load font");

            // Load the font from disk
            let font = crate::asset_owned(path);

            // Keep track of it, to see if it needs to be updated or not
            self.fonts.insert(path.into(), font);
        }
    }
}
