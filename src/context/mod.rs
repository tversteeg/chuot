use std::{cell::RefCell, rc::Rc};

use crate::backend::{wgpu::WgpuWinitBackend, Backend};

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
pub struct Context<B: Backend = WgpuWinitBackend> {
    /// Implementation of all non-primitive parts.
    inner: Rc<RefCell<ContextInner<B>>>,
}

/// Internally used methods.
impl<B: Backend> Context<B> {
    /// Create a new empty context.
    #[inline]
    pub(crate) fn new(backend: B) -> Self {
        Self {
            inner: Rc::new(RefCell::new(ContextInner::new(backend))),
        }
    }

    /// Get a read-only reference to the inner struct.
    ///
    /// # Panics
    ///
    /// - When internal [`RwLock`] is poisoned.
    #[inline]
    pub(crate) fn read<R>(&self, reader: impl FnOnce(&ContextInner<B>) -> R) -> R {
        reader(&self.inner.borrow())
    }

    /// Get a mutable reference to the inner struct.
    ///
    /// # Panics
    ///
    /// - When internal [`RwLock`] is poisoned.
    #[inline]
    pub(crate) fn write<R>(&self, writer: impl FnOnce(&mut ContextInner<B>) -> R) -> R {
        writer(&mut self.inner.borrow_mut())
    }
}

/// Internal wrapped implementation for [`Context`].
pub(crate) struct ContextInner<B: Backend> {
    /// Backend state.
    backend: B,
}

impl<B: Backend> ContextInner<B> {
    /// Initialize the inner context.
    pub(crate) fn new(backend: B) -> Self {
        Self { backend }
    }
}
