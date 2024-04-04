//! User-facing context used in [`crate::PixelGame::tick`].

pub mod sprite;
pub mod text;

use std::{cell::RefCell, rc::Rc};

use assets_manager::Compound;
use glamour::{Angle, Rect, Vector2};
use hashbrown::HashMap;
use smol_str::SmolStr;
use winit::{event::MouseButton, keyboard::KeyCode};
use winit_input_helper::WinitInputHelper;

use crate::{
    assets::{AssetRef, Assets},
    font::Font,
    graphics::instance::Instances,
    sprite::Sprite,
};

use self::{sprite::DrawSpriteContext, text::DrawTextContext};

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
/// All methods use a `path` as the first argument, which is then used to retrieve the assets when they haven't been loaded before with [`crate::assets`].
impl Context {
    /// Handle sprite assets, mostly used for drawing.
    ///
    /// This will load the sprite asset from disk and upload it to the GPU the first time this sprite is referenced.
    /// Check the [`DrawSpriteContext`] documentation for drawing options available.
    ///
    /// # Arguments
    ///
    /// * `path` - Asset path of the sprite, see [`crate::assets`] for more information about asset loading and storing.
    ///
    /// # Returns
    ///
    /// - A helper struct allowing you to specify the transformations of the sprite.
    ///
    /// # Panics
    ///
    /// - When asset failed loading.
    #[inline(always)]
    pub fn sprite<'path>(&self, path: &'path str) -> DrawSpriteContext<'path, '_> {
        DrawSpriteContext {
            path,
            ctx: self,
            position: Vector2::ZERO,
            rotation: Angle::new(0.0),
        }
    }

    /// Draw some text on the screen at the set position with a rotation of `0`.
    ///
    /// This will load the font asset from disk and upload it to the GPU the first time this font is referenced.
    /// Check the [`DrawTextContext`] documentation for drawing options available.
    ///
    /// # Arguments
    ///
    /// * `path` - Asset path of the font, see [`crate::assets`] for more information about asset loading and storing.
    /// * `text` - ASCII text that will be drawn character by character.
    ///
    /// # Panics
    ///
    /// - When asset failed loading.
    #[inline]
    pub fn text<'path, 'text>(
        &self,
        path: &'path str,
        text: &'text str,
    ) -> DrawTextContext<'path, 'text, '_> {
        DrawTextContext {
            path,
            text,
            ctx: self,
            position: Vector2::ZERO,
        }
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
    /// # Returns
    ///
    /// - Time since last tick in seconds.
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
    ///   ctx.text("Beachball", &format!("{fps:.1}")).draw();
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
    ///
    /// # Returns
    ///
    /// - `None` when the mouse is not on the buffer of pixels.
    /// - `Some(..)` with the coordinates of the pixel if the mouse is on the buffer of pixels.
    #[inline]
    pub fn mouse(&self) -> Option<Vector2> {
        self.read(|ctx| ctx.mouse)
    }

    /// Whether the mouse button goes from "not pressed" to "pressed".
    ///
    /// # Arguments
    ///
    /// * `mouse_button` - Mouse button to check the state of.
    ///
    /// # Returns
    ///
    /// - `true` when the mouse is pressed.
    #[inline]
    pub fn mouse_pressed(&self, mouse_button: MouseButton) -> bool {
        self.read(|ctx| ctx.input.mouse_pressed(mouse_button))
    }

    /// Whether the mouse button goes from "pressed" to "not pressed".
    ///
    /// # Arguments
    ///
    /// * `mouse_button` - Mouse button to check the state of.
    ///
    /// # Returns
    ///
    /// - `true` when the mouse is released.
    #[inline]
    pub fn mouse_released(&self, mouse_button: MouseButton) -> bool {
        self.read(|ctx| ctx.input.mouse_released(mouse_button))
    }

    /// Whether the mouse button is in a "pressed" state.
    ///
    /// # Arguments
    ///
    /// * `mouse_button` - Mouse button to check the state of.
    ///
    /// # Returns
    ///
    /// - `true` when the mouse is being held down.
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
    ///
    /// # Returns
    ///
    /// - `true` when the specified key is pressed.
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
    ///
    /// # Returns
    ///
    /// - `true` when the specified key is released.
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
    ///
    /// # Returns
    ///
    /// - `true` when the specified key is being held.
    #[inline]
    pub fn key_held(&self, keycode: KeyCode) -> bool {
        self.read(|ctx| ctx.input.key_held(keycode))
    }
}

/// Generic asset loading.
impl Context {
    /// Load a reference to any non-renderable asset.
    ///
    /// # Arguments
    ///
    /// * `path` - Directory structure of the asset file in `assets/` where every `/` is a `.`.
    ///
    /// # Panics
    ///
    /// - When asset with path does not exist.
    /// - When asset could not be loaded due to an invalid format.
    #[inline]
    pub fn asset<T>(&self, path: impl AsRef<str>) -> AssetRef<T>
    where
        T: Compound,
    {
        self.read(|ctx| ctx.asset(path.as_ref()))
    }

    /// Load a clone of any non-renderable asset.
    ///
    /// Sets up the asset manager once, which can be accessed with the global function in this module.
    ///
    /// # Arguments
    ///
    /// * `path` - Directory structure of the asset file in `assets/` where every `/` is a `.`.
    ///
    /// # Panics
    ///
    /// - When asset with path does not exist.
    /// - When asset could not be loaded due to an invalid format.
    #[inline]
    pub fn asset_owned<T>(&self, path: impl AsRef<str>) -> T
    where
        T: Compound,
    {
        self.read(|ctx| ctx.asset_owned(path.as_ref()))
    }
}

/// Internally used methods.
impl Context {
    /// Create a new empty context.
    #[inline]
    pub(crate) fn new() -> Self {
        Self {
            inner: Rc::new(RefCell::new(ContextInner::new())),
        }
    }

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
    /// Asset cache.
    pub(crate) assets: Assets,
    /// All sprite textures to render.
    pub(crate) sprites: HashMap<SmolStr, Sprite>,
    /// All font textures to render.
    pub(crate) fonts: HashMap<SmolStr, Font>,
    /// Portions of textures that need to be re-written.
    pub(crate) texture_update_queue: Vec<(SmolStr, Rect, Vec<u32>)>,
}

impl ContextInner {
    /// Initialize the inner context.
    pub(crate) fn new() -> Self {
        let exit = false;
        let mouse = None;
        let input = WinitInputHelper::default();
        let instances = Instances::default();
        let assets = Assets::new();
        let sprites = HashMap::new();
        let fonts = HashMap::new();
        let texture_update_queue = Vec::new();

        Self {
            exit,
            mouse,
            input,
            instances,
            assets,
            sprites,
            fonts,
            texture_update_queue,
        }
    }

    /// Get all sprites from any container with sprites.
    pub(crate) fn sprites_iter_mut(&mut self) -> impl Iterator<Item = &mut Sprite> {
        profiling::scope!("Sprite iterator");
        // PERF: improve performance by removing chain

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
    pub(crate) fn load_sprite_if_not_loaded(&mut self, path: &str) {
        if !self.sprites.contains_key(path) {
            profiling::scope!("Load sprite");

            // Load the sprite from disk
            let sprite = self.asset_owned(path);

            // Keep track of it, to see if it needs to be updated or not
            self.sprites.insert(path.into(), sprite);
        }
    }

    /// Load the font asset if it doesn't exist yet.
    pub(crate) fn load_font_if_not_loaded(&mut self, path: &str) {
        if !self.fonts.contains_key(path) {
            profiling::scope!("Load font");

            // Load the font from disk
            let font = self.asset_owned(path);

            // Keep track of it, to see if it needs to be updated or not
            self.fonts.insert(path.into(), font);
        }
    }

    /// Load an asset.
    pub(crate) fn asset<T>(&self, path: &str) -> AssetRef<T>
    where
        T: Compound,
    {
        profiling::scope!("Load asset");

        self.assets.asset(path)
    }

    /// Load a clone of an asset.
    pub(crate) fn asset_owned<T>(&self, path: &str) -> T
    where
        T: Compound,
    {
        profiling::scope!("Load owned asset");

        self.assets.asset_owned(path)
    }
}
