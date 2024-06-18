use std::{cell::RefCell, rc::Rc, sync::Arc};

use winit::window::Window;

use crate::{config::Config, graphics::Graphics};

/// Context containing most functionality for interfacing with the game engine.
///
/// Exposed in [`crate::PixelGame::update`] and [`crate::PixelGame::render`].
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
/// It's assumed for this table that [`crate::load_assets`] in [`crate::PixelGame`] is called without any arguments or with `chuot::load_assets!("assets/")`.
#[derive(Clone)]
pub struct Context {
    /// Implementation of all non-primitive parts.
    inner: Rc<RefCell<ContextInner>>,
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
pub(crate) struct ContextInner {
    /// Window instance.
    pub(crate) window: Arc<Window>,
    /// Graphics state.
    pub(crate) graphics: Graphics,
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

        Self {
            config,
            graphics,
            window,
        }
    }
}
