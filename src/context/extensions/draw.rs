//! Drawing.

/// Allow drawing to the screen.
pub trait Draw: Sized {
    /// Draw to the screen.
    ///
    /// # Panics
    ///
    /// - When asset failed loading.
    fn draw(self);
}
