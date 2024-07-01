//! Main interface with the game.

use std::{cell::RefCell, rc::Rc, sync::Arc};

/// Re-exported `gilrs` types for [`Context`] arguments.
pub use gilrs::ev::{Axis, Button};
use gilrs::GamepadId;
use smallvec::SmallVec;
/// Re-exported `winit` types for [`Context`] arguments.
pub use winit::{event::MouseButton, keyboard::KeyCode};
use winit::window::{Fullscreen, Window};

use crate::{
    assets::{
        AssetManager,
        CustomAssetManager,
        Id, loadable::{audio::Audio, font::Font, Loadable, sprite::Sprite}, source::AssetSource,
    },
    config::Config,
    graphics::Graphics,
    input::Input,
};

pub mod sprite;
pub mod text;

/// Context containing most functionality for interfacing with the game engine.
///
/// Exposed in [`crate::Game::update`] and [`crate::Game::render`].
///
/// [`Context`] is safe and cheap to clone due to being a `Rc<RefCell<..>>` under the hood.
///
/// All paths used for asset loading use a `.` symbol instead of a path separator and exclude extensions, here's a list of examples how assets are converted:
///
/// | Example call | Path(s) on disk |
/// | --- | --- |
/// | `ctx.sprite("player")` | `assets/player.png` & `assets/player.toml` (optional) |
/// | `ctx.sprite("gui.widgets.button")` | `assets/gui/widgets/button.png` & `assets/gui/widgets/button.toml` (optional) |
/// | `ctx.audio("song")` | `assets/song.ogg` |
///
/// It's assumed for this table that [`crate::load_assets`] in [`crate::Game`] is called without any arguments or with `chuot::load_assets!("assets/")`.
#[derive(Clone)]
pub struct Context {
    /// Implementation of all non-primitive parts.
    inner: Rc<RefCell<ContextInner>>,
}

/// Window methods.
impl Context {
    /// Horizontal size of the drawable part of the window in pixels.
    ///
    /// This ignores any scaling.
    ///
    /// # Returns
    ///
    /// - width: width of the drawable part of the window.
    #[inline]
    #[must_use]
    pub fn width(&self) -> f32 {
        self.read(|ctx| ctx.graphics.buffer_width)
    }

    /// Vertical size of the drawable part of the window in pixels.
    ///
    /// This ignores any scaling.
    ///
    /// # Returns
    ///
    /// - height: height of the drawable part of the window.
    #[inline]
    #[must_use]
    pub fn height(&self) -> f32 {
        self.read(|ctx| ctx.graphics.buffer_height)
    }

    /// Size of the drawable part of the window in pixels.
    ///
    /// This ignores any scaling.
    ///
    /// # Returns
    ///
    /// - (`width, `height): width and height of the drawable part of the window.
    #[inline]
    #[must_use]
    pub fn size(&self) -> (f32, f32) {
        self.read(|ctx| (ctx.graphics.buffer_width, ctx.graphics.buffer_height))
    }

    /// Show the OS cursor or hide it.
    ///
    /// # Arguments
    ///
    /// * `visible` - `true` to show the OS cursor, `false` to hide it.
    #[inline]
    pub fn set_cursor_visible(&self, visible: bool) {
        self.write(|ctx| ctx.window.set_cursor_visible(visible));
    }

    /// Toggle fullscreen mode.
    ///
    /// Uses a borderless fullscreen mode, not exclusive.
    #[inline]
    pub fn toggle_fullscreen(&self) {
        self.write(|ctx| {
            // Check if we currently are in fullscreen mode
            let is_fullscreen = ctx.window.fullscreen().is_some();

            ctx.window.set_fullscreen(if is_fullscreen {
                // Turn fullscreen off
                None
            } else {
                // Enable fullscreen
                Some(Fullscreen::Borderless(None))
            });
        });
    }
}

/// Game state methods.
impl Context {
    /// Get the delta time in seconds for the update tick.
    ///
    /// This is a constant set by [`Config::with_update_delta_time`].
    ///
    /// # Returns
    ///
    /// - Seconds a single update tick took, this is a constant.
    #[inline]
    #[must_use]
    pub fn delta_time(&self) -> f32 {
        self.read(|ctx| ctx.config.update_delta_time)
    }

    /// Get the amount of frames drawn in a second.
    ///
    /// This counts the times [`crate::Game::render`] is called.
    ///
    /// # Returns
    ///
    /// - Frames per second drawn.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use chuot::{Context, KeyCode, glamour::Vector2};
    ///
    /// # struct Empty; impl Empty {
    /// // In `Game::render` trait implementation
    /// // ..
    /// fn render(&mut self, ctx: Context) {
    ///   // Draw a simple FPS counter on the top-left of the screen
    ///   let fps = ctx.delta_time().recip();
    ///   ctx.text("Beachball", &format!("{:.1}", ctx.frames_per_second())).draw();
    /// }
    /// # }
    #[inline]
    #[must_use]
    pub fn frames_per_second(&self) -> f32 {
        self.read(|ctx| ctx.frames_per_second)
    }
}

/// Mouse input methods.
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
    #[must_use]
    pub fn mouse(&self) -> Option<(f32, f32)> {
        self.read(|ctx| ctx.input.mouse())
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
    #[must_use]
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
    #[must_use]
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
    #[must_use]
    pub fn mouse_held(&self, mouse_button: MouseButton) -> bool {
        self.read(|ctx| ctx.input.mouse_held(mouse_button))
    }

    /// How much the mouse scroll wheel changed in the last update tick.
    ///
    /// # Returns
    ///
    /// - Vector where the X dimension is horizontal scrolling and the Y dimension vertical scrolling.
    #[inline]
    #[must_use]
    pub fn scroll_delta(&self) -> (f32, f32) {
        self.read(|ctx| ctx.input.scroll_diff())
    }
}

/// Keyboard input methods.
impl Context {
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
    #[must_use]
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
    #[must_use]
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
    #[must_use]
    pub fn key_held(&self, keycode: KeyCode) -> bool {
        self.read(|ctx| ctx.input.key_held(keycode))
    }
}

/// Gamepad input methods.
impl Context {
    /// List all connected gamepads.
    ///
    /// Required for querying gamepad state and handling input in the other functions.
    ///
    /// The array is stack allocated up to 4 connected gamepads, after that it will be heap allocated.
    ///
    /// # Returns
    ///
    /// - Stack allocated array of all currently connected gamepad IDs.
    #[inline]
    #[must_use]
    pub fn gamepads_ids(&self) -> SmallVec<[GamepadId; 4]> {
        self.read(|ctx| ctx.input.gamepads_ids())
    }

    /// Whether a gamepad button goes from "not pressed" to "pressed".
    ///
    /// # Arguments
    ///
    /// * `gamepad_id` - ID of the gamepad to check the button of, must be retrieved with [`Context::gamepad_ids`].
    /// * `button` - Which button on the gamepad to check.
    ///
    /// # Returns
    ///
    /// - `None` when gamepad is not found or is disconnected.
    /// - `Some(true)` when gamepad button is being pressed this update tick.
    #[inline]
    #[must_use]
    pub fn gamepad_button_pressed(&self, gamepad_id: GamepadId, button: Button) -> Option<bool> {
        self.read(|ctx| ctx.input.gamepad_button_pressed(gamepad_id, button))
    }

    /// Whether a gamepad button goes from "pressed" to "not pressed".
    ///
    /// # Arguments
    ///
    /// * `gamepad_id` - ID of the gamepad to check the button of, must be retrieved with [`Context::gamepad_ids`].
    /// * `button` - Which button on the gamepad to check.
    ///
    /// # Returns
    ///
    /// - `None` when gamepad is not found or is disconnected.
    /// - `Some(true)` when gamepad button is being released this update tick.
    #[inline]
    #[must_use]
    pub fn gamepad_button_released(&self, gamepad_id: GamepadId, button: Button) -> Option<bool> {
        self.read(|ctx| ctx.input.gamepad_button_released(gamepad_id, button))
    }

    /// Whether a gamepad button is in a "pressed" state.
    ///
    /// # Arguments
    ///
    /// * `gamepad_id` - ID of the gamepad to check the button of, must be retrieved with [`Context::gamepad_ids`].
    /// * `button` - Which button on the gamepad to check.
    ///
    /// # Returns
    ///
    /// - `None` when gamepad is not found or is disconnected.
    /// - `Some(true)` when gamepad button is being pressed.
    /// - `Some(false)` when gamepad button is released.
    #[inline]
    #[must_use]
    pub fn gamepad_button_held(&self, gamepad_id: GamepadId, button: Button) -> Option<bool> {
        self.read(|ctx| ctx.input.gamepad_button_held(gamepad_id, button))
    }

    /// "Value" of a gamepad button between 0.0 and 1.0.
    ///
    /// Used for triggers.
    ///
    /// # Arguments
    ///
    /// * `gamepad_id` - ID of the gamepad to check the button of, must be retrieved with [`Context::gamepad_ids`].
    /// * `button` - Which button element on the gamepad to check.
    ///
    /// # Returns
    ///
    /// - `None` when gamepad is not found or is disconnected.
    /// - `Some(0.0)` when gamepad axis element is not moved.
    /// - `Some(1.0)` when gamepad axis element is fully engaged.
    #[inline]
    #[must_use]
    pub fn gamepad_button_value(&self, gamepad_id: GamepadId, button: Button) -> Option<f32> {
        self.read(|ctx| ctx.input.gamepad_button_value(gamepad_id, button))
    }

    /// "Value" of a gamepad element between -1.0 and 1.0.
    ///
    /// Used for sticks.
    ///
    /// # Arguments
    ///
    /// * `gamepad_id` - ID of the gamepad to check the axis of, must be retrieved with [`Context::gamepad_ids`].
    /// * `axis` - Which axis element on the gamepad to check.
    ///
    /// # Returns
    ///
    /// - `None` when gamepad is not found or is disconnected.
    /// - `Some(0.0)` when gamepad axis element is not moved.
    /// - `Some(1.0)` when gamepad axis element is fully engaged.
    /// - `Some(-1.0)` when gamepad axis element is fully negatively engaged, for example a horizontal axis being moved left.
    #[inline]
    #[must_use]
    pub fn gamepad_axis(&self, gamepad_id: GamepadId, axis: Axis) -> Option<f32> {
        self.read(|ctx| ctx.input.gamepad_axis(gamepad_id, axis))
    }
}

/// Generic asset loading.
impl Context {
    /// Load a read-only reference to a custom defined asset.
    ///
    /// # Arguments
    ///
    /// * `path` - Asset path of the custom asset, see [`Self`] for more information about asset loading and storing.
    ///
    /// # Panics
    ///
    /// - When asset with path does not exist.
    /// - When asset could not be loaded due to an invalid format.
    #[inline]
    pub fn asset<T>(&self, path: impl AsRef<str>) -> Rc<T>
        where
            T: Loadable,
    {
        self.write(|ctx| ctx.custom(path.as_ref()))
    }

    /// Load a clone of a custom defined asset.
    ///
    /// # Arguments
    ///
    /// * `path` - Asset path of the custom asset, see [`Self`] for more information about asset loading and storing.
    ///
    /// # Panics
    ///
    /// - When asset with path does not exist.
    /// - When asset could not be loaded due to an invalid format.
    #[inline]
    pub fn asset_owned<T>(&self, path: impl AsRef<str>) -> T
        where
            T: Loadable + Clone,
    {
        self.write(|ctx| ctx.custom_owned(path.as_ref()))
    }
}

/// Internally used methods.
impl Context {
    /// Create a new empty context.
    #[inline]
    pub(crate) async fn new(config: Config, window: Window) -> Self {
        // Setup the inner context
        let context_inner = ContextInner::new(config, window).await;

        // Wrap in a reference counted refcell so it can safely be passed to any of the user functions
        let inner = Rc::new(RefCell::new(context_inner));

        Self { inner }
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
pub struct ContextInner {
    /// Sources for loading assets from memory or disk.
    pub asset_source: AssetSource,
    /// Window instance.
    pub(crate) window: Arc<Window>,
    /// Graphics state.
    pub(crate) graphics: Graphics,
    /// Frames per second for the render tick.
    pub(crate) frames_per_second: f32,
    /// Interpolation alpha for the render tick.
    pub(crate) blending_factor: f32,
    /// Input manager.
    pub(crate) input: Input,
    /// User supplied game configuration.
    config: Config,
    /// Sprite assets.
    sprites: AssetManager<Sprite>,
    /// Font assets.
    fonts: AssetManager<Font>,
    /// Audio assets.
    audio: AssetManager<Audio>,
    /// Custom type erased assets.
    custom: CustomAssetManager,
}

impl ContextInner {
    /// Initialize the inner context.
    pub(crate) async fn new(config: Config, window: Window) -> Self {
        // Wrap in an arc so graphics can use it
        let window = Arc::new(window);

        // Setup the initial graphics
        let graphics = Graphics::new(config.clone(), Arc::clone(&window)).await;

        // Load the assets
        let asset_source = AssetSource::new().with_runtime_dir("assets");
        let sprites = AssetManager::default();
        let fonts = AssetManager::default();
        let audio = AssetManager::default();
        let custom = CustomAssetManager::default();

        // Define default values for the timing functions
        let frames_per_second = 0.0;
        let blending_factor = 0.0;

        // Default input values and state
        let input = Input::new();

        Self {
            asset_source,
            window,
            graphics,
            frames_per_second,
            blending_factor,
            input,
            config,
            sprites,
            fonts,
            audio,
            custom,
        }
    }

    /// Get or load a sprite.
    ///
    /// # Panics
    ///
    /// - When font sprite could not be loaded.
    #[inline]
    pub(crate) fn sprite(&mut self, id: &str) -> Rc<Sprite> {
        // Create the ID
        let id = Id::new(id);

        // Try to load the asset first
        if let Some(asset) = self.sprites.get(&id) {
            return asset;
        }

        // Asset not found, load it
        let asset = Sprite::load(&id, self);
        self.sprites.insert(id, asset)
    }

    /// Get or load a font.
    ///
    /// # Panics
    ///
    /// - When font asset could not be loaded.
    #[inline]
    pub(crate) fn font(&mut self, id: &str) -> Rc<Font> {
        // Create the ID
        let id = Id::new(id);

        // Try to load the asset first
        if let Some(asset) = self.fonts.get(&id) {
            return asset;
        }

        // Asset not found, load it
        let asset = Font::load(&id, self);
        self.fonts.insert(id, asset)
    }

    /// Get or load an audio file.
    ///
    /// # Panics
    ///
    /// - When audio asset could not be loaded.
    #[inline]
    pub(crate) fn audio(&mut self, id: &str) -> Rc<Audio> {
        // Create the ID
        let id = Id::new(id);

        // Try to load the asset first
        if let Some(asset) = self.audio.get(&id) {
            return asset;
        }

        // Asset not found, load it
        let asset = Audio::load(&id, self);
        self.audio.insert(id, asset)
    }

    /// Get or load a custom asset.
    ///
    /// # Panics
    ///
    /// - When audio asset could not be loaded.
    /// - When type used to load the asset mismatches the type used to get it.
    #[inline]
    pub(crate) fn custom<T>(&mut self, id: &str) -> Rc<T>
        where
            T: Loadable,
    {
        // Create the ID
        let id = Id::new(id);

        // Try to load the asset first
        if let Some(asset) = self.custom.get(&id) {
            return asset;
        }

        // Asset not found, load it
        let asset = T::load(&id, self);
        self.custom.insert(id, asset)
    }

    /// Get a clone or load a custom asset.
    ///
    /// # Panics
    ///
    /// - When audio asset could not be loaded.
    /// - When type used to load the asset mismatches the type used to get it.
    #[inline]
    pub(crate) fn custom_owned<T>(&mut self, id: &str) -> T
        where
            T: Loadable + Clone,
    {
        // Create a clone of the asset
        Rc::<T>::unwrap_or_clone(self.custom(id))
    }
}
