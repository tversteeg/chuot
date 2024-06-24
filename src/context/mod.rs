//! Main interface with the game.

pub mod sprite;

use std::{cell::RefCell, rc::Rc, sync::Arc};

use winit::window::{Fullscreen, Window};

use crate::{
    assets::{
        loadable::{audio::Audio, font::Font, sprite::Sprite, Loadable},
        source::AssetSource,
        AssetManager, CustomAssetManager, Id,
    },
    config::Config,
    graphics::Graphics,
};

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
    /// // In `PixelGame::render` trait implementation
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

        Self {
            asset_source,
            window,
            graphics,
            frames_per_second,
            blending_factor,
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
