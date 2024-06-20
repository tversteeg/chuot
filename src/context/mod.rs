//! Main interface with the game.

pub mod sprite;

use std::{cell::RefCell, rc::Rc, sync::Arc};

use winit::window::{Fullscreen, Window};

use crate::{
    assets::{source::AssetSource, Assets},
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
        self.read(|ctx| {
            (
                ctx.graphics.config.width as f32,
                ctx.graphics.config.height as f32,
            )
        })
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

/// Internally used methods.
impl Context {
    /// Create a new empty context.
    #[inline]
    pub(crate) async fn new(config: Config, window: Window) -> Self {
        // Setup the inner context
        let context_inner = ContextInner::new(config, window).await;

        // Wrap in a reference counted refcell so it can safely be passed to any of the user functions
        let inner = Rc::new(RefCell::new(context_inner));

        let this = Self { inner };

        // Set the created context so it can be referenced
        this.write(|ctx| ctx.assets.source.ctx = Some(this.clone()));

        this
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
    /// Window instance.
    pub(crate) window: Arc<Window>,
    /// Graphics state.
    pub(crate) graphics: Graphics,
    // /// Assets.
    pub(crate) assets: Assets,
    /// User supplied game configuration.
    config: Config,
}

impl ContextInner {
    /// Initialize the inner context.
    pub(crate) async fn new(config: Config, window: Window) -> Self {
        // Wrap in an arc so graphics can use it
        let window = Arc::new(window);

        // Setup the initial graphics
        let graphics = Graphics::new(&config, Arc::clone(&window)).await;

        // Load the assets
        let assets = Assets::new(AssetSource::new().with_runtime_dir("assets"));

        Self {
            window,
            graphics,
            assets,
            config,
        }
    }
}
